//! The Map as a dependency graph: Tickets placed in two dimensions, edges drawn between them.

use std::cmp::Reverse;

use rataflow::{
    Background, BackgroundVariant, Controls, ControlsPosition, Direction, Edge, EdgeStyle,
    FitViewOptions, Flow, MiniMap, MiniMapPosition, Node, NodeContent, NodeRenderContext, Palette,
    Position, StepEdge, Sugiyama, Theme,
};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Paragraph, Widget, Wrap};
use tui_popup::Popup;

use super::map::{Map, Ticket, TicketKind, TicketStatus};
use super::{Selection, Variant};

const NODE_WIDTH: f64 = 28.0;
const NODE_HEIGHT: f64 = 4.0;
const COLUMN_GAP: f64 = 4.0;
const RANK_GAP: f64 = 2.0;
const MIN_ZOOM: f64 = 0.2;
const READABLE_ZOOM: f64 = 0.75;
const PAN_STEP: f64 = 8.0;
const HINTS: &str = " hjkl move   HJKL pan   +/- zoom   f fit   0 reset   c focus   n/p order   d detail   m chrome";

type TicketFlow = Flow<TicketNode, StepEdge>;

pub struct Canvas {
    flow: Option<TicketFlow>,
    map_id: String,
    revealed: Option<String>,
    detail: bool,
    chrome: bool,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            flow: None,
            map_id: String::new(),
            revealed: None,
            detail: false,
            chrome: true,
        }
    }
}

impl Variant for Canvas {
    fn name(&self) -> &'static str {
        "Canvas"
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, map: &Map, selection: &Selection) {
        let [header, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        draw_header(frame, header, map);

        let [surface, hints] = if body.height >= 12 {
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(body)
        } else {
            [body, Rect::ZERO]
        };
        frame.render_widget(
            Paragraph::new(HINTS).style(Style::new().fg(Color::DarkGray)),
            hints,
        );

        if self.map_id != map.id || self.flow.is_none() {
            self.flow = Some(build(map));
            self.map_id = map.id.clone();
            self.revealed = None;
        }
        let Some(flow) = self.flow.as_mut() else {
            return;
        };

        select(flow, selection);
        if self.revealed.as_deref() != selection.ticket_id()
            && let Some(id) = selection.ticket_id()
        {
            match self.revealed {
                Some(_) => reveal(flow, id, surface),
                None => open(flow, id, surface),
            }
            self.revealed = Some(id.to_string());
        }

        clamp(flow, surface);
        frame.render_widget(
            Background::new(&*flow)
                .variant(BackgroundVariant::Dots)
                .gap(grid(flow.viewport.zoom, 6.0), grid(flow.viewport.zoom, 3.0)),
            surface,
        );
        frame.render_widget(&mut *flow, surface);

        if self.chrome && surface.width >= 64 && surface.height >= 14 {
            frame.render_widget(
                Controls::new(&*flow)
                    .position(ControlsPosition::BottomLeft)
                    .show_lock(false),
                surface,
            );
            frame.render_widget(
                MiniMap::new(&*flow)
                    .position(MiniMapPosition::BottomRight)
                    .size(22, 6),
                surface,
            );
        }

        if self.detail
            && surface.width >= 34
            && surface.height >= 10
            && let Some(ticket) = selection.ticket_id().and_then(|id| map.ticket(id))
        {
            frame.render_widget(detail(map, ticket, surface), surface);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, map: &Map, selection: &mut Selection) {
        if key.code == KeyCode::Esc {
            self.detail = false;
            return;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let Some(flow) = self.flow.as_mut() else {
            return;
        };
        match key.code {
            KeyCode::Left if shift => flow.pan(PAN_STEP, 0.0),
            KeyCode::Right if shift => flow.pan(-PAN_STEP, 0.0),
            KeyCode::Up if shift => flow.pan(0.0, PAN_STEP / 2.0),
            KeyCode::Down if shift => flow.pan(0.0, -PAN_STEP / 2.0),
            KeyCode::Char('H') => flow.pan(PAN_STEP, 0.0),
            KeyCode::Char('L') => flow.pan(-PAN_STEP, 0.0),
            KeyCode::Char('K') => flow.pan(0.0, PAN_STEP / 2.0),
            KeyCode::Char('J') => flow.pan(0.0, -PAN_STEP / 2.0),
            KeyCode::Char('h') | KeyCode::Left => step(flow, selection, Direction::Left),
            KeyCode::Char('l') | KeyCode::Right => step(flow, selection, Direction::Right),
            KeyCode::Char('k') | KeyCode::Up => step(flow, selection, Direction::Up),
            KeyCode::Char('j') | KeyCode::Down => step(flow, selection, Direction::Down),
            KeyCode::Char('n') => selection.step(map, 1),
            KeyCode::Char('p') => selection.step(map, -1),
            KeyCode::Char('+') | KeyCode::Char('=') => flow.zoom_in(),
            KeyCode::Char('-') | KeyCode::Char('_') => flow.zoom_out(),
            KeyCode::Char('0') => flow.reset_zoom(),
            KeyCode::Char('f') => flow.request_fit_view_with_options(
                FitViewOptions::default()
                    .with_padding(1.0)
                    .with_min_zoom(MIN_ZOOM),
            ),
            KeyCode::Char('c') => {
                flow.center_on_selected();
                self.revealed = selection.ticket_id().map(str::to_string);
            }
            KeyCode::Char('m') => self.chrome = !self.chrome,
            KeyCode::Char('d') | KeyCode::Char(' ') | KeyCode::Enter => self.detail = !self.detail,
            _ => {}
        }
    }
}

/// rataflow scales the background gap by the zoom, so a fixed gap collapses into a solid
/// field of dots once zoomed out. Dividing it back keeps the grid steady on screen.
fn grid(zoom: f64, cells: f64) -> u16 {
    (cells / zoom.max(0.05)).round().clamp(1.0, 64.0) as u16
}

fn draw_header(frame: &mut Frame, area: Rect, map: &Map) {
    let title = Line::from(vec![
        Span::styled(
            map.title.clone(),
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  {}",
                map.destination().map_or("", |section| section.gist())
            ),
            Style::new().fg(Color::DarkGray),
        ),
    ]);

    let legend = Line::from(vec![
        Span::styled("ready ", Style::new().fg(State::Ready.color())),
        Span::styled("blocked ", Style::new().fg(State::Blocked.color())),
        Span::styled("closed ", Style::new().fg(State::Closed.color())),
        Span::styled("unknown", Style::new().fg(State::Unknown.color())),
    ])
    .right_aligned();

    if area.width >= 92 {
        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(30)]).areas(area);
        frame.render_widget(Paragraph::new(title), left);
        frame.render_widget(Paragraph::new(legend), right);
    } else {
        frame.render_widget(Paragraph::new(title), area);
    }
}

