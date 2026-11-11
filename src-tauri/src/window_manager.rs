//! Hybrid window management: main dashboard + per-note sticky windows.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

pub fn sticky_label(id: &str) -> String {
    format!("note-{id}")
}

pub fn note_id_from_label(label: &str) -> Option<String> {
    label.strip_prefix("note-").map(|s| s.to_string())
}

pub fn open_sticky_window(app: &AppHandle, id: &str) -> Result<(), String> {
    let label = sticky_label(id);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    let store = crate::storage::Storage::new(crate::storage::data_dir(app)?)?;
    let note = store.read_note(id)?;
    let title = if note.meta.title.is_empty() {
        "Sticky Note".to_string()
    } else {
        note.meta.title.clone()
    };
    let win = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(note.meta.width as f64, note.meta.height as f64)
        .min_inner_size(220.0, 160.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(note.meta.always_on_top)
        .resizable(true)
        .visible(true)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: note.meta.x,
        y: note.meta.y,
    }));
    let _ = win.set_opacity(note.meta.opacity);
    // Persist position/size on close/move via event handler in lib.rs setup.
    Ok(())
}

pub fn close_sticky_window(app: &AppHandle, id: &str) {
    let label = sticky_label(id);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.hide();
    }
}

pub fn toggle_all_stickies(app: &AppHandle, visible: Option<bool>) -> Result<(), String> {
    let store = crate::storage::Storage::new(crate::storage::data_dir(app)?)?;
    let mut any_visible = false;
    for id in store.list_ids()? {
        if let Ok(note) = store.read_note(&id) {
            if note.meta.trashed {
                continue;
            }
            let label = sticky_label(&id);
            if let Some(win) = app.get_webview_window(&label) {
                if win.is_visible().unwrap_or(false) {
                    any_visible = true;
                }
            }
        }
    }
    let show = visible.unwrap_or(!any_visible);
    for id in store.list_ids()? {
        if let Ok(note) = store.read_note(&id) {
            if note.meta.trashed {
                continue;
            }
            let label = sticky_label(&id);
            if show {
                open_sticky_window(app, &id)?;
            } else if let Some(win) = app.get_webview_window(&label) {
                let _ = win.hide();
            }
        }
    }
    Ok(())
}

pub fn set_always_on_top(app: &AppHandle, id: &str, enabled: bool) -> Result<(), String> {
    let label = sticky_label(id);
    if let Some(win) = app.get_webview_window(&label) {
        win.set_always_on_top(enabled)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
    Ok(())
}

/// Save window geometry back to the note file (called on move/resize/close).
pub fn persist_geometry(app: &AppHandle, label: &str) {
    let Some(id) = note_id_from_label(label) else {
        return;
    };
    let Some(win) = app.get_webview_window(label) else {
        return;
    };
    let Ok(pos) = win.outer_position() else {
        return;
    };
    let Ok(size) = win.outer_size() else {
        return;
    };
    let Ok(dir) = crate::storage::data_dir(app) else {
        return;
    };
    if let Ok(store) = crate::storage::Storage::new(dir) {
        if let Ok(mut note) = store.read_note(&id) {
            note.meta.x = pos.x;
            note.meta.y = pos.y;
            note.meta.width = size.width;
            note.meta.height = size.height;
            let _ = store.write_note(&note);
        }
    }
}

/// Handle window events: close-to-tray + geometry persistence.
pub fn handle_window_event(app: &AppHandle, window: &tauri::Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            let label = window.label();
            if label == "main" {
                let settings = crate::settings::get_settings(app).unwrap_or_default();
                if settings.close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            } else if note_id_from_label(label).is_some() {
                // Sticky close = hide (note persists on disk).
                api.prevent_close();
                persist_geometry(app, label);
                let _ = window.hide();
            }
        }
        WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
            persist_geometry(app, window.label());
        }
        _ => {}
    }
}
