//! Variant D: the Map as one scrolled document, with every Ticket link annotated in the prose.

use std::borrow::Cow;
use std::mem;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Position, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use tui_scrollview::{ScrollView, ScrollViewState};

use super::map::{LinePart, Map, Section, Ticket, TicketStatus, split_ticket_links};
use super::{Selection, Variant};

const MEASURE: u16 = 98;
const LEAD_ROWS: u16 = 3;
const SECTION_LINES: usize = 8;
const GIST_CHARS: usize = 110;

/// Sentinels standing in for a Ticket link while the markdown renderer runs over the source.
const LINK_OPEN: char = '\u{e000}';
const LINK_SEP: char = '\u{e001}';
const LINK_CLOSE: char = '\u{e002}';

const FRAME: Style = Style::new().fg(Color::DarkGray);

#[derive(PartialEq, Eq)]
struct Key {
    map: String,
    ticket: Option<String>,
    width: u16,
    expanded: bool,
}

/// The whole document, laid out once per [`Key`], with the row each Ticket is first read at.
struct Built {
    view: ScrollView,
    anchors: Vec<(u16, String)>,
}

#[derive(Default)]
pub struct Document {
    built: Option<Built>,
    key: Option<Key>,
    scroll: ScrollViewState,
    expanded: bool,
}

impl Variant for Document {
    fn name(&self) -> &'static str {
        "Document"
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, map: &Map, selection: &Selection) {
        let page = area.centered_horizontally(Constraint::Length(area.width.min(MEASURE)));
        if page.width < 8 || page.height == 0 {
            return;
        }

        let key = Key {
            map: map.id.clone(),
            ticket: selection.ticket_id().map(str::to_string),
            width: page.width,
            expanded: self.expanded,
        };

        if self.key.as_ref() != Some(&key) {
            let refocus = self.key.as_ref().is_none_or(|old| {
                old.map != key.map || old.ticket != key.ticket || old.expanded != key.expanded
            });
            let built = build(
                map,
                selection.ticket_id(),
                self.expanded,
                key.width.saturating_sub(1),
            );
            if refocus {
                let row = selection
                    .ticket_id()
                    .and_then(|id| anchor(&built.anchors, id))
                    .unwrap_or(0);
                self.scroll
                    .set_offset(Position::new(0, row.saturating_sub(LEAD_ROWS)));
            }
            self.built = Some(built);
            self.key = Some(key);
        }

        if let Some(built) = &self.built {
            frame.render_stateful_widget(&built.view, page, &mut self.scroll);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, map: &Map, selection: &mut Selection) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.scroll.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.scroll.scroll_up(),
            KeyCode::Char(' ') | KeyCode::PageDown => self.scroll.scroll_page_down(),
            KeyCode::PageUp => self.scroll.scroll_page_up(),
            KeyCode::Char('g') | KeyCode::Home => self.scroll.scroll_to_top(),
            KeyCode::Char('G') | KeyCode::End => self.scroll.scroll_to_bottom(),
            KeyCode::Char('n') => self.step_mention(map, selection, 1),
            KeyCode::Char('p') => self.step_mention(map, selection, -1),
            KeyCode::Char('e') | KeyCode::Enter => self.expanded = !self.expanded,
            KeyCode::Esc => self.expanded = false,
            _ => {}
        }
    }
}

impl Document {
    /// Moves the selection along the order the document mentions Tickets in, not the Map's.
    fn step_mention(&self, map: &Map, selection: &mut Selection, delta: isize) {
        let Some(built) = self
            .built
            .as_ref()
            .filter(|built| !built.anchors.is_empty())
        else {
            selection.step(map, delta);
            return;
        };
        let count = built.anchors.len() as isize;
        let current = selection
            .ticket_id()
            .and_then(|id| {
                built
                    .anchors
                    .iter()
                    .position(|(_, anchored)| anchored == id)
            })
            .map_or(0, |index| index as isize);
        let next = (current + delta).rem_euclid(count) as usize;
        selection.select(&built.anchors[next].1);
    }
}