fn build(map: &Map) -> TicketFlow {
    let tags = tags(map);
    let nodes: Vec<Node<TicketNode>> = map
        .tickets
        .iter()
        .zip(tags)
        .map(|(ticket, tag)| {
            Node::new(
                ticket.id.clone(),
                (0.0, 0.0),
                (NODE_WIDTH, NODE_HEIGHT),
                TicketNode::new(map, tag, ticket),
            )
            .with_draggable(false)
        })
        .collect();

    let mut edges: Vec<Edge<StepEdge>> = Vec::new();
    for (blocker, blocked) in map.edges() {
        if blocker == blocked {
            continue;
        }
        let (from, to) = (&map.tickets[blocker], &map.tickets[blocked]);
        let id = format!("{}|{}", from.id, to.id);
        if edges.iter().any(|edge| edge.id == id) {
            continue;
        }
        edges.push(Edge::new(id, from.id.clone(), to.id.clone()).with_content(dependency(from)));
    }

    let mut flow = Flow::with_graph(nodes, edges)
        .unwrap_or_default()
        .with_min_zoom(MIN_ZOOM)
        .with_theme(theme());
    flow.apply_layout(
        Sugiyama::vertical()
            .with_node_spacing(COLUMN_GAP)
            .with_rank_spacing(RANK_GAP)
            .with_margin(0.0),
    );
    pack(&mut flow, map);
    for ticket in &map.tickets {
        flow.set_handles_hidden(&ticket.id, true);
    }
    flow
}

/// A blocker still open is a live edge; once it closes the dependency is only history.
fn dependency(blocker: &Ticket) -> StepEdge {
    let resting = if blocker.is_open() {
        EdgeStyle::default().with_stroke_style(Style::new().fg(Color::Indexed(244)))
    } else {
        EdgeStyle::dotted().with_stroke_style(Style::new().fg(Color::Indexed(238)))
    };
    StepEdge::default().with_style(resting).with_selected_style(
        EdgeStyle::default()
            .with_stroke_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    )
}

