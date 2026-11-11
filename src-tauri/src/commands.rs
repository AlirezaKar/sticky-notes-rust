//! Tauri commands exposed to the frontend (see SPEC §8).

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Emitter, Manager};

use crate::models::{Note, NoteMeta, Reminder, SearchHit, Settings, VersionMeta};
use crate::storage::{self, Storage};
use crate::window_manager;

fn storage(app: &AppHandle) -> Result<Storage, String> {
    Storage::new(storage::data_dir(app)?)
}

#[tauri::command]
pub fn list_notes(app: AppHandle, include_trashed: Option<bool>) -> Result<Vec<NoteMeta>, String> {
    let store = storage(&app)?;
    let want_trashed = include_trashed.unwrap_or(false);
    let mut out = Vec::new();
    for id in store.list_ids()? {
        if let Ok(note) = store.read_note(&id) {
            if note.meta.trashed && !want_trashed {
                continue;
            }
            out.push(NoteMeta::from(&note));
        }
    }
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(out)
}

#[tauri::command]
pub fn get_note(app: AppHandle, id: String) -> Result<Note, String> {
    storage(&app)?.read_note(&id)
}

#[tauri::command]
pub fn create_note(
    app: AppHandle,
    title: Option<String>,
    color: Option<String>,
    body: Option<String>,
) -> Result<Note, String> {
    let store = storage(&app)?;
    let mut meta = crate::models::Frontmatter::default();
    if let Some(t) = title {
        meta.title = t;
    }
    if let Some(c) = color {
        meta.color = c;
    }
    // Cascade position so new stickies don't stack exactly.
    let count = store.list_ids().len() as i32;
    meta.x = 120 + (count % 6) * 40;
    meta.y = 80 + (count % 6) * 40;
    let note = Note {
        meta,
        body: body.unwrap_or_default(),
    };
    store.write_note(&note)?;
    let _ = app.emit("note-created", &note);
    Ok(note)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_note(
    app: AppHandle,
    id: String,
    title: Option<String>,
    body: Option<String>,
    color: Option<String>,
    x: Option<i32>,
    y: Option<i32>,
    width: Option<u32>,
    height: Option<u32>,
    pinned: Option<bool>,
    always_on_top: Option<bool>,
    opacity: Option<f64>,
    tags: Option<Vec<String>>,
) -> Result<Note, String> {
    let store = storage(&app)?;
    let mut note = store.read_note(&id)?;
    // Snapshot previous version when body changes.
    if body.is_some() {
        let _ = store.snapshot_version(&note);
    }
    if let Some(v) = title {
        note.meta.title = v;
    }
    if let Some(v) = body {
        note.meta.updated_at = Utc::now();
        note.body = v;
    }
    if let Some(v) = color {
        note.meta.color = v;
    }
    if let Some(v) = x {
        note.meta.x = v;
    }
    if let Some(v) = y {
        note.meta.y = v;
    }
    if let Some(v) = width {
        note.meta.width = v;
    }
    if let Some(v) = height {
        note.meta.height = v;
    }
    if let Some(v) = pinned {
        note.meta.pinned = v;
    }
    if let Some(v) = always_on_top {
        note.meta.always_on_top = v;
    }
    if let Some(v) = opacity {
        note.meta.opacity = v.clamp(0.4, 1.0);
    }
    if let Some(v) = tags {
        note.meta.tags = v;
    }
    store.write_note(&note)?;
    let _ = app.emit("note-updated", &note);
    Ok(note)
}

#[tauri::command]
pub fn delete_note(app: AppHandle, id: String) -> Result<(), String> {
    let store = storage(&app)?;
    let mut note = store.read_note(&id)?;
    note.meta.trashed = true;
    note.meta.updated_at = Utc::now();
    store.write_note(&note)?;
    window_manager::close_sticky_window(&app, &id);
    let _ = app.emit("note-deleted", &id);
    Ok(())
}

#[tauri::command]
pub fn restore_note(app: AppHandle, id: String) -> Result<Note, String> {
    let store = storage(&app)?;
    let mut note = store.read_note(&id)?;
    note.meta.trashed = false;
    note.meta.updated_at = Utc::now();
    store.write_note(&note)?;
    let _ = app.emit("note-restored", &note);
    Ok(note)
}

#[tauri::command]
pub fn purge_note(app: AppHandle, id: String) -> Result<(), String> {
    storage(&app)?.delete_file(&id)?;
    window_manager::close_sticky_window(&app, &id);
    let _ = app.emit("note-purged", &id);
    Ok(())
}

#[tauri::command]
pub fn empty_trash(app: AppHandle) -> Result<(), String> {
    let store = storage(&app)?;
    for id in store.list_ids()? {
        if let Ok(note) = store.read_note(&id) {
            if note.meta.trashed {
                store.delete_file(&id)?;
            }
        }
    }
    let _ = app.emit("trash-emptied", ());
    Ok(())
}

#[tauri::command]
pub fn search_notes(app: AppHandle, query: String) -> Result<Vec<SearchHit>, String> {
    crate::search::search(&app, &query)
}

#[tauri::command]
pub fn get_versions(app: AppHandle, id: String) -> Result<Vec<VersionMeta>, String> {
    storage(&app)?.list_versions(&id)
}

#[tauri::command]
pub fn restore_version(app: AppHandle, id: String, timestamp: String) -> Result<Note, String> {
    let store = storage(&app)?;
    let version = store.read_version(&id, &timestamp)?;
    let mut current = store.read_note(&id)?;
    let _ = store.snapshot_version(&current);
    current.body = version.body;
    current.meta.updated_at = Utc::now();
    store.write_note(&current)?;
    let _ = app.emit("note-updated", &current);
    Ok(current)
}

#[tauri::command]
pub fn export_note(app: AppHandle, id: String, format: String) -> Result<String, String> {
    crate::export::export_note(&app, &id, &format)
}

#[tauri::command]
pub fn export_all(app: AppHandle, format: String) -> Result<String, String> {
    crate::export::export_all(&app, &format)
}

#[tauri::command]
pub fn open_sticky_window(app: AppHandle, id: String) -> Result<(), String> {
    window_manager::open_sticky_window(&app, &id)
}

#[tauri::command]
pub fn close_sticky_window(app: AppHandle, id: String) -> Result<(), String> {
    window_manager::close_sticky_window(&app, &id);
    Ok(())
}

#[tauri::command]
pub fn toggle_all_stickies(app: AppHandle, visible: Option<bool>) -> Result<(), String> {
    window_manager::toggle_all_stickies(&app, visible)
}

#[tauri::command]
pub fn set_always_on_top(app: AppHandle, id: String, enabled: bool) -> Result<(), String> {
    window_manager::set_always_on_top(&app, &id, enabled)?;
    let store = storage(&app)?;
    let mut note = store.read_note(&id)?;
    note.meta.always_on_top = enabled;
    store.write_note(&note)?;
    Ok(())
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::AutoLaunchExt;
    let al = app.autolaunch();
    if enabled {
        al.enable().map_err(|e| e.to_string())?;
    } else {
        al.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn is_autostart(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::AutoLaunchExt;
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_reminder(app: AppHandle, id: String, at: Option<String>) -> Result<(), String> {
    let store = storage(&app)?;
    let note = store.read_note(&id)?;
    let mut reminders = store.load_reminders()?;
    reminders.retain(|r| r.note_id != id);
    if let Some(at_str) = at {
        let at_dt: DateTime<Utc> = at_str.parse().map_err(|e: chrono::ParseError| e.to_string())?;
        reminders.push(Reminder {
            id: uuid::Uuid::new_v4().to_string(),
            note_id: id.clone(),
            note_title: note.meta.title.clone(),
            at: at_dt,
            fired: false,
        });
    }
    store.save_reminders(&reminders)
}

#[tauri::command]
pub fn list_reminders(app: AppHandle) -> Result<Vec<Reminder>, String> {
    storage(&app)?.load_reminders()
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    crate::settings::get_settings(&app)
}

#[tauri::command]
pub fn update_settings(app: AppHandle, settings: Settings) -> Result<Settings, String> {
    crate::settings::update_settings(&app, settings)
}

#[tauri::command]
pub fn open_notes_folder(app: AppHandle) -> Result<(), String> {
    let dir = storage::notes_dir(&app)?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Show/focus the main dashboard window (used by tray + hotkeys).
#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    window_manager::show_main_window(&app)
}