fn anchor(anchors: &[(u16, String)], id: &str) -> Option<u16> {
    anchors
        .iter()
        .find(|(_, anchored)| anchored == id)
        .map(|(row, _)| *row)
}

fn build(map: &Map, selected: Option<&str>, expanded: bool, width: u16) -> Built {
    let mut rows: Vec<Line<'static>> = Vec::new();
    let mut anchors: Vec<(u16, String)> = Vec::new();
    let mut shown = false;

    for logical in markdown_lines(&document_source(map)) {
        let row = rows.len().min(u16::MAX as usize) as u16;
        let (line, mentions) = decorate(logical, map, selected);
        rows.extend(wrap(line, width as usize));
        for id in mentions {
            let Some(ticket) = map.ticket(&id) else {
                continue;
            };
            if !anchors.iter().any(|(_, anchored)| *anchored == id) {
                anchors.push((row, id.clone()));
            }
            if !shown && selected == Some(id.as_str()) {
                rows.extend(expansion(map, ticket, expanded, width as usize));
                shown = true;
            }
        }
    }

    let height = rows.len().clamp(1, u16::MAX as usize) as u16;
    let mut view = ScrollView::new(Size::new(width, height));
    for (row, line) in rows.iter().take(height as usize).enumerate() {
        view.render_widget(line, Rect::new(0, row as u16, width, 1));
    }
    Built { view, anchors }
}

/// `map.md` with its Ticket links marked, followed by the Tickets the prose leaves out.
fn document_source(map: &Map) -> String {
    let linked = linked_tickets(map);
    let mut source = String::new();
    for line in &map.source {
        push_rewritten(&mut source, line);
        source.push('\n');
    }

    let unlinked: Vec<&Ticket> = map
        .tickets
        .iter()
        .filter(|ticket| !linked.contains(&ticket.id))
        .collect();
    if unlinked.is_empty() {
        return source;
    }

    source.push_str("\n## Tickets the prose never links\n\n");
    for ticket in unlinked {
        source.push_str("- ");
        push_link(&mut source, &ticket.id, &ticket.label);
        source.push(' ');
        push_rewritten(&mut source, &clipped(ticket.gist(), GIST_CHARS));
        source.push('\n');
    }
    source
}

fn linked_tickets(map: &Map) -> Vec<String> {
    map.source
        .iter()
        .flat_map(|line| split_ticket_links(line))
        .filter_map(|part| match part {
            LinePart::Link { ticket, .. } => Some(ticket),
            LinePart::Text(_) => None,
        })
        .collect()
}

fn push_rewritten(out: &mut String, line: &str) {
    for part in split_ticket_links(line) {
        match part {
            LinePart::Text(text) => out.push_str(text),
            LinePart::Link { text, ticket } => push_link(out, &ticket, text),
        }
    }
}

fn push_link(out: &mut String, ticket: &str, text: &str) {
    out.push(LINK_OPEN);
    out.push_str(ticket);
    out.push(LINK_SEP);
    out.push_str(text);
    out.push(LINK_CLOSE);
}

fn clipped(text: &str, chars: usize) -> String {
    let trimmed = text.trim().trim_start_matches(['-', '*', '>', '#', ' ']);
    if trimmed.chars().count() <= chars {
        return trimmed.to_string();
    }
    trimmed.chars().take(chars).chain(['…']).collect()
}

fn markdown_lines(source: &str) -> Vec<Line<'static>> {
    tui_markdown::from_str(source)
        .lines
        .into_iter()
        .map(owned_line)
        .collect()
}