/// The rataflow dark palette over the terminal's own background, so the drawing sits on
/// whatever the user already runs.
fn theme() -> Theme {
    let mut palette = Palette::DARK;
    palette.canvas_bg = Color::Reset;
    palette.surface = Color::Reset;
    Theme::Custom(palette)
}

/// Sugiyama lays out each connected component in its own coordinate space and never
/// places a Ticket that carries no edge, so both land on the origin. This spreads the
/// components across rows and gives the edgeless ones somewhere to be.
fn pack(flow: &mut TicketFlow, map: &Map) {
    let mut groups = components(map);
    groups.sort_by_key(|group| (Reverse(group.len()), group[0]));

    let clusters: Vec<Cluster> = groups
        .iter()
        .map(|group| cluster(flow, map, group))
        .collect();
    let limit = clusters
        .iter()
        .map(|cluster| cluster.width)
        .fold(4.0 * (NODE_WIDTH + COLUMN_GAP), f64::max);

    let mut placements: Vec<(String, Position)> = Vec::new();
    let (mut x, mut y, mut tallest) = (0.0_f64, 0.0_f64, 0.0_f64);
    for cluster in clusters {
        if x > 0.0 && x + cluster.width > limit {
            x = 0.0;
            y += tallest + RANK_GAP;
            tallest = 0.0;
        }
        for (id, position) in cluster.nodes {
            placements.push((id, Position::new(position.x + x, position.y + y)));
        }
        x += cluster.width + COLUMN_GAP * 2.0;
        tallest = tallest.max(cluster.height);
    }
    flow.set_node_positions(placements);
}

struct Cluster {
    nodes: Vec<(String, Position)>,
    width: f64,
    height: f64,
}

fn cluster(flow: &TicketFlow, map: &Map, group: &[usize]) -> Cluster {
    let placed: Vec<(String, Position)> = group
        .iter()
        .filter_map(|index| map.tickets.get(*index))
        .filter_map(|ticket| {
            flow.node(&ticket.id)
                .map(|node| (ticket.id.clone(), node.position))
        })
        .collect();

    if placed.is_empty() {
        return Cluster {
            nodes: placed,
            width: 0.0,
            height: 0.0,
        };
    }

    let left = placed.iter().map(|(_, at)| at.x).fold(f64::MAX, f64::min);
    let top = placed.iter().map(|(_, at)| at.y).fold(f64::MAX, f64::min);
    let right = placed.iter().map(|(_, at)| at.x).fold(f64::MIN, f64::max);
    let bottom = placed.iter().map(|(_, at)| at.y).fold(f64::MIN, f64::max);

    Cluster {
        nodes: placed
            .into_iter()
            .map(|(id, at)| (id, Position::new(at.x - left, at.y - top)))
            .collect(),
        width: right - left + NODE_WIDTH,
        height: bottom - top + NODE_HEIGHT,
    }
}

/// Ticket indices grouped by weakly connected component, each group in Map order.
fn components(map: &Map) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..map.tickets.len()).collect();
    for (blocker, blocked) in map.edges() {
        let (a, b) = (root(&mut parent, blocker), root(&mut parent, blocked));
        parent[a] = b;
    }

    let mut group_of: Vec<Option<usize>> = vec![None; map.tickets.len()];
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for index in 0..map.tickets.len() {
        let owner = root(&mut parent, index);
        match group_of[owner] {
            Some(group) => groups[group].push(index),
            None => {
                group_of[owner] = Some(groups.len());
                groups.push(vec![index]);
            }
        }
    }
    groups
}

fn root(parent: &mut [usize], mut index: usize) -> usize {
    while parent[index] != index {
        parent[index] = parent[parent[index]];
        index = parent[index];
    }
    index
}

/// Mirrors the switcher's selection onto the graph, lighting the Ticket and every
/// dependency that touches it.
fn select(flow: &mut TicketFlow, selection: &Selection) {
    let Some(id) = selection.ticket_id() else {
        flow.clear_selection();
        return;
    };
    flow.select_node(id);
    let touching: Vec<String> = flow
        .edges()
        .iter()
        .filter(|edge| edge.source == id || edge.target == id)
        .map(|edge| edge.id.clone())
        .collect();
    for edge in touching {
        flow.toggle_edge_selection(&edge);
    }
}

