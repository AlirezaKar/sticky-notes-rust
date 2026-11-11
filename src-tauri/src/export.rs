//! Export notes to .md / .html / .json / .zip via save dialog.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::storage::{self, Storage};

fn safe_filename(title: &str, fallback: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let trimmed = cleaned.trim_matches('_').to_string();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.chars().take(60).collect()
    }
}

fn note_to_html(note: &crate::models::Note) -> String {
    // Minimal standalone HTML (body is escaped; full Markdown rendering
    // happens in the app's preview — export keeps source + simple wrap).
    let escaped = html_escape(&note.body);
    format!(
        "<!doctype html>\n<html><head><meta charset=\"utf-8\"><title>{}</title><style>body{{font-family:system-ui,sans-serif;max-width:720px;margin:2rem auto;padding:0 1rem;line-height:1.6}}pre{{background:#f4f4f4;padding:1rem;border-radius:8px;overflow:auto}}</style></head><body><h1>{}</h1><pre>{}</pre></body></html>",
        html_escape(&note.meta.title),
        html_escape(&note.meta.title),
        escaped
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn export_note(app: &AppHandle, id: &str, format: &str) -> Result<String, String> {
    let store = Storage::new(storage::data_dir(app)?)?;
    let note = store.read_note(id)?;
    let base = safe_filename(&note.meta.title, id);
    let (ext, contents) = match format {
        "md" => ("md", storage::serialize_note(&note)),
        "html" => ("html", note_to_html(&note)),
        "json" => (
            "json",
            serde_json::to_string_pretty(&note).map_err(|e| e.to_string())?,
        ),
        _ => return Err(format!("unsupported format: {format}")),
    };
    let path = app
        .dialog()
        .file()
        .set_file_name(format!("{base}.{ext}"))
        .blocking_save_file()
        .ok_or_else(|| "save cancelled".to_string())?;
    let path = path.as_path().ok_or_else(|| "invalid path".to_string())?;
    fs::write(path, contents).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

pub fn export_all(app: &AppHandle, format: &str) -> Result<String, String> {
    let store = Storage::new(storage::data_dir(app)?)?;
    let notes: Vec<_> = store
        .list_ids()?
        .iter()
        .filter_map(|id| store.read_note(id).ok())
        .filter(|n| !n.meta.trashed)
        .collect();

    match format {
        "zip" => {
            let path = app
                .dialog()
                .file()
                .set_file_name("sticky-notes-export.zip")
                .blocking_save_file()
                .ok_or_else(|| "save cancelled".to_string())?;
            let path = path.as_path().ok_or_else(|| "invalid path".to_string())?;
            let file = fs::File::create(path).map_err(|e| e.to_string())?;
            let mut zip = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default();
            for note in &notes {
                let name = format!("{}.md", safe_filename(&note.meta.title, &note.meta.id));
                zip.start_file(name, opts).map_err(|e| e.to_string())?;
                zip.write_all(storage::serialize_note(note).as_bytes())
                    .map_err(|e| e.to_string())?;
            }
            zip.finish().map_err(|e| e.to_string())?;
            Ok(path.to_string_lossy().to_string())
        }
        "json" => {
            let path = app
                .dialog()
                .file()
                .set_file_name("sticky-notes-export.json")
                .blocking_save_file()
                .ok_or_else(|| "save cancelled".to_string())?;
            let path = path.as_path().ok_or_else(|| "invalid path".to_string())?;
            let json = serde_json::to_string_pretty(&notes).map_err(|e| e.to_string())?;
            fs::write(path, json).map_err(|e| e.to_string())?;
            Ok(path.to_string_lossy().to_string())
        }
        "md" => {
            // Export all as a single combined .md file.
            let path = app
                .dialog()
                .file()
                .set_file_name("sticky-notes-all.md")
                .blocking_save_file()
                .ok_or_else(|| "save cancelled".to_string())?;
            let path = path.as_path().ok_or_else(|| "invalid path".to_string())?;
            let mut combined = String::from("# Sticky Notes Export\n\n");
            for note in &notes {
                combined.push_str(&format!("## {}\n\n", note.meta.title));
                combined.push_str(&note.body);
                combined.push_str("\n\n---\n\n");
            }
            fs::write(path, combined).map_err(|e| e.to_string())?;
            Ok(path.to_string_lossy().to_string())
        }
        _ => Err(format!("unsupported format: {format}")),
    }
}

/// Export dir helper (used by tests / future folder export).
#[allow(dead_code)]
pub fn export_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = storage::data_dir(app)?.join("exports");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}