fn owned_line(line: Line<'_>) -> Line<'static> {
    Line {
        style: line.style,
        alignment: line.alignment,
        spans: line
            .spans
            .into_iter()
            .map(|span| Span {
                style: span.style,
                content: Cow::Owned(span.content.into_owned()),
            })
            .collect(),
    }
}

enum Scan {
    Text,
    Id(String),
    Label {
        id: String,
        spans: Vec<Span<'static>>,
    },
}

/// Turns the sentinels back into decorated links, reporting the Tickets the line mentions.
fn decorate(
    line: Line<'static>,
    map: &Map,
    selected: Option<&str>,
) -> (Line<'static>, Vec<String>) {
    let style = line.style;
    let alignment = line.alignment;
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut mentions = Vec::new();
    let mut scan = Scan::Text;
    let mut chunk = String::new();

    for span in line.spans {
        for ch in span.content.chars() {
            match ch {
                LINK_OPEN => {
                    flush(&mut chunk, span.style, &mut scan, &mut out);
                    scan = Scan::Id(String::new());
                }
                LINK_SEP => {
                    if let Scan::Id(id) = &mut scan {
                        let id = mem::take(id);
                        scan = Scan::Label {
                            id,
                            spans: Vec::new(),
                        };
                    } else {
                        chunk.push(ch);
                    }
                }
                LINK_CLOSE => {
                    flush(&mut chunk, span.style, &mut scan, &mut out);
                    if let Scan::Label { id, spans } = mem::replace(&mut scan, Scan::Text) {
                        let selected = selected == Some(id.as_str());
                        out.extend(link_spans(map, &id, spans, selected));
                        mentions.push(id);
                    }
                }
                _ => match &mut scan {
                    Scan::Id(id) => id.push(ch),
                    _ => chunk.push(ch),
                },
            }
        }
        flush(&mut chunk, span.style, &mut scan, &mut out);
    }
    if let Scan::Label { spans, .. } = scan {
        out.extend(spans);
    }

    (
        Line {
            spans: out,
            style,
            alignment,
        },
        mentions,
    )
}

fn flush(chunk: &mut String, style: Style, scan: &mut Scan, out: &mut Vec<Span<'static>>) {
    if chunk.is_empty() {
        return;
    }
    let span = Span::styled(mem::take(chunk), style);
    match scan {
        Scan::Label { spans, .. } => spans.push(span),
        _ => out.push(span),
    }
}

fn link_spans(
    map: &Map,
    id: &str,
    label: Vec<Span<'static>>,
    selected: bool,
) -> Vec<Span<'static>> {
    let ticket = map.ticket(id);
    let base = match ticket {
        Some(ticket) => status_style(ticket.status),
        None => Style::new().fg(Color::Red),
    };
    let base = if selected {
        base.add_modifier(Modifier::REVERSED | Modifier::BOLD)
    } else {
        base.add_modifier(Modifier::UNDERLINED)
    };

    let glyph = ticket.map_or("x", |ticket| status_glyph(ticket.status));
    let mut spans = vec![Span::styled(format!("{glyph} "), base)];
    if label.is_empty() {
        spans.push(Span::styled(id.to_string(), base));
    } else {
        spans.extend(label.into_iter().map(|span| span.patch_style(base)));
    }
    spans.push(Span::styled(
        match ticket {
            Some(ticket) => format!(" ({})", ticket.kind.label()),
            None => " (no such Ticket)".to_string(),
        },
        FRAME,
    ));
    spans
}

fn status_style(status: TicketStatus) -> Style {
    Style::new().fg(match status {
        TicketStatus::Open => Color::Yellow,
        TicketStatus::Closed => Color::Green,
        TicketStatus::Unknown => Color::Gray,
    })
}

fn status_glyph(status: TicketStatus) -> &'static str {
    match status {
        TicketStatus::Open => "○",
        TicketStatus::Closed => "●",
        TicketStatus::Unknown => "?",
    }
}

/// The selected Ticket, opened where the prose names it.
fn expansion(map: &Map, ticket: &Ticket, full: bool, width: usize) -> Vec<Line<'static>> {
    let inner = width.saturating_sub(4).max(8);
    let readiness = if ticket.blocked_by.is_empty() {
        "no blockers".to_string()
    } else if map.is_ready(ticket) {
        format!("ready, was blocked by {}", ticket.blocked_by.join(", "))
    } else {
        format!("blocked by {}", ticket.blocked_by.join(", "))
    };

    let header = Line::from(vec![
        Span::styled("  ╭ ", FRAME),
        Span::styled(
            ticket.id.clone(),
            status_style(ticket.status).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  {}  {}  {readiness}",
                ticket.kind.label(),
                ticket.status.label()
            ),
            FRAME,
        ),
    ]);
    let mut rows = wrap(header, width);

    for line in markdown_lines(&body_source(ticket, full)) {
        let (line, _) = decorate(line, map, None);
        rows.extend(wrap(line, inner).into_iter().map(quoted));
    }

    let hint = if full {
        "e question and resolution"
    } else {
        "e whole Ticket"
    };
    rows.push(Line::from(Span::styled(format!("  ╰ {hint}"), FRAME)));
    rows.push(Line::default());
    rows
}