fn step(flow: &mut TicketFlow, selection: &mut Selection, direction: Direction) {
    if let Some(id) = selection.ticket_id() {
        flow.select_node(id);
    }
    flow.select_node_in_direction(direction);
    if let Some(id) = flow.first_selected_node_id() {
        selection.select(&id);
    }
}

/// First frame on a Map: as much of the graph as still carries a label, centred on the
/// selected Ticket.
fn open(flow: &mut TicketFlow, id: &str, area: Rect) {
    let (width, height) = extent(flow);
    flow.viewport.zoom = (f64::from(area.width) / width)
        .min(f64::from(area.height) / height)
        .clamp(READABLE_ZOOM, 1.0);
    focus(flow, id, area);
}

/// Holds the drawing against the view, so panning stops at the edge of the graph instead
/// of wandering into empty canvas.
fn clamp(flow: &mut TicketFlow, area: Rect) {
    let (width, height) = extent(flow);
    let zoom = flow.viewport.zoom;
    flow.viewport.x = anchor(flow.viewport.x, width * zoom, f64::from(area.width));
    flow.viewport.y = anchor(flow.viewport.y, height * zoom, f64::from(area.height));
}

fn anchor(offset: f64, span: f64, limit: f64) -> f64 {
    const MARGIN: f64 = 2.0;
    if span + MARGIN * 2.0 <= limit {
        (limit - span) / 2.0
    } else {
        offset.clamp(limit - span - MARGIN, MARGIN)
    }
}

/// Width and height the laid out graph occupies, measured from the origin the packing
/// leaves it on.
fn extent(flow: &TicketFlow) -> (f64, f64) {
    let (mut width, mut height) = (1.0_f64, 1.0_f64);
    for node in flow.nodes() {
        width = width.max(node.position.x + node.width);
        height = height.max(node.position.y + node.height);
    }
    (width, height)
}

fn focus(flow: &mut TicketFlow, id: &str, area: Rect) {
    let Some(bounds) = flow.node_bounds(id) else {
        return;
    };
    let zoom = flow.viewport.zoom;
    let center = bounds.center();
    flow.viewport.x = f64::from(area.width) / 2.0 - center.x * zoom;
    flow.viewport.y = f64::from(area.height) / 2.0 - center.y * zoom;
}

fn reveal(flow: &mut TicketFlow, id: &str, area: Rect) {
    let Some(bounds) = flow.node_bounds(id) else {
        return;
    };
    let zoom = flow.viewport.zoom;
    let x = bounds.x() * zoom + flow.viewport.x;
    let y = bounds.y() * zoom + flow.viewport.y;
    flow.viewport.x += nudge(x, bounds.width() * zoom, f64::from(area.width));
    flow.viewport.y += nudge(y, bounds.height() * zoom, f64::from(area.height));
}

/// Shift that brings a span of `length` starting at `start` inside `0..limit`, keeping a
/// cell to spare. Zero when it already sits there, centred when it cannot fit.
fn nudge(start: f64, length: f64, limit: f64) -> f64 {
    if length + 2.0 >= limit {
        return (limit - length) / 2.0 - start;
    }
    if start < 1.0 {
        return 1.0 - start;
    }
    if start + length > limit - 1.0 {
        return limit - 1.0 - length - start;
    }
    0.0
}

fn detail(map: &Map, ticket: &Ticket, area: Rect) -> Popup<'static, Text<'static>> {
    let width = usize::from(area.width).saturating_sub(12).clamp(24, 62);
    let dim = Style::new().fg(Color::DarkGray);
    let state = State::of(map, ticket);

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("{} ", ticket.kind.label()), dim),
            Span::styled(state.label(), Style::new().fg(state.color())),
            Span::styled(
                ticket
                    .assignee
                    .as_deref()
                    .map_or(String::new(), |who| format!("  {who}")),
                dim,
            ),
        ]),
        Line::from(""),
    ];
    lines.extend(wrapped(ticket.gist(), width).into_iter().map(Line::from));

    let waiting: Vec<&Ticket> = ticket
        .blocked_by
        .iter()
        .filter_map(|id| map.ticket(id))
        .collect();
    lines.extend(list("blocked by", &waiting, width, dim));
    lines.extend(list("blocks", &map.blocks(&ticket.id), width, dim));

    Popup::new(Text::from(lines))
        .title(Line::from(format!(" {} ", ticket.label)))
        .border_style(Style::new().fg(Color::Cyan))
}

