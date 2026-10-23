use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use renderdag::{ConnectionKind, GraphLayout, GraphNode, LaneId, RenderConfig, RowPlan, TrackCell};
use tui_popup::{KnownSizeWrapper, Popup};

use super::map::{Map, Ticket, TicketKind, TicketStatus};
use super::{Selection, Variant};

const LANE_COLORS: [Color; 6] = [
    Color::Cyan,
    Color::Magenta,
    Color::Green,
    Color::Blue,
    Color::LightRed,
    Color::LightCyan,
];

const SELECTED_BG: Color = Color::Indexed(237);
const TAG_WIDTH: usize = 11;

#[derive(Default)]
pub struct Rail {
    top: usize,
    detail: bool,
    detail_scroll: u16,
    ascii: bool,
}

impl Variant for Rail {
    fn name(&self) -> &'static str {
        "Rail"
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, map: &Map, selection: &Selection) {
        let [head, body] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
        frame.render_widget(header(map, area.width as usize), head);

        let order = rail_order(map);
        let nodes = rail_nodes(map, &order);
        let mut layout = GraphLayout::new();
        let plans = layout.layout(&nodes);

        if order.is_empty() {
            frame.render_widget(
                Paragraph::new(Line::styled(
                    "  no Tickets under this Map",
                    Style::new().fg(Color::DarkGray),
                )),
                body,
            );
            return;
        }

        let cursor = order
            .iter()
            .position(|&index| selection.is(&map.tickets[index].id))
            .unwrap_or(0);
        let visible = body.height as usize;
        self.top = scroll_top(self.top, cursor, order.len(), visible);

        let width = body.width as usize;
        let widest = plans
            .iter()
            .map(|plan| plan.width)
            .max()
            .unwrap_or(1)
            .max(1);
        let gutter = widest.min((width / 2).max(1));
        let painter = Painter {
            ascii: self.ascii,
            config: glyph_config(self.ascii),
            gutter,
            label: label_width(map, width, gutter),
            width,
            related: plans.get(cursor).map(related_lanes).unwrap_or_default(),
        };

        let lines: Vec<Line> = order
            .iter()
            .zip(&plans)
            .enumerate()
            .skip(self.top)
            .take(visible)
            .map(|(row, (&index, plan))| {
                let ticket = &map.tickets[index];
                painter.row(ticket, plan, map.is_ready(ticket), row == cursor)
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), body);

        if self.detail {
            self.popup(frame, area, map, &map.tickets[order[cursor]]);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, map: &Map, selection: &mut Selection) {
        if self.detail {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                    self.detail = false;
                    self.detail_scroll = 0;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.detail_scroll = self.detail_scroll.saturating_add(1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                }
                _ => {}
            }
            return;
        }

        let order = rail_order(map);
        if order.is_empty() {
            return;
        }
        let cursor = order
            .iter()
            .position(|&index| selection.is(&map.tickets[index].id))
            .unwrap_or(0);

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => step(map, &order, selection, cursor, 1),
            KeyCode::Up | KeyCode::Char('k') => step(map, &order, selection, cursor, -1),
            KeyCode::Home | KeyCode::Char('g') => select(map, &order, selection, 0),
            KeyCode::End | KeyCode::Char('G') => select(map, &order, selection, order.len() - 1),
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(blocker) = map.tickets[order[cursor]].blocked_by.first() {
                    selection.select(blocker);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let ticket = &map.tickets[order[cursor]];
                if let Some(blocked) = map.blocks(&ticket.id).first() {
                    selection.select(&blocked.id);
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.detail = true,
            KeyCode::Char('a') => self.ascii = !self.ascii,
            _ => {}
        }
    }
}

impl Rail {
    fn popup(&mut self, frame: &mut Frame, area: Rect, map: &Map, ticket: &Ticket) {
        if area.width < 28 || area.height < 10 {
            return;
        }
        let width = (area.width as usize).saturating_sub(8).min(86);
        let height = (area.height as usize).saturating_sub(4).min(26);
        let text = detail(map, ticket);
        let wrapped: usize = text
            .iter()
            .map(|line| line.width().div_ceil(width).max(1))
            .sum();
        self.detail_scroll = self
            .detail_scroll
            .min(wrapped.saturating_sub(height) as u16);

        let inner = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((self.detail_scroll, 0));
        let popup = Popup::new(KnownSizeWrapper {
            inner,
            width,
            height,
        })
        .title(format!(" {}  j/k scroll, Esc close ", ticket.id))
        .border_style(Style::new().fg(status_color(ticket.status)))
        .style(Style::new().bg(Color::Black));
        frame.render_widget(popup, area);
    }
}

