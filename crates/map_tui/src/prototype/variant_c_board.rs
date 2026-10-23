use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, Widget};
use tui_widget_list::{ListBuilder, ListState, ListView};

use super::map::{Map, Section, Ticket, TicketKind, TicketStatus};
use super::{Selection, Variant};

const LANES: usize = 3;
const BAR: &str = "▌";
const SELECTED_BG: Color = Color::Indexed(237);
const MUTED: Color = Color::Indexed(245);
const FAINT: Color = Color::Indexed(240);
const RULE: Color = Color::Indexed(238);
const WIDE_LABEL: usize = 40;
const EXPANDED_GIST: usize = 4;
const HINTS: &str = "j k card   h l column   b blocker   f dependent   enter detail";

pub struct Board {
    columns: [ListState; LANES],
    detail: bool,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            columns: Default::default(),
            detail: true,
        }
    }
}

impl Variant for Board {
    fn name(&self) -> &'static str {
        "Board"
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, map: &Map, selection: &Selection) {
        if area.width < 6 || area.height < 2 {
            return;
        }

        let lanes = board(map, selection);
        let rows = header_rows(area.height);
        let [head, body] =
            Layout::vertical([Constraint::Length(rows), Constraint::Fill(1)]).areas(area);
        if rows > 0 {
            frame.render_widget(Paragraph::new(header(map, head.width, rows)), head);
        }
        if body.height < 2 {
            return;
        }

        let columns: [Rect; LANES] = Layout::horizontal([Constraint::Fill(1); LANES])
            .spacing(1)
            .areas(body);
        let inner = body.height.saturating_sub(1) as usize;
        let detail = self.detail;

        for (index, lane) in Lane::ALL.iter().enumerate() {
            let area = columns[index];
            let cards = &lanes[index];
            let roomy = measure(cards, area.width, true, false) <= inner;
            let overflows = measure(cards, area.width, roomy, detail) > inner;
            let block = Block::new()
                .borders(Borders::TOP)
                .border_style(Style::new().fg(RULE))
                .title(lane_title(*lane, cards.len(), overflows));

            if cards.is_empty() {
                let note = Line::styled(
                    format!(
                        " {}",
                        clip(
                            empty_note(*lane, map, &lanes),
                            (area.width as usize).saturating_sub(1)
                        )
                    ),
                    Style::new().fg(FAINT),
                );
                frame.render_widget(Paragraph::new(note).block(block), area);
                continue;
            }

            let reserve = u16::from(overflows);
            let builder = ListBuilder::new(move |context| {
                let card = &cards[context.index];
                let lines = card_lines(
                    card,
                    context.cross_axis_size.saturating_sub(reserve),
                    roomy,
                    detail,
                );
                let height = lines.len() as u16;
                (
                    CardWidget {
                        lines,
                        style: card.background(),
                    },
                    height,
                )
            });

            let mut view = ListView::new(builder, cards.len())
                .block(block)
                .infinite_scrolling(false);
            if overflows {
                view = view.scrollbar(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(None)
                        .end_symbol(None)
                        .style(Style::new().fg(RULE)),
                );
            }

            let state = &mut self.columns[index];
            state.select(cards.iter().position(|card| card.selected()));
            frame.render_stateful_widget(view, area, state);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, map: &Map, selection: &mut Selection) {
        let lanes = board(map, selection);
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => step_row(&lanes, selection, 1),
            KeyCode::Char('k') | KeyCode::Up => step_row(&lanes, selection, -1),
            KeyCode::Char('l') | KeyCode::Right => step_lane(&lanes, selection, 1),
            KeyCode::Char('h') | KeyCode::Left => step_lane(&lanes, selection, -1),
            KeyCode::Char('g') | KeyCode::Home => jump_row(&lanes, selection, 0),
            KeyCode::Char('G') | KeyCode::End => jump_row(&lanes, selection, usize::MAX),
            KeyCode::Char('b') => follow(selection, first_blocker(map, selection)),
            KeyCode::Char('f') => follow(selection, first_dependent(map, selection)),
            KeyCode::Enter | KeyCode::Char(' ') => self.detail = !self.detail,
            KeyCode::Esc => self.detail = false,
            _ => {}
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    Ready,
    Waiting,
    Done,
}

impl Lane {
    const ALL: [Self; LANES] = [Self::Ready, Self::Waiting, Self::Done];

    /// Closed Tickets are done, the rest split on whether every blocker is closed.
    fn of(map: &Map, ticket: &Ticket) -> Self {
        if ticket.status == TicketStatus::Closed {
            Self::Done
        } else if map.is_ready(ticket) {
            Self::Ready
        } else {
            Self::Waiting
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Ready => 0,
            Self::Waiting => 1,
            Self::Done => 2,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Waiting => "Waiting",
            Self::Done => "Done",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Ready => Color::Green,
            Self::Waiting => Color::Yellow,
            Self::Done => Color::DarkGray,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Relation {
    None,
    Selected,
    Blocker,
    Dependent,
}

struct Card {
    id: String,
    short: String,
    label: String,
    gist: String,
    kind: TicketKind,
    status: TicketStatus,
    needs: usize,
    feeds: usize,
    lane: Lane,
    relation: Relation,
}

impl Card {
    fn selected(&self) -> bool {
        self.relation == Relation::Selected
    }

    fn accent(&self) -> Style {
        let color = match self.relation {
            Relation::Blocker => Color::LightMagenta,
            Relation::Dependent => Color::LightCyan,
            _ => self.lane.color(),
        };
        let style = Style::new().fg(color);
        if self.selected() {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        }
    }

    fn background(&self) -> Style {
        if self.selected() {
            Style::new().bg(SELECTED_BG)
        } else {
            Style::new()
        }
    }

    fn label_style(&self) -> Style {
        if self.selected() {
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD)
        } else if self.lane == Lane::Done {
            Style::new().fg(MUTED)
        } else {
            Style::new().fg(Color::Gray)
        }
    }

    fn status_span(&self) -> Span<'static> {
        let (glyph, color) = match self.status {
            TicketStatus::Open => ("●", Color::LightGreen),
            TicketStatus::Closed => ("✓", MUTED),
            TicketStatus::Unknown => ("?", Color::Yellow),
        };
        Span::styled(glyph, Style::new().fg(color))
    }

    fn kind_span(&self) -> Span<'static> {
        let color = match self.kind {
            TicketKind::Grilling => Color::Magenta,
            TicketKind::Research => Color::Cyan,
            TicketKind::Task => Color::Blue,
            TicketKind::Prototype => Color::LightYellow,
            TicketKind::Unknown => Color::DarkGray,
        };
        Span::styled(self.kind.label(), Style::new().fg(color))
    }
}

struct CardWidget {
    lines: Vec<Line<'static>>,
    style: Style,
}

impl Widget for CardWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        Paragraph::new(self.lines).render(area, buf);
    }
}