fn list(heading: &str, tickets: &[&Ticket], width: usize, dim: Style) -> Vec<Line<'static>> {
    if tickets.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(heading.to_string(), dim)),
    ];
    lines.extend(tickets.iter().map(|ticket| {
        Line::from(format!(
            "  {} {}",
            glyph(ticket.kind),
            truncated(&ticket.label, width.saturating_sub(4))
        ))
    }));
    lines
}

fn wrapped(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if line.is_empty() {
            line.push_str(word);
        } else if line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        } else {
            line.push(' ');
            line.push_str(word);
        }
        if lines.len() == 6 {
            break;
        }
    }
    if !line.is_empty() && lines.len() < 6 {
        lines.push(line);
    }
    lines
}

fn truncated(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    text.chars()
        .take(width.saturating_sub(1))
        .chain(['…'])
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Ready,
    Blocked,
    Closed,
    Unknown,
}

impl State {
    fn of(map: &Map, ticket: &Ticket) -> Self {
        match ticket.status {
            TicketStatus::Closed => Self::Closed,
            TicketStatus::Unknown => Self::Unknown,
            TicketStatus::Open if map.is_ready(ticket) => Self::Ready,
            TicketStatus::Open => Self::Blocked,
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Ready => Color::Green,
            Self::Blocked => Color::Yellow,
            Self::Closed => Color::DarkGray,
            Self::Unknown => Color::Blue,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Blocked => "blocked",
            Self::Closed => "closed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug)]
struct TicketNode {
    tag: String,
    kind: char,
    label: String,
    state: State,
}

impl TicketNode {
    fn new(map: &Map, tag: String, ticket: &Ticket) -> Self {
        Self {
            tag,
            kind: glyph(ticket.kind),
            label: ticket.label.clone(),
            state: State::of(map, ticket),
        }
    }
}

impl NodeContent for TicketNode {
    fn render(&self, ctx: &NodeRenderContext, buf: &mut Buffer) {
        let color = self.state.color();

        if ctx.area.width < 12 || ctx.area.height < 3 {
            let style =
                Style::new()
                    .fg(Color::Black)
                    .bg(if ctx.selected { Color::Cyan } else { color });
            buf.set_style(ctx.area, style);
            buf.set_stringn(
                ctx.area.x,
                ctx.area.y,
                format!("{} {} {}", self.tag, self.kind, self.label),
                usize::from(ctx.area.width),
                style,
            );
            return;
        }

        let border = if ctx.selected {
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(color)
        };
        let block = Block::bordered()
            .border_type(if ctx.selected {
                BorderType::Double
            } else {
                BorderType::Rounded
            })
            .border_style(border)
            .title(Span::styled(
                format!(" {} {} ", self.tag, self.kind),
                Style::new().fg(color),
            ))
            .title(
                Line::from(Span::styled(
                    format!(" {} ", self.state.label()),
                    Style::new().fg(color),
                ))
                .right_aligned(),
            );

        let inner = block.inner(ctx.area);
        block.render(ctx.area, buf);
        Paragraph::new(self.label.as_str())
            .wrap(Wrap { trim: true })
            .style(if self.state == State::Closed {
                Style::new().fg(Color::Gray).add_modifier(Modifier::DIM)
            } else {
                Style::new().fg(Color::White)
            })
            .render(inner, buf);
    }
}

/// The number each Ticket file carries, or Map order when those numbers do not name one
/// Ticket each.
fn tags(map: &Map) -> Vec<String> {
    let numbers: Vec<String> = map
        .tickets
        .iter()
        .map(|ticket| ticket.id.chars().take_while(char::is_ascii_digit).collect())
        .collect();

    let mut sorted = numbers.clone();
    sorted.sort();
    let clash = sorted.windows(2).any(|pair| pair[0] == pair[1]);

    if clash || numbers.iter().any(String::is_empty) {
        (1..=map.tickets.len()).map(|n| format!("{n:02}")).collect()
    } else {
        numbers
    }
}

fn glyph(kind: TicketKind) -> char {
    match kind {
        TicketKind::Grilling => 'G',
        TicketKind::Research => 'R',
        TicketKind::Task => 'T',
        TicketKind::Prototype => 'P',
        TicketKind::Unknown => '?',
    }
}