/// Ticket order the rail pins its rows to: every blocker above every Ticket it blocks, otherwise
/// the Map's own order.
fn rail_order(map: &Map) -> Vec<usize> {
    let count = map.tickets.len();
    let mut waiting: Vec<usize> = map
        .tickets
        .iter()
        .map(|ticket| {
            ticket
                .blocked_by
                .iter()
                .filter(|id| map.index_of(id).is_some())
                .count()
        })
        .collect();
    let mut placed = vec![false; count];
    let mut order = Vec::with_capacity(count);

    while order.len() < count {
        let free = (0..count).find(|&index| !placed[index] && waiting[index] == 0);
        let Some(next) = free.or_else(|| (0..count).find(|&index| !placed[index])) else {
            break;
        };
        placed[next] = true;
        order.push(next);
        for (index, ticket) in map.tickets.iter().enumerate() {
            if !placed[index]
                && ticket
                    .blocked_by
                    .iter()
                    .any(|id| map.index_of(id) == Some(next))
            {
                waiting[index] = waiting[index].saturating_sub(1);
            }
        }
    }
    order
}

struct RailNode {
    id: String,
    blocks: Vec<String>,
}

/// renderdag draws a node's parents below it, so a Ticket's parents are the Tickets it blocks.
impl GraphNode for RailNode {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn parents(&self) -> &[Self::Id] {
        &self.blocks
    }
}

fn rail_nodes(map: &Map, order: &[usize]) -> Vec<RailNode> {
    order
        .iter()
        .enumerate()
        .map(|(row, &index)| {
            let ticket = &map.tickets[index];
            RailNode {
                id: ticket.id.clone(),
                blocks: order
                    .iter()
                    .skip(row + 1)
                    .map(|&later| &map.tickets[later])
                    .filter(|blocked| blocked.blocked_by.contains(&ticket.id))
                    .map(|blocked| blocked.id.clone())
                    .collect(),
            }
        })
        .collect()
}

/// Lanes the selected row forks into, merges from or sits in.
fn related_lanes(plan: &RowPlan<'_, RailNode>) -> Vec<LaneId> {
    plan.operations
        .iter()
        .filter(|op| match op.cell {
            TrackCell::Node(_) => true,
            TrackCell::Connection(kind) => !matches!(
                kind,
                ConnectionKind::Empty | ConnectionKind::Vertical | ConnectionKind::CrossOver
            ),
        })
        .filter_map(|op| op.lane_id)
        .collect()
}

struct Painter {
    ascii: bool,
    config: RenderConfig,
    gutter: usize,
    label: usize,
    width: usize,
    related: Vec<LaneId>,
}

impl Painter {
    fn row(
        &self,
        ticket: &Ticket,
        plan: &RowPlan<'_, RailNode>,
        ready: bool,
        selected: bool,
    ) -> Line<'static> {
        let mut spans = self.lanes(plan, ticket, selected);
        let mut left = self.width.saturating_sub(self.gutter + 1);

        let emphasis = if selected {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };
        push(
            &mut spans,
            &mut left,
            pad(&ticket.label, self.label),
            Style::new()
                .fg(status_fg(ticket.status))
                .add_modifier(emphasis),
        );
        push(
            &mut spans,
            &mut left,
            format!(" {:<3} ", kind_tag(ticket.kind)),
            Style::new().fg(Color::Blue),
        );
        let (state, style) = state_tag(ticket, ready);
        push(&mut spans, &mut left, format!("{state:<5} "), style);
        let gist = truncate(ticket.gist(), left);
        push(
            &mut spans,
            &mut left,
            gist,
            Style::new().fg(Color::Gray).add_modifier(Modifier::DIM),
        );
        spans.push(Span::raw(" ".repeat(left)));