/// Ready is ordered by how many Tickets each one unblocks, so its top card is the decision worth
/// making first. The other two lanes keep Map order.
fn board(map: &Map, selection: &Selection) -> [Vec<Card>; LANES] {
    let mut lanes: [Vec<Card>; LANES] = Default::default();
    for ticket in &map.tickets {
        let lane = Lane::of(map, ticket);
        lanes[lane.index()].push(card(map, ticket, lane, selection));
    }
    lanes[Lane::Ready.index()].sort_by_key(|card| std::cmp::Reverse(card.feeds));
    lanes
}

fn card(map: &Map, ticket: &Ticket, lane: Lane, selection: &Selection) -> Card {
    Card {
        short: short_id(&ticket.id),
        label: ticket.label.clone(),
        gist: ticket.gist().trim().to_string(),
        kind: ticket.kind,
        status: ticket.status,
        needs: ticket.blocked_by.len(),
        feeds: map.blocks(&ticket.id).len(),
        lane,
        relation: relation(map, ticket, selection),
        id: ticket.id.clone(),
    }
}

fn relation(map: &Map, ticket: &Ticket, selection: &Selection) -> Relation {
    let Some(selected) = selection.ticket_id() else {
        return Relation::None;
    };
    if selected == ticket.id {
        return Relation::Selected;
    }
    if ticket.blocked_by.iter().any(|id| id == selected) {
        return Relation::Dependent;
    }
    let blocks_selected = map
        .ticket(selected)
        .is_some_and(|target| target.blocked_by.contains(&ticket.id));
    if blocks_selected {
        Relation::Blocker
    } else {
        Relation::None
    }
}

