//! The Map and Ticket types the prototype variants render against.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketKind {
    Grilling,
    Research,
    Task,
    Prototype,
    Unknown,
}

impl TicketKind {
    pub fn parse(raw: &str) -> Self {
        match raw.trim() {
            "grilling" => Self::Grilling,
            "research" => Self::Research,
            "task" => Self::Task,
            "prototype" => Self::Prototype,
            _ => Self::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Grilling => "grilling",
            Self::Research => "research",
            Self::Task => "task",
            Self::Prototype => "prototype",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketStatus {
    Open,
    Closed,
    Unknown,
}

impl TicketStatus {
    pub fn parse(raw: &str) -> Self {
        match raw.trim() {
            "open" => Self::Open,
            "closed" => Self::Closed,
            _ => Self::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Unknown => "unknown",
        }
    }
}

/// A `## ` heading and the lines under it.
#[derive(Debug, Clone)]
pub struct Section {
    pub heading: String,
    pub body: Vec<String>,
}

impl Section {
    /// First line carrying text, empty when the section has none.
    pub fn gist(&self) -> &str {
        self.body
            .iter()
            .map(String::as_str)
            .find(|line| !line.trim().is_empty())
            .unwrap_or("")
    }
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: String,
    pub label: String,
    pub kind: TicketKind,
    pub status: TicketStatus,
    pub blocked_by: Vec<String>,
    pub assignee: Option<String>,
    pub sections: Vec<Section>,
    pub source: Vec<String>,
}

impl Ticket {
    pub fn is_open(&self) -> bool {
        self.status == TicketStatus::Open
    }

    pub fn section_starting_with(&self, prefix: &str) -> Option<&Section> {
        self.sections
            .iter()
            .find(|section| section.heading.starts_with(prefix))
    }

    pub fn question(&self) -> Option<&Section> {
        self.section_starting_with("Question")
    }

    pub fn resolution(&self) -> Option<&Section> {
        self.section_starting_with("Resolution")
    }

    /// One line standing for the whole Ticket, taken from its question.
    pub fn gist(&self) -> &str {
        self.question()
            .or_else(|| self.sections.first())
            .map(Section::gist)
            .unwrap_or("")
    }
}

/// A markdown line cut at the Ticket links inside it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinePart<'a> {
    Text(&'a str),
    Link { text: &'a str, ticket: String },
}

/// Splits a line on `[text](tickets/<id>.md)` links, leaving every other link as text.
pub fn split_ticket_links(line: &str) -> Vec<LinePart<'_>> {
    let mut parts = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        let Some(link) = parse_link(&rest[open..]) else {
            let (head, tail) = rest.split_at(open + 1);
            push_text(&mut parts, head);
            rest = tail;
            continue;
        };
        push_text(&mut parts, &rest[..open]);
        parts.push(LinePart::Link {
            text: link.text,
            ticket: link.ticket,
        });
        rest = &rest[open + link.len..];
    }
    push_text(&mut parts, rest);
    parts
}

struct ParsedLink<'a> {
    text: &'a str,
    ticket: String,
    len: usize,
}

fn parse_link(at: &str) -> Option<ParsedLink<'_>> {
    let close = at.find("](")?;
    let end = at[close..].find(')')? + close;
    let target = &at[close + 2..end];
    let ticket = target
        .strip_prefix("tickets/")
        .and_then(|name| name.strip_suffix(".md"))?;
    Some(ParsedLink {
        text: &at[1..close],
        ticket: ticket.to_string(),
        len: end + 1,
    })
}

fn push_text<'a>(parts: &mut Vec<LinePart<'a>>, text: &'a str) {
    if !text.is_empty() {
        parts.push(LinePart::Text(text));
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub id: String,
    pub title: String,
    pub sections: Vec<Section>,
    pub tickets: Vec<Ticket>,
    pub source: Vec<String>,
}

impl Map {
    pub fn ticket(&self, id: &str) -> Option<&Ticket> {
        self.tickets.iter().find(|ticket| ticket.id == id)
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.tickets.iter().position(|ticket| ticket.id == id)
    }

    pub fn section(&self, heading: &str) -> Option<&Section> {
        self.sections
            .iter()
            .find(|section| section.heading == heading)
    }

    pub fn destination(&self) -> Option<&Section> {
        self.section("Destination")
    }

    /// Tickets naming `id` in their `blocked_by`.
    pub fn blocks(&self, id: &str) -> Vec<&Ticket> {
        self.tickets
            .iter()
            .filter(|ticket| ticket.blocked_by.iter().any(|blocker| blocker == id))
            .collect()
    }

    /// True when nothing the Ticket waits on is still open.
    pub fn is_ready(&self, ticket: &Ticket) -> bool {
        ticket.blocked_by.iter().all(|id| {
            self.ticket(id)
                .is_none_or(|blocker| blocker.status == TicketStatus::Closed)
        })
    }

    pub fn open_tickets(&self) -> usize {
        self.tickets.iter().filter(|t| t.is_open()).count()
    }

    /// Edges of the Ticket graph, each a `(blocker, blocked)` pair of indices.
    pub fn edges(&self) -> Vec<(usize, usize)> {
        let mut edges = Vec::new();
        for (blocked, ticket) in self.tickets.iter().enumerate() {
            for blocker in &ticket.blocked_by {
                if let Some(blocker) = self.index_of(blocker) {
                    edges.push((blocker, blocked));
                }
            }
        }
        edges
    }
}