        let line = Line::from(spans);
        if selected {
            line.style(Style::new().bg(SELECTED_BG))
        } else {
            line
        }
    }

    fn lanes(
        &self,
        plan: &RowPlan<'_, RailNode>,
        ticket: &Ticket,
        selected: bool,
    ) -> Vec<Span<'static>> {
        let cut = plan.width > self.gutter;
        let mut spans: Vec<Span<'static>> = plan
            .operations
            .iter()
            .take_while(|op| op.x < self.gutter)
            .map(|op| match op.cell {
                TrackCell::Node(_) => Span::styled(
                    node_glyph(ticket.status, self.ascii).to_string(),
                    Style::new()
                        .fg(status_color(ticket.status))
                        .add_modifier(Modifier::BOLD),
                ),
                TrackCell::Connection(kind) => Span::styled(
                    self.config.glyph_for_connection(kind).to_string(),
                    self.lane_style(op.lane_id, selected),
                ),
            })
            .collect();

        if cut && let Some(last) = spans.last_mut() {
            *last = if plan.node_lane_col * 2 >= self.gutter {
                Span::styled(
                    node_glyph(ticket.status, self.ascii).to_string(),
                    Style::new().fg(status_color(ticket.status)),
                )
            } else {
                Span::styled("…", Style::new().fg(Color::DarkGray))
            };
        }

        spans.push(Span::raw(
            " ".repeat(self.gutter.saturating_sub(plan.width) + 1),
        ));
        spans
    }

    fn lane_style(&self, lane: Option<LaneId>, selected: bool) -> Style {
        let Some(lane) = lane else {
            return Style::new().fg(Color::DarkGray);
        };
        let style = Style::new().fg(LANE_COLORS[lane as usize % LANE_COLORS.len()]);
        if selected || self.related.contains(&lane) {
            style
        } else {
            style.add_modifier(Modifier::DIM)
        }
    }
}

fn header(map: &Map, width: usize) -> Paragraph<'static> {
    let links = map.edges().len();
    let counted = if links == 0 {
        format!("{} Tickets, no links", map.tickets.len())
    } else {
        format!("{} Tickets, {links} links", map.tickets.len())
    };
    let destination = map
        .destination()
        .map(|section| section.gist())
        .unwrap_or("");
    let title = truncate(
        &map.title,
        width.saturating_sub(counted.chars().count() + 4),
    );

    Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!(" {title}  "),
                Style::new().add_modifier(Modifier::BOLD),
            ),
            Span::styled(counted, Style::new().fg(Color::DarkGray)),
        ]),
        Line::styled(
            format!(" {}", truncate(destination, width.saturating_sub(2))),
            Style::new().fg(Color::Gray).add_modifier(Modifier::ITALIC),
        ),
    ])
}

