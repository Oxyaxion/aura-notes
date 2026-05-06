use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};

use serde::Serialize;

use crate::frontmatter::ParsedNote;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct NoteRow {
    pub name: String,
    pub title: Option<String>,
    pub date: Option<String>,
    pub status: Option<String>,
    pub tags: Vec<String>,
    pub modified_at: i64,
    pub aliases: Vec<String>,
    pub note_type: Option<String>,
    pub due: Option<String>,
    pub url: Option<String>,
    pub author: Option<String>,
    pub rating: Option<i64>,
    pub pinned: bool,
    pub locked: bool,
    pub area: Option<String>,
    pub priority: Option<String>,
    pub project: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub name: String,
    pub title: Option<String>,
    pub snippet: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TagCount {
    pub tag: String,
    pub count: usize,
}

// ── Internal storage ──────────────────────────────────────────────────────────

struct StoredNote {
    row: NoteRow,
    body: String,
}

// ── Note index ────────────────────────────────────────────────────────────────

pub struct Db(RwLock<HashMap<String, StoredNote>>);

impl Db {
    pub fn new() -> Self {
        Db(RwLock::new(HashMap::new()))
    }

    pub fn upsert(&self, name: &str, parsed: &ParsedNote, modified_at: i64) {
        let row = NoteRow {
            name: name.to_string(),
            title: parsed.title.clone(),
            date: parsed.date.clone(),
            status: parsed.status.clone(),
            tags: parsed.tags.clone(),
            modified_at,
            aliases: parsed.aliases.clone(),
            note_type: parsed.note_type.clone(),
            due: parsed.due.clone(),
            url: parsed.url.clone(),
            author: parsed.author.clone(),
            rating: parsed.rating,
            pinned: parsed.pinned,
            locked: parsed.locked,
            area: parsed.area.clone(),
            priority: parsed.priority.clone(),
            project: parsed.project.clone(),
            last_modified: parsed.last_modified.clone(),
        };
        self.0.write().unwrap().insert(
            name.to_string(),
            StoredNote { row, body: parsed.body.clone() },
        );
    }

    pub fn delete(&self, name: &str) {
        self.0.write().unwrap().remove(name);
    }

    pub fn rename(&self, old_name: &str, new_name: &str) {
        let mut index = self.0.write().unwrap();
        if let Some(mut stored) = index.remove(old_name) {
            stored.row.name = new_name.to_string();
            index.insert(new_name.to_string(), stored);
        }
    }

    pub fn search(&self, q: &str) -> Vec<SearchResult> {
        if q.trim().is_empty() {
            return vec![];
        }
        let lower_q = q.to_lowercase();
        let index = self.0.read().unwrap();
        let mut results: Vec<SearchResult> = index
            .values()
            .filter(|n| {
                let title_hay = n.row.title.as_deref().unwrap_or(&n.row.name).to_lowercase();
                title_hay.contains(&lower_q) || n.body.to_lowercase().contains(&lower_q)
            })
            .map(|n| SearchResult {
                name: n.row.name.clone(),
                title: n.row.title.clone(),
                snippet: make_snippet(&n.body, q),
            })
            .collect();
        results.sort_by(|a, b| a.name.cmp(&b.name));
        results
    }

    pub fn list_tags(&self) -> Vec<TagCount> {
        let index = self.0.read().unwrap();
        let mut counts: HashMap<String, usize> = HashMap::new();
        for note in index.values() {
            for tag in &note.row.tags {
                *counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        let mut tags: Vec<TagCount> = counts
            .into_iter()
            .map(|(tag, count)| TagCount { tag, count })
            .collect();
        tags.sort_by(|a, b| b.count.cmp(&a.count).then(a.tag.cmp(&b.tag)));
        tags
    }

    pub fn get_aliases(&self) -> HashMap<String, String> {
        let index = self.0.read().unwrap();
        let mut map = HashMap::new();
        for note in index.values() {
            for alias in &note.row.aliases {
                map.insert(alias.clone(), note.row.name.clone());
            }
        }
        map
    }

    pub fn get_note_aliases(&self, name: &str) -> Vec<String> {
        self.0
            .read()
            .unwrap()
            .get(name)
            .map(|n| n.row.aliases.clone())
            .unwrap_or_default()
    }

    pub fn get_meta_for_list(&self) -> HashMap<String, (bool, bool, bool)> {
        let index = self.0.read().unwrap();
        index
            .values()
            .map(|n| {
                let is_template = n.row.note_type.as_deref() == Some("template");
                let is_index    = n.row.note_type.as_deref() == Some("index");
                (n.row.name.clone(), (n.row.pinned, is_template, is_index))
            })
            .collect()
    }

    pub fn list_all_meta(&self) -> Vec<(String, bool, bool, bool)> {
        let index = self.0.read().unwrap();
        index.values().map(|n| {
            let is_template = n.row.note_type.as_deref() == Some("template");
            let is_index    = n.row.note_type.as_deref() == Some("index");
            (n.row.name.clone(), n.row.pinned, is_template, is_index)
        }).collect()
    }

    pub fn get_field_values(&self, field: &str) -> Vec<String> {
        let index = self.0.read().unwrap();
        let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for note in index.values() {
            let val: Option<String> = match field {
                "status" => note.row.status.clone(),
                "area"   => note.row.area.clone(),
                "author" => note.row.author.clone(),
                "type"   => note.row.note_type.clone(),
                "due"    => note.row.due.clone(),
                "url"    => note.row.url.clone(),
                "rating"   => note.row.rating.map(|r| r.to_string()),
                "priority"     => note.row.priority.clone(),
                "project"      => note.row.project.clone(),
                "lastModified" => note.row.last_modified.clone(),
                _              => None,
            };
            if let Some(v) = val && !v.is_empty() {
                values.insert(v);
            }
        }
        values.into_iter().collect()
    }

    pub fn get_media_usage(&self) -> (HashSet<String>, HashSet<String>) {
        static ASSET_RE: OnceLock<regex::Regex> = OnceLock::new();
        static DRAWING_RE: OnceLock<regex::Regex> = OnceLock::new();
        let asset_re = ASSET_RE.get_or_init(|| regex::Regex::new(r#"/assets/([^)\s"'\n>]+)"#).unwrap());
        let drawing_re = DRAWING_RE.get_or_init(|| regex::Regex::new(r"(?m)^```drawing\r?\n(\S+)").unwrap());

        let index = self.0.read().unwrap();
        let mut used_assets = HashSet::new();
        let mut used_drawings = HashSet::new();
        for note in index.values() {
            for cap in asset_re.captures_iter(&note.body) {
                used_assets.insert(cap[1].to_string());
            }
            for cap in drawing_re.captures_iter(&note.body) {
                used_drawings.insert(cap[1].to_string());
            }
        }
        (used_assets, used_drawings)
    }

    pub fn query_notes(&self, q: &str) -> Vec<NoteRow> {
        let index = self.0.read().unwrap();
        let (or_groups, global_not, recent, oldest, order_by) = parse_query(q);

        let mut results: Vec<&NoteRow> = index
            .values()
            .map(|n| &n.row)
            .filter(|note| {
                let or_match = or_groups.is_empty()
                    || or_groups.iter().any(|group| group.iter().all(|p| p.eval(note)));
                let not_match = global_not.iter().all(|p| p.eval(note));
                or_match && not_match
            })
            .collect();

        // recent:N / oldest:N limit by modification time before any order by
        if let Some(n) = recent {
            results.sort_by_key(|r| std::cmp::Reverse(r.modified_at));
            results.truncate(n);
        } else if let Some(n) = oldest {
            results.sort_by_key(|r| r.modified_at);
            results.truncate(n);
        }

        // order by overrides the final sort (after potential recent/oldest truncation)
        if let Some(ref ob) = order_by {
            results.sort_by(|a, b| {
                let cmp = compare_by_field(a, b, &ob.field);
                if ob.desc { cmp.reverse() } else { cmp }
            });
        } else if recent.is_none() && oldest.is_none() {
            results.sort_by(|a, b| a.name.cmp(&b.name));
        }

        results.into_iter().cloned().collect()
    }
}

// ── Snippet helper ────────────────────────────────────────────────────────────

fn make_snippet(body: &str, query: &str) -> String {
    let lower_body = body.to_lowercase();
    let lower_q = query.to_lowercase();
    if let Some(pos) = lower_body.find(&lower_q) {
        let start = pos.saturating_sub(80);
        let end = (pos + lower_q.len() + 80).min(body.len());
        let start = body.char_indices().map(|(i, _)| i).filter(|&i| i <= start).next_back().unwrap_or(0);
        let end = body.char_indices().map(|(i, _)| i).find(|&i| i >= end).unwrap_or(body.len());
        let prefix = if start > 0 { "…" } else { "" };
        let suffix = if end < body.len() { "…" } else { "" };
        let raw = body[start..end].trim();
        let clean = raw
            .trim_start_matches(['#', '-', '*', '>', ' ', '\t'])
            .trim();
        format!("{}{}{}", prefix, clean, suffix)
    } else {
        body.chars().take(160).collect()
    }
}

// ── Query DSL ─────────────────────────────────────────────────────────────────

/// A single filter predicate, with a `not` flag for negation.
enum Pred {
    Tag(String, bool),        // prefix match on any tag (lowercase)
    Status(String, bool),     // exact match
    DatePrefix(String, bool), // starts_with
    Title(String, bool),      // substring on title field only (falls back to name if no title)
    Text(String, bool),       // bare word: substring on title OR name (lowercase)
    Path(String, bool),       // substring on full path (lowercase)
    FileName(String, bool),   // substring on last segment only (lowercase)
    NoteType(String, bool),   // exact match
    DuePrefix(String, bool),  // starts_with
    Area(String, bool),       // substring (lowercase)
    Author(String, bool),     // substring (lowercase)
    Rating(i64, bool),        // exact match
    Alias(String, bool),      // exact match (lowercase)
    Url(String, bool),        // substring (lowercase)
    Pinned(bool),             // expected value (NOT already folded in at parse time)
    Locked(bool),
    Priority(String, bool),        // exact match (high/medium/low)
    Project(String, bool),         // substring (lowercase)
    LastModified(String, bool),    // starts_with
    Depth(usize, bool),            // exact slash count in name
}

impl Pred {
    fn eval(&self, note: &NoteRow) -> bool {
        match self {
            Pred::Tag(prefix, not) => {
                let has = note.tags.iter().any(|t| t.to_lowercase().starts_with(prefix.as_str()));
                if *not { !has } else { has }
            }
            Pred::Status(val, not) => {
                let m = note.status.as_deref() == Some(val.as_str());
                if *not { !m } else { m }
            }
            Pred::DatePrefix(prefix, not) => {
                let m = note.date.as_deref().is_some_and(|d| d.starts_with(prefix.as_str()));
                if *not { !m } else { m }
            }
            Pred::Title(pattern, not) => {
                let segment = note.name.split('/').next_back().unwrap_or(&note.name);
                let hay = note.title.as_deref().unwrap_or(segment).to_lowercase();
                let m = hay.contains(pattern.as_str());
                if *not { !m } else { m }
            }
            Pred::Text(pattern, not) => {
                let title_m = note.title.as_deref()
                    .is_some_and(|t| t.to_lowercase().contains(pattern.as_str()));
                let name_m = note.name.to_lowercase().contains(pattern.as_str());
                let m = title_m || name_m;
                if *not { !m } else { m }
            }
            Pred::Path(pattern, not) => {
                let m = note.name.to_lowercase().contains(pattern.as_str());
                if *not { !m } else { m }
            }
            Pred::FileName(pattern, not) => {
                let segment = note.name.split('/').next_back().unwrap_or(&note.name).to_lowercase();
                let m = segment.contains(pattern.as_str());
                if *not { !m } else { m }
            }
            Pred::NoteType(val, not) => {
                let m = note.note_type.as_deref() == Some(val.as_str());
                if *not { !m } else { m }
            }
            Pred::DuePrefix(prefix, not) => {
                let m = note.due.as_deref().is_some_and(|d| d.starts_with(prefix.as_str()));
                if *not { !m } else { m }
            }
            Pred::Area(pattern, not) => {
                let m = note.area.as_deref().is_some_and(|s| s.to_lowercase().contains(pattern.as_str()));
                if *not { !m } else { m }
            }
            Pred::Author(pattern, not) => {
                let m = note.author.as_deref().is_some_and(|s| s.to_lowercase().contains(pattern.as_str()));
                if *not { !m } else { m }
            }
            Pred::Rating(val, not) => {
                let m = note.rating == Some(*val);
                if *not { !m } else { m }
            }
            Pred::Alias(val, not) => {
                let m = note.aliases.iter().any(|a| a.to_lowercase() == val.as_str());
                if *not { !m } else { m }
            }
            Pred::Url(pattern, not) => {
                let m = note.url.as_deref().is_some_and(|s| s.to_lowercase().contains(pattern.as_str()));
                if *not { !m } else { m }
            }
            Pred::Pinned(expected) => note.pinned == *expected,
            Pred::Locked(expected) => note.locked == *expected,
            Pred::Priority(val, not) => {
                let m = note.priority.as_deref() == Some(val.as_str());
                if *not { !m } else { m }
            }
            Pred::Project(pattern, not) => {
                let m = note.project.as_deref().is_some_and(|s| s.to_lowercase().contains(pattern.as_str()));
                if *not { !m } else { m }
            }
            Pred::LastModified(prefix, not) => {
                let m = note.last_modified.as_deref().is_some_and(|d| d.starts_with(prefix.as_str()));
                if *not { !m } else { m }
            }
            Pred::Depth(n, not) => {
                let count = note.name.chars().filter(|&c| c == '/').count();
                let m = count == *n;
                if *not { !m } else { m }
            }
        }
    }
}

// ── Order by ─────────────────────────────────────────────────────────────────

struct OrderBy {
    field: String,
    desc: bool,
}

fn compare_by_field(a: &NoteRow, b: &NoteRow, field: &str) -> std::cmp::Ordering {
    match field {
        "name"                  => a.name.cmp(&b.name),
        "title"                 => {
            let ta = a.title.as_deref().unwrap_or(&a.name);
            let tb = b.title.as_deref().unwrap_or(&b.name);
            ta.cmp(tb)
        }
        "date"                  => cmp_opt_str(a.date.as_deref(), b.date.as_deref()),
        "modified" | "modified_at" => a.modified_at.cmp(&b.modified_at),
        "due"                   => cmp_opt_str(a.due.as_deref(), b.due.as_deref()),
        "status"                => cmp_opt_str(a.status.as_deref(), b.status.as_deref()),
        "rating"                => cmp_opt_i64(a.rating, b.rating),
        "area"                  => cmp_opt_str(a.area.as_deref(), b.area.as_deref()),
        "author"                => cmp_opt_str(a.author.as_deref(), b.author.as_deref()),
        "priority"              => cmp_priority(a.priority.as_deref(), b.priority.as_deref()),
        "project"               => cmp_opt_str(a.project.as_deref(), b.project.as_deref()),
        "lastModified"          => cmp_opt_str(a.last_modified.as_deref(), b.last_modified.as_deref()),
        _                       => std::cmp::Ordering::Equal,
    }
}

fn cmp_priority(a: Option<&str>, b: Option<&str>) -> std::cmp::Ordering {
    fn rank(p: Option<&str>) -> u8 {
        match p {
            Some("high")   => 0,
            Some("medium") => 1,
            Some("low")    => 2,
            _              => 3,
        }
    }
    rank(a).cmp(&rank(b))
}

fn cmp_opt_str(a: Option<&str>, b: Option<&str>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None)    => std::cmp::Ordering::Less,
        (None, Some(_))    => std::cmp::Ordering::Greater,
        (None, None)       => std::cmp::Ordering::Equal,
    }
}

fn cmp_opt_i64(a: Option<i64>, b: Option<i64>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None)    => std::cmp::Ordering::Less,
        (None, Some(_))    => std::cmp::Ordering::Greater,
        (None, None)       => std::cmp::Ordering::Equal,
    }
}

// ── Query parser ──────────────────────────────────────────────────────────────

/// Parse a DSL query string into OR-groups of AND-predicates.
///
/// Precedence: AND binds tighter than OR.
///   `A OR B AND C`  →  `[[A], [B, C]]`  →  A OR (B AND C)
fn parse_query(q: &str) -> (Vec<Vec<Pred>>, Vec<Pred>, Option<usize>, Option<usize>, Option<OrderBy>) {
    let mut or_groups: Vec<Vec<Pred>> = vec![vec![]];
    let mut global_not: Vec<Pred> = vec![];
    let mut pending_not = false;
    let mut pending_or = false;
    let mut recent: Option<usize> = None;
    let mut oldest: Option<usize> = None;
    let mut order_by: Option<OrderBy> = None;

    // order by parsing state: 0=idle, 1=expect BY, 2=expect field, 3=expect asc/desc
    let mut ob_state: u8 = 0;
    let mut ob_field = String::new();

    for token in q.split_whitespace() {
        let upper = token.to_ascii_uppercase();

        // order by state machine
        match ob_state {
            1 => { if upper == "BY" { ob_state = 2; } else { ob_state = 0; } continue; }
            2 => { ob_field = token.to_lowercase(); ob_state = 3; continue; }
            3 => {
                let desc = upper == "DESC";
                if upper == "ASC" || desc {
                    order_by = Some(OrderBy { field: ob_field.clone(), desc });
                    ob_state = 0;
                    continue;
                }
                // No direction token — commit with default asc and fall through
                order_by = Some(OrderBy { field: ob_field.clone(), desc: false });
                ob_state = 0;
                // fall through to normal token handling below
            }
            _ => {}
        }

        match upper.as_str() {
            "OR"    => { pending_or  = true; continue; }
            "AND"   => {                     continue; }
            "NOT"   => { pending_not = true; continue; }
            "ORDER" => { ob_state = 1;       continue; }
            _ => {}
        }

        if pending_or {
            or_groups.push(vec![]);
            pending_or = false;
        }

        let not = std::mem::replace(&mut pending_not, false);

        // NOT predicates are global exclusions — applied after OR-group evaluation
        // so `A OR B AND NOT C` means `(A OR B) AND NOT C`, not `A OR (B AND NOT C)`.
        let dest_is_global = not;

        let pred = if let Some(v) = token.strip_prefix('#') {
            Pred::Tag(v.to_lowercase(), not)
        } else if let Some((k, v)) = token.split_once(':') {
            match k.to_lowercase().as_str() {
                "tag"    => Pred::Tag(v.to_lowercase(), not),
                "status" => Pred::Status(v.to_string(), not),
                "date"   => Pred::DatePrefix(v.to_string(), not),
                "title"  => Pred::Title(v.to_lowercase(), not),
                "path" => Pred::Path(v.to_lowercase(), not),
                "name" => Pred::FileName(v.to_lowercase(), not),
                "type"   => Pred::NoteType(v.to_string(), not),
                "due"    => Pred::DuePrefix(v.to_string(), not),
                "area"   => Pred::Area(v.to_lowercase(), not),
                "author" => Pred::Author(v.to_lowercase(), not),
                "rating" => match v.parse::<i64>() {
                    Ok(n)  => Pred::Rating(n, not),
                    Err(_) => continue,
                },
                "alias"    => Pred::Alias(v.to_lowercase(), not),
                "url"      => Pred::Url(v.to_lowercase(), not),
                "priority"     => Pred::Priority(v.to_lowercase(), not),
                "project"      => Pred::Project(v.to_lowercase(), not),
                "lastmodified" => Pred::LastModified(v.to_string(), not),
                "depth" => match v.parse::<usize>() {
                    Ok(n)  => Pred::Depth(n, not),
                    Err(_) => continue,
                },
                "pinned" => match v {
                    "true"  => Pred::Pinned(!not),
                    "false" => Pred::Pinned(not),
                    _       => continue,
                },
                "locked" => match v {
                    "true"  => Pred::Locked(!not),
                    "false" => Pred::Locked(not),
                    _       => continue,
                },
                "recent" => {
                    if let Ok(n) = v.parse::<usize>() { recent = Some(n); }
                    continue;
                }
                "oldest" => {
                    if let Ok(n) = v.parse::<usize>() { oldest = Some(n); }
                    continue;
                }
                _ => continue,
            }
        } else {
            // Bare token → title OR name search
            Pred::Text(token.to_lowercase(), not)
        };

        if dest_is_global {
            global_not.push(pred);
        } else {
            or_groups.last_mut().unwrap().push(pred);
        }
    }

    // Flush a pending order field with no direction token
    if ob_state == 3 && !ob_field.is_empty() {
        order_by = Some(OrderBy { field: ob_field, desc: false });
    }

    // Drop empty groups (e.g. from a leading OR)
    let or_groups = or_groups.into_iter().filter(|g| !g.is_empty()).collect();
    (or_groups, global_not, recent, oldest, order_by)
}
