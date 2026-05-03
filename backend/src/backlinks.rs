use std::{collections::HashMap, path::Path, sync::{Arc, OnceLock}};
use axum::{extract::{Path as AxumPath, State}, http::StatusCode, response::Json};
use regex::Regex;
use serde::Serialize;

use crate::AppState;

#[derive(Default)]
pub struct BacklinkIndex {
    // target_note -> set of notes that link to it
    index: HashMap<String, Vec<String>>,
}

impl BacklinkIndex {
    pub async fn build(notes_dir: &Path) -> Self {
        static WIKILINK_RE: OnceLock<Regex> = OnceLock::new();
        let notes_dir = notes_dir.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let re = WIKILINK_RE.get_or_init(|| Regex::new(r"\[\[([^\]]+)\]\]").unwrap());
            let mut forward: HashMap<String, Vec<String>> = HashMap::new();
            for entry in walkdir::WalkDir::new(&notes_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let Ok(rel) = path.strip_prefix(&notes_dir) else { continue };
                let source = rel.with_extension("").to_string_lossy().replace('\\', "/");
                if let Ok(content) = std::fs::read_to_string(path) {
                    for cap in re.captures_iter(&content) {
                        forward
                            .entry(cap[1].to_string())
                            .or_default()
                            .push(source.clone());
                    }
                }
            }
            BacklinkIndex { index: forward }
        })
        .await
        .unwrap_or_default()
    }

    pub fn get(&self, note: &str) -> Vec<String> {
        self.index.get(note).cloned().unwrap_or_default()
    }
}

#[derive(Serialize)]
pub struct BacklinksResponse {
    pub note: String,
    pub backlinks: Vec<String>,
}

pub async fn get_backlinks(
    State(state): State<Arc<AppState>>,
    AxumPath(name): AxumPath<String>,
) -> Result<Json<BacklinksResponse>, StatusCode> {
    let index = state.backlink_index.read().await;
    let mut backlinks = index.get(&name);

    // Also gather backlinks via aliases
    let db = state.db.clone();
    let name_clone = name.clone();
    let aliases = tokio::task::spawn_blocking(move || db.get_note_aliases(&name_clone))
        .await
        .unwrap_or_default();
    for alias in aliases {
        for bl in index.get(&alias) {
            if !backlinks.contains(&bl) {
                backlinks.push(bl);
            }
        }
    }

    Ok(Json(BacklinksResponse {
        backlinks,
        note: name,
    }))
}
