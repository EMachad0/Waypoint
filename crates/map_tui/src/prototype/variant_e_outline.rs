//! A Map is an index: the left pane lists every Ticket, the right pane holds the one selected.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Wrap,
};

use super::map::{Map, Ticket, TicketKind, TicketStatus};
use super::{Selection, Variant};

const LIST_SHARE: u32 = 45;
const LIST_MIN: u16 = 22;
const LIST_MAX: u16 = 72;
const BODY_MIN: u16 = 30;
const GUTTER: usize = 6;
const LABEL_MIN: usize = 10;
const HEADER: u16 = 4;
const PROSE_MAX: u16 = 100;
const MARKER_MAX: usize = 11;

/// One Ticket on one line of the outline, at its depth in the spanning tree.
struct Row {
    id: String,
    depth: usize,
    also_blocked_by: Vec<String>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Focus {
    #[default]
    List,
    Body,
}

/// Parent of every Ticket: its first blocker that resolves inside the Map. Cycles are cut by
/// detaching the Ticket that closes them.
fn parents(map: &Map) -> Vec<Option<usize>> {
    let mut parents: Vec<Option<usize>> = map
        .tickets
        .iter()
        .enumerate()
        .map(|(index, ticket)| {
            ticket
                .blocked_by
                .iter()
                .filter_map(|blocker| map.index_of(blocker))
                .find(|blocker| *blocker != index)
        })
        .collect();
    for start in 0..parents.len() {
        let mut walker = parents[start];
        let mut steps = 0;
        while let Some(index) = walker {
            if index == start || steps > parents.len() {
                parents[start] = None;
                break;
            }
            walker = parents[index];
            steps += 1;
        }
    }
    parents
}

fn branch(map: &Map, parents: &[Option<usize>], index: usize, depth: usize, rows: &mut Vec<Row>) {
    let ticket = &map.tickets[index];
    rows.push(Row {
        id: ticket.id.clone(),
        depth,
        also_blocked_by: ticket
            .blocked_by
            .iter()
            .filter(|blocker| map.ticket(blocker).is_some())
            .filter(|blocker| map.index_of(blocker) != parents[index])
            .cloned()
            .collect(),
    });
    for child in 0..map.tickets.len() {
        if parents[child] == Some(index) {
            branch(map, parents, child, depth + 1, rows);
        }
    }
}

/// The Tickets in outline order: roots in Map order, each followed by its subtree.
fn outline(map: &Map) -> Vec<Row> {
    let parents = parents(map);
    let mut rows = Vec::with_capacity(map.tickets.len());
    for index in 0..map.tickets.len() {
        if parents[index].is_none() {
            branch(map, &parents, index, 0, &mut rows);
        }
    }
    rows
}

fn fit(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    text.chars()
        .take(width - 1)
        .chain(std::iter::once('\u{2026}'))
        .collect()
}

/// Two columns naming a Ticket inside its Map, so a blocker marker can be looked up in the list.
fn tag(id: &str) -> String {
    let digits: String = id.chars().take_while(char::is_ascii_digit).collect();
    let short = if digits.is_empty() { id } else { &digits };
    format!("{:>2}", short.chars().take(2).collect::<String>())
}

/// Names the blockers the spanning tree could not indent under, or counts them when naming them
/// all would not fit.
fn blocker_marker(also_blocked_by: &[String]) -> Option<String> {
    if also_blocked_by.is_empty() {
        return None;
    }
    let named = also_blocked_by
        .iter()
        .map(|id| format!("+{}", tag(id)))
        .collect::<Vec<_>>()
        .join(" ");
    if named.chars().count() <= MARKER_MAX {
        Some(named)
    } else {
        Some(format!("+{} more", also_blocked_by.len()))
    }
}

fn status_glyph(status: TicketStatus) -> char {
    match status {
        TicketStatus::Open => '\u{25cf}',
        TicketStatus::Closed => '\u{2713}',
        TicketStatus::Unknown => '\u{b7}',
    }
}

fn kind_glyph(kind: TicketKind) -> char {
    match kind {
        TicketKind::Grilling => 'G',
        TicketKind::Research => 'R',
        TicketKind::Task => 'T',
        TicketKind::Prototype => 'P',
        TicketKind::Unknown => ' ',
    }
}

fn status_style(map: &Map, ticket: &Ticket) -> Style {
    match ticket.status {
        TicketStatus::Closed => Style::new().fg(Color::Green),
        TicketStatus::Open if map.is_ready(ticket) => {
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        }
        _ => Style::new().fg(Color::DarkGray),
    }
}

fn label_style(ticket: &Ticket) -> Style {
    if ticket.status == TicketStatus::Closed {
        Style::new().fg(Color::DarkGray)
    } else {
        Style::new()
    }
}

fn row_line(map: &Map, row: &Row, width: usize) -> Line<'static> {
    let Some(ticket) = map.ticket(&row.id) else {
        return Line::raw(row.id.clone());
    };

