//! Throwaway prototype: five structurally different renderings of a Map, switched at runtime.
//!
//! Nothing here is production code. Run it with `just proto::map`.

pub mod fixtures;
pub mod map;
mod variant_a_canvas;
mod variant_b_rail;
mod variant_c_board;
mod variant_d_document;
mod variant_e_outline;

use std::io;

use ratatui::Frame;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use fixtures::Fixtures;
use map::Map;

/// The Ticket the user is looking at, carried across variant and Map switches.
#[derive(Default)]
pub struct Selection {
    ticket: Option<String>,
}

impl Selection {
    pub fn ticket_id(&self) -> Option<&str> {
        self.ticket.as_deref()
    }

    pub fn is(&self, id: &str) -> bool {
        self.ticket.as_deref() == Some(id)
    }

    pub fn select(&mut self, id: &str) {
        self.ticket = Some(id.to_string());
    }

    pub fn index_in(&self, map: &Map) -> Option<usize> {
        self.ticket.as_deref().and_then(|id| map.index_of(id))
    }

    /// Moves the selection along the Map's Ticket order, wrapping at both ends.
    pub fn step(&mut self, map: &Map, delta: isize) {
        if map.tickets.is_empty() {
            self.ticket = None;
            return;
        }
        let count = map.tickets.len() as isize;
        let current = self.index_in(map).unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(count) as usize;
        self.ticket = Some(map.tickets[next].id.clone());
    }

    /// Keeps the selection when the Ticket exists in `map`, otherwise falls back to its first.
    fn reconcile(&mut self, map: &Map) {
        let lands = self
            .ticket
            .as_deref()
            .is_some_and(|id| map.ticket(id).is_some());
        if !lands {
            self.ticket = map.tickets.first().map(|ticket| ticket.id.clone());
        }
    }
}

/// One rendering of a Map. Variants share the data and the switcher, never the layout.
pub trait Variant {
    /// Shown in the bottom bar next to the variant key.
    fn name(&self) -> &'static str;

    fn draw(&mut self, frame: &mut Frame, area: Rect, map: &Map, selection: &Selection);

    /// Keys the switcher does not claim. Default is to ignore them.
    fn handle_key(&mut self, _key: KeyEvent, _map: &Map, _selection: &mut Selection) {}
}

fn variants() -> Vec<Box<dyn Variant>> {
    vec![
        Box::new(variant_a_canvas::Canvas::default()),
        Box::new(variant_b_rail::Rail::default()),
        Box::new(variant_c_board::Board::default()),
        Box::new(variant_d_document::Document::default()),
        Box::new(variant_e_outline::Outline::default()),
    ]
}

pub struct Prototype {
    fixtures: Fixtures,
    variants: Vec<Box<dyn Variant>>,
    variant: usize,
    map: usize,
    selection: Selection,
    quit: bool,
}

