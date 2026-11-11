//! Full-text search over `.md` notes (title + body, snippet with context).

use tauri::AppHandle;

use crate::models::SearchHit;
use crate::storage::Storage;

pub fn search(app: &AppHandle, query: &str) -> Result<Vec<SearchHit>, String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let store = Storage::new(crate::storage::data_dir(app)?)?;
    let mut hits = Vec::new();
    for id in store.list_ids()? {
        let note = match store.read_note(&id) {
            Ok(n) => n,
            Err(_) => continue,
        };
        if note.meta.trashed {
            continue;
        }
        let title_l = note.meta.title.to_lowercase();
        let body_l = note.body.to_lowercase();
        if !title_l.contains(&q) && !body_l.contains(&q) {
            continue;
        }
        // Build a snippet around the first body match (or title).
        let snippet = if let Some(idx) = body_l.find(&q) {
            let start = idx.saturating_sub(40);
            let end = (idx + q.len() + 60).min(note.body.len());
            let raw = &note.body[start..end];
            let clean: String = raw
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            format!("…{}…", clean.trim())
        } else {
            note.body
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or("")
                .to_string()
        };
        hits.push(SearchHit {
            id: note.meta.id.clone(),
            title: note.meta.title.clone(),
            color: note.meta.color.clone(),
            snippet,
            updated_at: note.meta.updated_at,
        });
    }
    hits.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(hits)
}