    let usable = width.saturating_sub(GUTTER);
    let indent = (row.depth * 2).min(usable.saturating_sub(LABEL_MIN));
    let room = usable - indent;
    let marker = blocker_marker(&row.also_blocked_by)
        .filter(|text| room >= text.chars().count() + 2 + LABEL_MIN);
    let label_room = marker
        .as_ref()
        .map_or(room, |text| room - text.chars().count() - 2);
    let label = fit(&ticket.label, label_room);

    let mut spans = vec![
        Span::styled(
            status_glyph(ticket.status).to_string(),
            status_style(map, ticket),
        ),
        Span::styled(
            format!("{} {} ", kind_glyph(ticket.kind), tag(&ticket.id)),
            Style::new().fg(Color::DarkGray),
        ),
        Span::raw(" ".repeat(indent)),
    ];
    match marker {
        Some(marker) => {
            spans.push(Span::styled(
                format!("{label:<label_room$}  "),
                label_style(ticket),
            ));
            spans.push(Span::styled(marker, Style::new().fg(Color::Magenta)));
        }
        None => spans.push(Span::styled(label, label_style(ticket))),
    }
    Line::from(spans)
}

/// The Map-level counts the outline itself cannot show, for the bottom border of the list.
fn summary(map: &Map) -> String {
    if map.tickets.is_empty() {
        return "no Tickets".to_string();
    }
    let mut parts = vec![format!("{} tickets", map.tickets.len())];
    let unknown = map
        .tickets
        .iter()
        .filter(|ticket| ticket.status == TicketStatus::Unknown)
        .count();
    if unknown == map.tickets.len() {
        parts.push("no status".to_string());
    } else {
        parts.push(format!("{} open", map.open_tickets()));
    }
    if map.edges().is_empty() {
        parts.push("no blockers".to_string());
    } else {
        let ready = map
            .tickets
            .iter()
            .filter(|ticket| ticket.is_open() && map.is_ready(ticket))
            .count();
        parts.push(format!("{ready} ready"));
    }
    parts.join("  ")
}

/// The full blocker and dependent lists, which the indentation cannot carry.
fn relations(map: &Map, ticket: &Ticket) -> Line<'static> {
    let mut spans = Vec::new();
    if !ticket.blocked_by.is_empty() {
        let ids = ticket
            .blocked_by
            .iter()
            .map(|id| tag(id))
            .collect::<Vec<_>>()
            .join(" ");
        spans.push(Span::styled(
            "blocked by ",
            Style::new().fg(Color::DarkGray),
        ));
        spans.push(Span::styled(ids, Style::new().fg(Color::Magenta)));
    }
    let blocks = map.blocks(&ticket.id);
    if !blocks.is_empty() {
        if !spans.is_empty() {
            spans.push(Span::raw("   "));
        }
        let ids = blocks
            .iter()
            .map(|blocked| tag(&blocked.id))
            .collect::<Vec<_>>()
            .join(" ");
        spans.push(Span::styled("blocks ", Style::new().fg(Color::DarkGray)));
        spans.push(Span::styled(ids, Style::new().fg(Color::Cyan)));
    }
    if spans.is_empty() {
        spans.push(Span::styled(
            "no recorded relations",
            Style::new().fg(Color::DarkGray),
        ));
    }
    Line::from(spans)
}