impl Prototype {
    pub fn new(fixtures: Fixtures) -> Self {
        let mut prototype = Self {
            fixtures,
            variants: variants(),
            variant: 0,
            map: 0,
            selection: Selection::default(),
            quit: false,
        };
        prototype.reconcile();
        prototype
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    fn current_map(&self) -> Option<&Map> {
        self.fixtures.maps.get(self.map)
    }

    fn reconcile(&mut self) {
        if let Some(map) = self.fixtures.maps.get(self.map) {
            self.selection.reconcile(map);
        }
    }

    fn cycle_variant(&mut self, delta: isize) {
        if self.variants.is_empty() {
            return;
        }
        let count = self.variants.len() as isize;
        self.variant = (self.variant as isize + delta).rem_euclid(count) as usize;
        self.reconcile();
    }

    fn cycle_map(&mut self, delta: isize) {
        if self.fixtures.maps.is_empty() {
            return;
        }
        let count = self.fixtures.maps.len() as isize;
        self.map = (self.map as isize + delta).rem_euclid(count) as usize;
        self.reconcile();
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let ctrl_c =
            key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl_c || key.code == KeyCode::Char('q') {
            self.quit = true;
            return;
        }
        match key.code {
            KeyCode::Tab => self.cycle_variant(1),
            KeyCode::BackTab => self.cycle_variant(-1),
            KeyCode::Char(']') => self.cycle_map(1),
            KeyCode::Char('[') => self.cycle_map(-1),
            _ => {
                let (Some(variant), Some(map)) = (
                    self.variants.get_mut(self.variant),
                    self.fixtures.maps.get(self.map),
                ) else {
                    return;
                };
                variant.handle_key(key, map, &mut self.selection);
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let [body, bar] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

        match (
            self.variants.get_mut(self.variant),
            self.fixtures.maps.get(self.map),
        ) {
            (Some(variant), Some(map)) => variant.draw(frame, body, map, &self.selection),
            _ => frame.render_widget(self.empty_state(), body),
        }

        frame.render_widget(self.bar(), bar);
    }

    fn empty_state(&self) -> Paragraph<'_> {
        let mut lines = vec![
            Line::from("No Map loaded."),
            Line::from(format!("Looked in {}", self.fixtures.dir.display())),
            Line::from(""),
        ];
        lines.extend(
            self.fixtures
                .notes
                .iter()
                .map(String::as_str)
                .map(Line::from),
        );
        Paragraph::new(lines)
    }

    fn bar(&self) -> Paragraph<'_> {
        let key = (b'A' + self.variant as u8) as char;
        let variant = self
            .variants
            .get(self.variant)
            .map_or("none", |variant| variant.name());

        let mut spans = vec![
            Span::styled(
                format!(" {key} {variant} "),
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ];

        match self.current_map() {
            Some(map) => spans.push(Span::raw(format!(
                "{}/{} {}  {} tickets, {} open",
                self.map + 1,
                self.fixtures.maps.len(),
                map.id,
                map.tickets.len(),
                map.open_tickets(),
            ))),
            None => spans.push(Span::raw("no Map")),
        }

        if let Some(id) = self.selection.ticket_id() {
            spans.push(Span::raw(format!("  sel {id}")));
        }
        if !self.fixtures.notes.is_empty() {
            spans.push(Span::styled(
                format!("  {}", self.fixtures.notes.join("; ")),
                Style::new().fg(Color::Yellow),
            ));
        }
        spans.push(Span::styled(
            "  Tab variant   [ ] Map   q quit",
            Style::new().fg(Color::DarkGray),
        ));

        Paragraph::new(Line::from(spans))
    }
}

/// Runs the prototype against the Maps found by [`fixtures::fixtures_dir`].
pub fn run() -> io::Result<()> {
    let fixtures = fixtures::load(&fixtures::fixtures_dir());
    ratatui::run(|terminal| {
        let mut prototype = Prototype::new(fixtures);
        while !prototype.should_quit() {
            terminal.draw(|frame| prototype.draw(frame))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                prototype.handle_key(key);
            }
        }
        Ok(())
    })
}

/// Prints what the loader parsed, without starting the terminal.
pub fn list() {
    let fixtures = fixtures::load(&fixtures::fixtures_dir());
    println!("{}", fixtures.dir.display());
    for note in &fixtures.notes {
        println!("  note: {note}");
    }
    for map in &fixtures.maps {
        println!(
            "\n{} [{}]\n  {} sections: {}\n  {} tickets, {} open, {} edges, {} inline links",
            map.title,
            map.id,
            map.sections.len(),
            map.sections
                .iter()
                .map(|section| section.heading.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            map.tickets.len(),
            map.open_tickets(),
            map.edges().len(),
            map.source
                .iter()
                .flat_map(|line| map::split_ticket_links(line))
                .filter(|part| matches!(part, map::LinePart::Link { .. }))
                .count(),
        );
        for ticket in &map.tickets {
            println!(
                "    {:<9} {:<7} {:<40} blocked_by [{}] ready={} gist: {}",
                ticket.kind.label(),
                ticket.status.label(),
                ticket.label,
                ticket.blocked_by.join(", "),
                map.is_ready(ticket),
                ticket.gist().chars().take(60).collect::<String>(),
            );
        }
    }
}