/// Card rows, as tall as the content needs, with a trailing spacer row. `roomy` is decided from
/// the column's compact height alone, so moving the selection never relays the column out.
fn card_lines(card: &Card, width: u16, roomy: bool, detail: bool) -> Vec<Line<'static>> {
    let accent = card.accent();
    let text = (width as usize).saturating_sub(2);
    if text == 0 {
        return vec![Line::from(Span::styled(BAR, accent))];
    }

    let expanded = card.selected() && detail;
    let mut rows: Vec<Vec<Span<'static>>> = Vec::new();

    let cap = if text >= WIDE_LABEL { 1 } else { 2 };
    let mut label = wrap(&card.label, text, cap);
    if label.is_empty() {
        label.push(String::new());
    }
    for line in label {
        rows.push(vec![Span::styled(line, card.label_style())]);
    }

    let gist = if expanded {
        EXPANDED_GIST
    } else {
        usize::from(roomy)
    };
    for line in wrap(&card.gist, text, gist) {
        rows.push(vec![Span::styled(line, Style::new().fg(FAINT))]);
    }

    rows.push(meta_row(card, text));
    rows.push(Vec::new());

    let last = rows.len() - 1;
    rows.into_iter()
        .enumerate()
        .map(|(row, spans)| {
            let bar = if row == last { " " } else { BAR };
            let mut line = vec![Span::styled(bar, accent), Span::raw(" ")];
            line.extend(spans);
            Line::from(line)
        })
        .collect()
}

/// Identity on the left, dependency badge and selection relation on the right.
fn meta_row(card: &Card, width: usize) -> Vec<Span<'static>> {
    let left = vec![
        Span::styled(card.short.clone(), Style::new().fg(MUTED)),
        Span::raw(" "),
        card.status_span(),
        Span::raw(" "),
        card.kind_span(),
    ];
    let left_width = card.short.chars().count() + 4 + card.kind.label().chars().count();

    let mut badges: Vec<Span<'static>> = Vec::new();
    match card.relation {
        Relation::Blocker => badges.push(Span::styled(
            "blocker",
            Style::new().fg(Color::LightMagenta),
        )),
        Relation::Dependent => {
            badges.push(Span::styled("dependent", Style::new().fg(Color::LightCyan)))
        }
        _ => {}
    }
    if card.needs > 0 {
        badges.push(Span::styled(
            format!("needs {}", card.needs),
            Style::new().fg(MUTED),
        ));
    }
    if card.feeds > 0 {
        badges.push(Span::styled(
            format!("feeds {}", card.feeds),
            Style::new().fg(FAINT),
        ));
    }

    while !badges.is_empty() && left_width + 2 + badge_width(&badges) > width {
        badges.pop();
    }
    if badges.is_empty() {
        return left;
    }

    let mut spans = left;
    spans.push(Span::raw(
        " ".repeat(width.saturating_sub(left_width + badge_width(&badges))),
    ));
    for (index, badge) in badges.into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(badge);
    }
    spans
}

fn badge_width(badges: &[Span<'_>]) -> usize {
    let text: usize = badges
        .iter()
        .map(|badge| badge.content.chars().count())
        .sum();
    text + badges.len().saturating_sub(1)
}

fn measure(cards: &[Card], width: u16, roomy: bool, detail: bool) -> usize {
    cards
        .iter()
        .map(|card| card_lines(card, width, roomy, detail).len())
        .sum()
}

fn lane_title(lane: Lane, count: usize, overflows: bool) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            format!(" {} ", lane.title()),
            Style::new().fg(lane.color()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{count} "), Style::new().fg(MUTED)),
    ];
    if overflows {
        spans.push(Span::styled("scrolls ", Style::new().fg(FAINT)));
    }
    Line::from(spans)
}

fn empty_note(lane: Lane, map: &Map, lanes: &[Vec<Card>; LANES]) -> &'static str {
    match lane {
        Lane::Ready => {
            if map.tickets.is_empty() {
                "no Tickets under this Map"
            } else if lanes[Lane::Done.index()].len() == map.tickets.len() {
                "every Ticket is closed"
            } else {
                "everything open is blocked"
            }
        }
        Lane::Waiting => {
            if map.edges().is_empty() {
                "no Ticket waits on another"
            } else {
                "nothing is blocked"
            }
        }
        Lane::Done => {
            if map
                .tickets
                .iter()
                .any(|ticket| ticket.status == TicketStatus::Unknown)
            {
                "no Ticket states closed"
            } else {
                "nothing closed yet"
            }
        }
    }
}

fn header_rows(height: u16) -> u16 {
    match height {
        0..=5 => 0,
        6..=9 => 1,
        10..=13 => 2,
        _ => 3,
    }
}