fn facts(map: &Map, ticket: &Ticket) -> Line<'static> {
    let mut spans = vec![
        Span::styled(ticket.status.label(), status_style(map, ticket)),
        Span::raw("  "),
        Span::styled(ticket.kind.label(), Style::new().fg(Color::DarkGray)),
    ];
    if ticket.is_open() {
        spans.push(Span::raw("  "));
        if map.is_ready(ticket) {
            spans.push(Span::styled("ready", Style::new().fg(Color::Yellow)));
        } else {
            spans.push(Span::styled("waiting", Style::new().fg(Color::DarkGray)));
        }
    }
    if let Some(assignee) = ticket
        .assignee
        .as_deref()
        .and_then(|assignee| assignee.split_whitespace().next())
    {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            assignee.to_string(),
            Style::new().fg(Color::DarkGray),
        ));
    }
    Line::from(spans)
}

fn wrapped_height(text: &Text<'_>, width: u16) -> usize {
    let width = usize::from(width);
    if width == 0 {
        return text.lines.len();
    }
    text.lines
        .iter()
        .map(|line| line.width().max(1).div_ceil(width))
        .sum()
}

/// Splits the area into list and body. The list takes its share of the width within bounds, and
/// takes all of it when what is left cannot hold a readable body.
fn panes(area: Rect) -> (Rect, Rect) {
    let share = (u32::from(area.width) * LIST_SHARE / 100) as u16;
    let list = share.clamp(LIST_MIN, LIST_MAX);
    if area.width < list.saturating_add(BODY_MIN) {
        return (area, Rect::new(area.right(), area.y, 0, area.height));
    }
    let [list, body] =
        Layout::horizontal([Constraint::Length(list), Constraint::Fill(1)]).areas(area);
    (list, body)
}

#[derive(Default)]
pub struct Outline {
    list: ListState,
    focus: Focus,
    scroll: u16,
    list_page: u16,
    body_page: u16,
    shown: Option<String>,
}

impl Outline {
    fn border(&self, focus: Focus) -> Style {
        if self.focus == focus {
            Style::new().fg(Color::Cyan)
        } else {
            Style::new().fg(Color::DarkGray)
        }
    }

    fn draw_list(&mut self, frame: &mut Frame, area: Rect, map: &Map, rows: &[Row]) {
        let width = usize::from(area.width.saturating_sub(4));
        let block = Block::bordered()
            .border_style(self.border(Focus::List))
            .title(fit(&map.title, width))
            .title_bottom(fit(&summary(map), width));
        let inner = block.inner(area);
        self.list_page = inner.height.max(1);
        let items: Vec<ListItem> = rows
            .iter()
            .map(|row| ListItem::new(row_line(map, row, usize::from(inner.width))))
            .collect();
        let highlight = if self.focus == Focus::List {
            Style::new().bg(Color::Cyan).fg(Color::Black)
        } else {
            Style::new().bg(Color::DarkGray)
        };
        frame.render_stateful_widget(
            List::new(items).block(block).highlight_style(highlight),
            area,
            &mut self.list,
        );
    }

