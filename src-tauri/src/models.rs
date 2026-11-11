//! Data models: notes, frontmatter, settings, reminders, versions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// YAML frontmatter stored at the top of every `.md` note file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frontmatter {
    pub id: String,
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub trashed: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: default_title(),
            color: default_color(),
            x: 120,
            y: 80,
            width: default_width(),
            height: default_height(),
            pinned: false,
            always_on_top: false,
            opacity: default_opacity(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            trashed: false,
            tags: Vec::new(),
        }
    }
}

fn default_title() -> String {
    "Untitled".to_string()
}
fn default_color() -> String {
    "yellow".to_string()
}
fn default_width() -> u32 {
    340
}
fn default_height() -> u32 {
    260
}
fn default_opacity() -> f64 {
    1.0
}
fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Full note: frontmatter + markdown body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(flatten)]
    pub meta: Frontmatter,
    pub body: String,
}

/// Lightweight metadata for lists (no body).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteMeta {
    pub id: String,
    pub title: String,
    pub color: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub pinned: bool,
    pub always_on_top: bool,
    pub opacity: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub trashed: bool,
    pub tags: Vec<String>,
    /// First ~120 chars of body for list previews.
    pub preview: String,
}

impl From<&Note> for NoteMeta {
    fn from(n: &Note) -> Self {
        let preview: String = n
            .body
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("")
            .chars()
            .take(120)
            .collect();
        Self {
            id: n.meta.id.clone(),
            title: n.meta.title.clone(),
            color: n.meta.color.clone(),
            x: n.meta.x,
            y: n.meta.y,
            width: n.meta.width,
            height: n.meta.height,
            pinned: n.meta.pinned,
            always_on_top: n.meta.always_on_top,
            opacity: n.meta.opacity,
            created_at: n.meta.created_at,
            updated_at: n.meta.updated_at,
            trashed: n.meta.trashed,
            tags: n.meta.tags.clone(),
            preview,
        }
    }
}

/// App-wide settings (persisted via tauri-plugin-store).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_theme")]
    pub theme: String, // "system" | "light" | "dark"
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    #[serde(default = "default_color")]
    pub default_color: String,
    #[serde(default = "default_opacity")]
    pub default_opacity: f64,
    #[serde(default = "default_true")]
    pub hotkeys_enabled: bool,
    #[serde(default = "default_true")]
    pub open_all_on_boot: bool,
    #[serde(default = "default_retention")]
    pub trash_retention_days: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            autostart: false,
            close_to_tray: true,
            default_color: default_color(),
            default_opacity: 1.0,
            hotkeys_enabled: true,
            open_all_on_boot: true,
            trash_retention_days: 30,
        }
    }
}

fn default_theme() -> String {
    "system".to_string()
}
fn default_true() -> bool {
    true
}
fn default_retention() -> u32 {
    30
}

/// A scheduled reminder for a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub note_id: String,
    pub note_title: String,
    pub at: DateTime<Utc>,
    pub fired: bool,
}

/// Version snapshot metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionMeta {
    pub timestamp: String,
    pub path: String,
    pub size: u64,
}

/// Search hit with snippet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub color: String,
    pub snippet: String,
    pub updated_at: DateTime<Utc>,
}