fn header(map: &Map, width: u16, rows: u16) -> Vec<Line<'static>> {
    let width = width as usize;
    let closed = map
        .tickets
        .iter()
        .filter(|ticket| ticket.status == TicketStatus::Closed)
        .count();
    let progress = format!("{closed} of {} closed", map.tickets.len());

    let mut lines = vec![spread(
        Span::styled(
            clip(
                &map.title,
                width.saturating_sub(progress.chars().count() + 2),
            ),
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Some(Span::styled(progress, Style::new().fg(MUTED))),
        width,
    )];

    if rows >= 2 {
        let unknown = map
            .tickets
            .iter()
            .filter(|ticket| ticket.status == TicketStatus::Unknown)
            .count();
        let note = (unknown > 0).then(|| {
            Span::styled(
                format!("{unknown} state no status"),
                Style::new().fg(Color::Yellow),
            )
        });
        let room = width.saturating_sub(
            note.as_ref()
                .map_or(0, |span| span.content.chars().count() + 2),
        );
        let destination = map.destination().map(Section::gist).unwrap_or_default();
        lines.push(spread(
            Span::styled(clip(destination, room), Style::new().fg(FAINT)),
            note,
            width,
        ));
    }

    if rows >= 3 {
        lines.push(Line::styled(
            clip(HINTS, width),
            Style::new().fg(Color::DarkGray),
        ));
    }
    lines
}

fn spread(left: Span<'static>, right: Option<Span<'static>>, width: usize) -> Line<'static> {
    let Some(right) = right else {
        return Line::from(left);
    };
    let used = left.content.chars().count() + right.content.chars().count();
    if used + 1 > width {
        return Line::from(left);
    }
    Line::from(vec![left, Span::raw(" ".repeat(width - used)), right])
}

fn short_id(id: &str) -> String {
    let head = id.split('-').next().unwrap_or(id);
    if !head.is_empty() && head.chars().all(|char| char.is_ascii_digit()) {
        return head.to_string();
    }
    clip(id, 6)
}

fn clip(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if text.chars().count() <= width {
        return text.to_string();
    }
    let mut clipped: String = text.chars().take(width - 1).collect();
    clipped.push('…');
    clipped
}

/// Greedy word wrap, with the last line ellipsised when the text runs past `max`.
fn wrap(text: &str, width: usize, max: usize) -> Vec<String> {
    if width == 0 || max == 0 {
        return Vec::new();
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
        while current.chars().count() > width {
            lines.push(current.chars().take(width).collect());
            current = current.chars().skip(width).collect();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.len() > max {
        lines.truncate(max);
        if let Some(last) = lines.last_mut() {
            let head: String = last.chars().take(width.saturating_sub(1)).collect();
            *last = format!("{head}…");
        }
    }
    lines
}

fn locate(lanes: &[Vec<Card>; LANES], selection: &Selection) -> Option<(usize, usize)> {
    lanes.iter().enumerate().find_map(|(index, cards)| {
        cards
            .iter()
            .position(|card| selection.is(&card.id))
            .map(|row| (index, row))
    })
}

fn first_card(lanes: &[Vec<Card>; LANES], selection: &mut Selection) {
    if let Some(card) = lanes.iter().find_map(|cards| cards.first()) {
        selection.select(&card.id);
    }
}

fn step_row(lanes: &[Vec<Card>; LANES], selection: &mut Selection, delta: isize) {
    let Some((lane, row)) = locate(lanes, selection) else {
        first_card(lanes, selection);
        return;
    };
    let count = lanes[lane].len() as isize;
    let next = (row as isize + delta).rem_euclid(count) as usize;
    selection.select(&lanes[lane][next].id);
}

fn jump_row(lanes: &[Vec<Card>; LANES], selection: &mut Selection, row: usize) {
    let Some((lane, _)) = locate(lanes, selection) else {
        first_card(lanes, selection);
        return;
    };
    let cards = &lanes[lane];
    selection.select(&cards[row.min(cards.len() - 1)].id);
}

/// Keeps the row while crossing to the next lane that holds any card.
fn step_lane(lanes: &[Vec<Card>; LANES], selection: &mut Selection, delta: isize) {
    let Some((lane, row)) = locate(lanes, selection) else {
        first_card(lanes, selection);
        return;
    };
    let mut next = lane as isize;
    for _ in 0..LANES {
        next = (next + delta).rem_euclid(LANES as isize);
        let cards = &lanes[next as usize];
        if let Some(card) = cards.get(row.min(cards.len().saturating_sub(1))) {
            selection.select(&card.id);
            return;
        }
    }
}

fn first_blocker(map: &Map, selection: &Selection) -> Option<String> {
    let ticket = map.ticket(selection.ticket_id()?)?;
    ticket
        .blocked_by
        .iter()
        .find_map(|id| map.ticket(id))
        .map(|blocker| blocker.id.clone())
}

fn first_dependent(map: &Map, selection: &Selection) -> Option<String> {
    let id = selection.ticket_id()?;
    map.blocks(id).first().map(|dependent| dependent.id.clone())
}

fn follow(selection: &mut Selection, target: Option<String>) {
    if let Some(id) = target {
        selection.select(&id);
    }
}