    fn draw_body(&mut self, frame: &mut Frame, area: Rect, map: &Map, ticket: Option<&Ticket>) {
        if area.width < 4 || area.height < 3 {
            return;
        }
        let style = self.border(Focus::Body);
        let Some(ticket) = ticket else {
            frame.render_widget(Block::bordered().border_style(style), area);
            return;
        };

        let inner = Block::bordered().inner(area);
        let (head, rest) = if inner.height > HEADER + 1 {
            let [head, rest] =
                Layout::vertical([Constraint::Length(HEADER), Constraint::Fill(1)]).areas(inner);
            (Some(head), rest)
        } else {
            (None, inner)
        };

        let prose = Rect {
            width: rest.width.min(PROSE_MAX),
            ..rest
        };
        let source = ticket.source.join("\n");
        let text = tui_markdown::from_str(&source);
        let height = wrapped_height(&text, prose.width);
        let span = usize::from(rest.height);
        let last = u16::try_from(height.saturating_sub(span)).unwrap_or(u16::MAX);
        self.scroll = self.scroll.min(last);
        self.body_page = rest.height.max(1);

        let width = usize::from(area.width.saturating_sub(4));
        let reach = (usize::from(self.scroll) + span).min(height);
        frame.render_widget(
            Block::bordered()
                .border_style(style)
                .title(fit(&ticket.id, width))
                .title_bottom(fit(&format!("{reach}/{height}"), width)),
            area,
        );

        if let Some(head) = head {
            let lines = vec![
                Line::styled(
                    ticket.label.clone(),
                    Style::new().add_modifier(Modifier::BOLD),
                ),
                facts(map, ticket),
                relations(map, ticket),
            ];
            frame.render_widget(Paragraph::new(lines), head);
        }

        frame.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll, 0)),
            prose,
        );

        if last > 0 {
            let mut state =
                ScrollbarState::new(usize::from(last)).position(usize::from(self.scroll));
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .thumb_style(style),
                Rect::new(area.right().saturating_sub(1), rest.y, 1, rest.height),
                &mut state,
            );
        }
    }

    fn scroll_by(&mut self, delta: i32) {
        let next = i32::from(self.scroll) + delta;
        self.scroll = u16::try_from(next.max(0)).unwrap_or(u16::MAX);
    }
}

/// Moves the selection along the outline order, which is not the Map order.
fn step(map: &Map, selection: &mut Selection, delta: isize) {
    let rows = outline(map);
    if rows.is_empty() {
        return;
    }
    let current = selection
        .ticket_id()
        .and_then(|id| rows.iter().position(|row| row.id == id))
        .unwrap_or(0) as isize;
    let next = (current + delta).rem_euclid(rows.len() as isize) as usize;
    selection.select(&rows[next].id);
}

fn jump(map: &Map, selection: &mut Selection, last: bool) {
    let rows = outline(map);
    let row = if last { rows.last() } else { rows.first() };
    if let Some(row) = row {
        selection.select(&row.id);
    }
}

impl Variant for Outline {
    fn name(&self) -> &'static str {
        "Outline"
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, map: &Map, selection: &Selection) {
        if self.shown.as_deref() != selection.ticket_id() {
            self.shown = selection.ticket_id().map(str::to_string);
            self.scroll = 0;
        }

        let rows = outline(map);
        self.list
            .select(rows.iter().position(|row| selection.is(&row.id)));

        let (list, body) = panes(area);
        self.draw_list(frame, list, map, &rows);
        self.draw_body(
            frame,
            body,
            map,
            selection.ticket_id().and_then(|id| map.ticket(id)),
        );
    }

    fn handle_key(&mut self, key: KeyEvent, map: &Map, selection: &mut Selection) {
        let body = self.focus == Focus::Body;
        let body_page = i32::from(self.body_page.max(1));
        let list_page = self.list_page.max(1) as isize;
        match key.code {
            KeyCode::Char('h') | KeyCode::Left | KeyCode::Esc => self.focus = Focus::List,
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => self.focus = Focus::Body,
            KeyCode::Char('j') | KeyCode::Down if body => self.scroll_by(1),
            KeyCode::Char('k') | KeyCode::Up if body => self.scroll_by(-1),
            KeyCode::Char('j') | KeyCode::Down => step(map, selection, 1),
            KeyCode::Char('k') | KeyCode::Up => step(map, selection, -1),
            KeyCode::PageDown if body => self.scroll_by(body_page),
            KeyCode::PageUp if body => self.scroll_by(-body_page),
            KeyCode::PageDown => step(map, selection, list_page),
            KeyCode::PageUp => step(map, selection, -list_page),
            KeyCode::Char('g') if body => self.scroll = 0,
            KeyCode::Char('G') if body => self.scroll = u16::MAX,
            KeyCode::Char('g') => jump(map, selection, false),
            KeyCode::Char('G') => jump(map, selection, true),
            _ => {}
        }
    }
}