fn detail(map: &Map, ticket: &Ticket) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::styled(
            ticket.label.clone(),
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Line::styled(
            format!(
                "{}, {}, {}",
                ticket.kind.label(),
                ticket.status.label(),
                ticket.assignee.as_deref().unwrap_or("unassigned")
            ),
            Style::new().fg(Color::DarkGray),
        ),
        Line::styled(
            format!("blocked by {}", joined(&ticket.blocked_by)),
            Style::new().fg(Color::Yellow),
        ),
        Line::styled(
            format!(
                "blocks {}",
                joined(
                    &map.blocks(&ticket.id)
                        .iter()
                        .map(|blocked| blocked.id.clone())
                        .collect::<Vec<_>>()
                )
            ),
            Style::new().fg(Color::Magenta),
        ),
        Line::raw(""),
    ];

    for section in &ticket.sections {
        lines.push(Line::styled(
            section.heading.clone(),
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        lines.extend(section.body.iter().cloned().map(Line::raw));
        lines.push(Line::raw(""));
    }
    lines
}

fn joined(ids: &[String]) -> String {
    if ids.is_empty() {
        "nothing".to_string()
    } else {
        ids.join(", ")
    }
}

fn select(map: &Map, order: &[usize], selection: &mut Selection, row: usize) {
    if let Some(&index) = order.get(row) {
        selection.select(&map.tickets[index].id);
    }
}

fn step(map: &Map, order: &[usize], selection: &mut Selection, cursor: usize, delta: isize) {
    let count = order.len() as isize;
    let row = (cursor as isize + delta).rem_euclid(count) as usize;
    select(map, order, selection, row);
}

fn scroll_top(top: usize, cursor: usize, count: usize, visible: usize) -> usize {
    if visible == 0 || count <= visible {
        return 0;
    }
    let mut top = top.min(count - visible);
    if cursor < top {
        top = cursor;
    }
    if cursor >= top + visible {
        top = cursor + 1 - visible;
    }
    top
}

fn label_width(map: &Map, width: usize, gutter: usize) -> usize {
    let longest = map
        .tickets
        .iter()
        .map(|ticket| ticket.label.chars().count())
        .max()
        .unwrap_or(0);
    let room = width.saturating_sub(gutter + 1 + TAG_WIDTH);
    longest.min(room).max(1)
}

fn glyph_config(ascii: bool) -> RenderConfig {
    let mut config = RenderConfig::default();
    if !ascii {
        return config;
    }
    for (kind, glyph) in [
        (ConnectionKind::Horizontal, '-'),
        (ConnectionKind::Vertical, '|'),
        (ConnectionKind::CornerUpLeft, '+'),
        (ConnectionKind::CornerUpRight, '+'),
        (ConnectionKind::CornerDownRight, '+'),
        (ConnectionKind::CornerDownLeft, '+'),
        (ConnectionKind::TeeUp, '+'),
        (ConnectionKind::TeeDown, '+'),
        (ConnectionKind::TeeLeft, '+'),
        (ConnectionKind::TeeRight, '+'),
        (ConnectionKind::EndLeft, '-'),
        (ConnectionKind::EndRight, '-'),
        (ConnectionKind::EndUp, '|'),
        (ConnectionKind::EndDown, '|'),
        (ConnectionKind::CrossOver, ':'),
    ] {
        config.set_connection_glyph(kind, glyph);
    }
    config
}

fn node_glyph(status: TicketStatus, ascii: bool) -> char {
    match (status, ascii) {
        (TicketStatus::Open, false) => '●',
        (TicketStatus::Closed, false) => '○',
        (TicketStatus::Unknown, false) => '◌',
        (TicketStatus::Open, true) => '*',
        (TicketStatus::Closed, true) => 'o',
        (TicketStatus::Unknown, true) => '?',
    }
}

fn status_color(status: TicketStatus) -> Color {
    match status {
        TicketStatus::Open => Color::LightYellow,
        TicketStatus::Closed => Color::Green,
        TicketStatus::Unknown => Color::DarkGray,
    }
}

fn status_fg(status: TicketStatus) -> Color {
    match status {
        TicketStatus::Open => Color::White,
        TicketStatus::Closed => Color::Gray,
        TicketStatus::Unknown => Color::DarkGray,
    }
}

fn kind_tag(kind: TicketKind) -> &'static str {
    match kind {
        TicketKind::Grilling => "grl",
        TicketKind::Research => "res",
        TicketKind::Task => "tsk",
        TicketKind::Prototype => "pro",
        TicketKind::Unknown => "-",
    }
}

fn state_tag(ticket: &Ticket, ready: bool) -> (&'static str, Style) {
    match (ticket.status, ready) {
        (TicketStatus::Unknown, _) => ("-", Style::new().fg(Color::DarkGray)),
        (TicketStatus::Closed, _) => ("done", Style::new().fg(Color::Green)),
        (TicketStatus::Open, true) => (
            "ready",
            Style::new()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        ),
        (TicketStatus::Open, false) => ("wait", Style::new().fg(Color::DarkGray)),
    }
}

fn push(spans: &mut Vec<Span<'static>>, left: &mut usize, text: String, style: Style) {
    *left = left.saturating_sub(text.chars().count());
    spans.push(Span::styled(text, style));
}

fn pad(text: &str, width: usize) -> String {
    let mut text = truncate(text, width);
    let short = width.saturating_sub(text.chars().count());
    text.extend(std::iter::repeat_n(' ', short));
    text
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    match width {
        0 => String::new(),
        _ => text.chars().take(width - 1).chain(['…']).collect(),
    }
}