fn body_source(ticket: &Ticket, full: bool) -> String {
    let mut source = String::new();
    if full {
        for line in &ticket.source {
            push_rewritten(&mut source, line);
            source.push('\n');
        }
        return source;
    }

    for section in [ticket.question(), ticket.resolution()]
        .into_iter()
        .flatten()
    {
        push_section(&mut source, section);
    }
    if source.is_empty()
        && let Some(section) = ticket.sections.first()
    {
        push_section(&mut source, section);
    }
    if source.trim().is_empty() {
        source.push_str("_No Question and no Resolution._\n");
    }
    source
}

fn push_section(out: &mut String, section: &Section) {
    let body: Vec<&String> = section
        .body
        .iter()
        .skip_while(|line| line.trim().is_empty())
        .collect();

    out.push_str("### ");
    out.push_str(&section.heading);
    out.push_str("\n\n");
    for line in body.iter().take(SECTION_LINES) {
        push_rewritten(out, line);
        out.push('\n');
    }
    if fences(&body) % 2 == 1 {
        out.push_str("```\n");
    }

    let rest = body.len().saturating_sub(SECTION_LINES);
    if rest > 0 {
        out.push_str(&format!("\n_{rest} more lines._\n"));
    }
    out.push('\n');
}

fn fences(body: &[&String]) -> usize {
    body.iter()
        .take(SECTION_LINES)
        .filter(|line| line.trim_start().starts_with("```"))
        .count()
}

fn quoted(row: Line<'static>) -> Line<'static> {
    let mut spans = vec![Span::styled("  │ ", FRAME)];
    spans.extend(row.spans);
    Line {
        spans,
        style: row.style,
        alignment: row.alignment,
    }
}

/// Breaks a logical line at the text measure, keeping list items aligned under their marker.
fn wrap(line: Line<'static>, width: usize) -> Vec<Line<'static>> {
    let width = width.max(8);
    let indent = hanging_indent(&line).min(width / 3);
    let style = line.style;
    let alignment = line.alignment;

    let mut rows: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used = 0;

    for token in tokens(line) {
        let blank = token.content.trim().is_empty();
        if !current.is_empty() && used + span_width(token.content.trim_end()) > width {
            rows.push(Line {
                spans: mem::take(&mut current),
                style,
                alignment,
            });
            used = 0;
            if blank {
                continue;
            }
            if indent > 0 {
                current.push(Span::raw(" ".repeat(indent)));
                used = indent;
            }
        }
        used += span_width(&token.content);
        current.push(token);
    }
    rows.push(Line {
        spans: current,
        style,
        alignment,
    });
    rows
}

fn tokens(line: Line<'static>) -> Vec<Span<'static>> {
    let mut tokens = Vec::new();
    for span in line.spans {
        for piece in span.content.split_inclusive(' ') {
            tokens.push(Span::styled(piece.to_string(), span.style));
        }
    }
    tokens
}

fn hanging_indent(line: &Line<'_>) -> usize {
    let text: String = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    let spaces = text.chars().take_while(|ch| *ch == ' ').count();
    let rest = &text[spaces..];

    if rest.starts_with("- ") || rest.starts_with("* ") {
        return spaces + 2;
    }
    let digits = rest.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digits > 0 && rest[digits..].starts_with(". ") {
        return spaces + digits + 2;
    }
    spaces
}

fn span_width(text: &str) -> usize {
    Span::raw(text).width()
}
