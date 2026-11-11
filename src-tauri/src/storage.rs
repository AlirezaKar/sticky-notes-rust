//! `.md` file storage: read/write notes with YAML frontmatter, trash, versions.
//!
//! Layout under the app data dir (`%APPDATA%\StickyNotes\` on Windows):
//!   notes/<uuid>.md, versions/<uuid>/<ts>.md, reminders.json

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use tauri::Manager;

use crate::models::{Frontmatter, Note};

pub struct Storage {
    pub notes_dir: PathBuf,
    pub versions_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        let notes_dir = data_dir.join("notes");
        let versions_dir = data_dir.join("versions");
        fs::create_dir_all(&notes_dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(&versions_dir).map_err(|e| e.to_string())?;
        Ok(Self {
            notes_dir,
            versions_dir,
            data_dir,
        })
    }

    fn note_path(&self, id: &str) -> PathBuf {
        self.notes_dir.join(format!("{id}.md"))
    }

    pub fn list_ids(&self) -> Result<Vec<String>, String> {
        let mut ids = Vec::new();
        for entry in fs::read_dir(&self.notes_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    ids.push(stem.to_string());
                }
            }
        }
        Ok(ids)
    }

    pub fn read_note(&self, id: &str) -> Result<Note, String> {
        let path = self.note_path(id);
        let raw = fs::read_to_string(&path).map_err(|e| format!("read {id}: {e}"))?;
        parse_note(&raw).map_err(|e| format!("parse {id}: {e}"))
    }

    pub fn write_note(&self, note: &Note) -> Result<(), String> {
        let path = self.note_path(&note.meta.id);
        let raw = serialize_note(note);
        // Atomic-ish write: temp file + rename.
        let tmp = path.with_extension("md.tmp");
        fs::write(&tmp, &raw).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_file(&self, id: &str) -> Result<(), String> {
        let path = self.note_path(id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Snapshot current body before overwrite (called on update).
    pub fn snapshot_version(&self, note: &Note) -> Result<(), String> {
        let dir = self.versions_dir.join(&note.meta.id);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let ts = Utc::now().format("%Y-%m-%dT%H-%M-%S-%3fZ").to_string();
        let raw = serialize_note(note);
        fs::write(dir.join(format!("{ts}.md")), raw).map_err(|e| e.to_string())?;
        self.prune_versions(&note.meta.id, 50)?;
        Ok(())
    }

    fn prune_versions(&self, id: &str, keep: usize) -> Result<(), String> {
        let dir = self.versions_dir.join(id);
        if !dir.exists() {
            return Ok(());
        }
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        files.sort();
        if files.len() > keep {
            for old in files.iter().take(files.len() - keep) {
                let _ = fs::remove_file(old);
            }
        }
        Ok(())
    }

    pub fn list_versions(&self, id: &str) -> Result<Vec<crate::models::VersionMeta>, String> {
        let dir = self.versions_dir.join(id);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let ts = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            out.push(crate::models::VersionMeta {
                timestamp: ts,
                path: path.to_string_lossy().to_string(),
                size,
            });
        }
        out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(out)
    }

    pub fn read_version(&self, id: &str, timestamp: &str) -> Result<Note, String> {
        let path = self.versions_dir.join(id).join(format!("{timestamp}.md"));
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        parse_note(&raw)
    }

    pub fn reminders_path(&self) -> PathBuf {
        self.data_dir.join("reminders.json")
    }

    pub fn load_reminders(&self) -> Result<Vec<crate::models::Reminder>, String> {
        let path = self.reminders_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_str(&raw).map_err(|e| e.to_string())
    }

    pub fn save_reminders(&self, reminders: &[crate::models::Reminder]) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(reminders).map_err(|e| e.to_string())?;
        fs::write(self.reminders_path(), raw).map_err(|e| e.to_string())
    }
}

/// Parse `---\n<yaml>\n---\n<body>` into a Note. Files without frontmatter
/// are treated as body-only with default metadata.
pub fn parse_note(raw: &str) -> Result<Note, String> {
    let trimmed = raw.trim_start_matches('\u{feff}');
    if let Some(rest) = trimmed.strip_prefix("---\n").or_else(|| trimmed.strip_prefix("---\r\n")) {
        if let Some(end) = rest.find("\n---\n").or_else(|| rest.find("\n---\r\n")) {
            let yaml = &rest[..end];
            let body_start = end + "\n---\n".len();
            let body = rest[body_start..].to_string();
            let meta: Frontmatter = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
            return Ok(Note { meta, body });
        }
    }
    Ok(Note {
        meta: Frontmatter::default(),
        body: raw.to_string(),
    })
}

pub fn serialize_note(note: &Note) -> String {
    let yaml = serde_yaml::to_string(&note.meta).unwrap_or_default();
    format!("---\n{}---\n{}", yaml, note.body)
}

/// Resolve a color preset name or hex to a hex string.
pub fn resolve_color(color: &str) -> String {
    match color {
        "yellow" => "#FFD966".to_string(),
        "orange" => "#FFB347".to_string(),
        "pink" => "#FF8FAB".to_string(),
        "green" => "#8CE99A".to_string(),
        "blue" => "#74C0FC".to_string(),
        "purple" => "#D0BFFF".to_string(),
        "gray" => "#CED4DA".to_string(),
        "white" => "#FFFFFF".to_string(),
        other => other.to_string(),
    }
}

/// Data dir helper: `%APPDATA%\StickyNotes` on Windows, `~/.local/share/StickyNotes` elsewhere.
pub fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let dir = base.join("StickyNotes");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Notes dir for "Open folder" action.
pub fn notes_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join("notes"))
}

/// Walk a dir (for export/search helpers).
pub fn walk_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p.to_path_buf());
        }
    }
    out
}
