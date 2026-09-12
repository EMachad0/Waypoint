//! Loads Maps from a directory of wayfinder charts, one subdirectory per Map.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::map::{Map, Section, Ticket, TicketKind, TicketStatus};

const DEFAULT_DIR: &str = "/Users/eliton.silva/home/ume/monorepo-python/docs/wayfinder";
const DIR_ENV: &str = "WAYFINDER_MAPS_DIR";

/// Maps that loaded, plus a note for every directory that did not.
pub struct Fixtures {
    pub dir: PathBuf,
    pub maps: Vec<Map>,
    pub notes: Vec<String>,
}

/// First command line argument, else `WAYFINDER_MAPS_DIR`, else the charts this prototype was
/// built against.
pub fn fixtures_dir() -> PathBuf {
    env::args()
        .skip(1)
        .find(|argument| !argument.starts_with('-'))
        .or_else(|| env::var(DIR_ENV).ok())
        .map_or_else(|| PathBuf::from(DEFAULT_DIR), PathBuf::from)
}

pub fn load(dir: &Path) -> Fixtures {
    let mut maps = Vec::new();
    let mut notes = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            notes.push(format!("{} unreadable: {error}", dir.display()));
            return Fixtures {
                dir: dir.to_path_buf(),
                maps,
                notes,
            };
        }
    };

    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();

    for path in dirs {
        match load_map(&path) {
            Ok(map) => maps.push(map),
            Err(reason) => notes.push(format!("skipped {}: {reason}", file_name(&path))),
        }
    }

    Fixtures {
        dir: dir.to_path_buf(),
        maps,
        notes,
    }
}

fn load_map(dir: &Path) -> Result<Map, String> {
    let source = read_lines(&dir.join("map.md"))?;

    let title = source
        .iter()
        .find_map(|line| line.strip_prefix("# "))
        .ok_or("map.md carries no title")?
        .trim()
        .to_string();

    Ok(Map {
        id: file_name(dir),
        title,
        sections: sections(&source),
        tickets: load_tickets(&dir.join("tickets")),
        source,
    })
}

fn load_tickets(dir: &Path) -> Vec<Ticket> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();

    paths.iter().filter_map(|path| load_ticket(path)).collect()
}

fn load_ticket(path: &Path) -> Option<Ticket> {
    let lines = read_lines(path).ok()?;
    let (front_matter, body) = split_front_matter(&lines);
    let id = path.file_stem()?.to_string_lossy().to_string();

    let kind = field(&front_matter, "type").map_or(TicketKind::Unknown, TicketKind::parse);
    let status = field(&front_matter, "status").map_or(TicketStatus::Unknown, TicketStatus::parse);

    Some(Ticket {
        label: label_from(&id, kind),
        kind,
        status,
        blocked_by: field(&front_matter, "blocked_by")
            .map(parse_list)
            .unwrap_or_default(),
        assignee: field(&front_matter, "assignee").map(str::to_string),
        sections: sections(body),
        source: body.to_vec(),
        id,
    })
}

/// Turns `03-grilling-naming-and-glossary` into `Naming and glossary`.
fn label_from(id: &str, kind: TicketKind) -> String {
    let mut words: Vec<&str> = id.split('-').collect();
    if words
        .first()
        .is_some_and(|word| word.parse::<u32>().is_ok())
    {
        words.remove(0);
    }
    if words.first().is_some_and(|word| *word == kind.label()) {
        words.remove(0);
    }
    let joined = words.join(" ");
    let mut chars = joined.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => id.to_string(),
    }
}

fn split_front_matter(lines: &[String]) -> (Vec<&str>, &[String]) {
    if lines.first().map(String::as_str) != Some("---") {
        return (Vec::new(), lines);
    }
    match lines[1..].iter().position(|line| line == "---") {
        Some(end) => (
            lines[1..=end].iter().map(String::as_str).collect(),
            &lines[end + 2..],
        ),
        None => (Vec::new(), lines),
    }
}

fn field<'a>(front_matter: &[&'a str], key: &str) -> Option<&'a str> {
    front_matter.iter().find_map(|line| {
        line.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(':'))
            .map(str::trim)
    })
}

fn parse_list(raw: &str) -> Vec<String> {
    raw.trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty() && *item != "none")
        .map(str::to_string)
        .collect()
}

fn sections(lines: &[String]) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    for line in lines {
        match line.strip_prefix("## ") {
            Some(heading) => sections.push(Section {
                heading: heading.trim().to_string(),
                body: Vec::new(),
            }),
            None => {
                if let Some(section) = sections.last_mut() {
                    section.body.push(line.clone());
                }
            }
        }
    }
    for section in &mut sections {
        while section
            .body
            .last()
            .is_some_and(|line| line.trim().is_empty())
        {
            section.body.pop();
        }
    }
    sections
}

fn read_lines(path: &Path) -> Result<Vec<String>, String> {
    fs::read_to_string(path)
        .map(|text| text.lines().map(str::to_string).collect())
        .map_err(|error| format!("{}: {error}", file_name(path)))
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .to_string()
}
