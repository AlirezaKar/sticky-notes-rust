//! Reminder checker: ticks every 30s, fires native notifications for due reminders.

use std::time::Duration;

use chrono::Utc;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::storage::Storage;

pub fn spawn_checker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            check_once(&app);
        }
    });
}

fn check_once(app: &AppHandle) {
    let Ok(dir) = crate::storage::data_dir(app) else {
        return;
    };
    let Ok(store) = Storage::new(dir) else {
        return;
    };
    let Ok(mut reminders) = store.load_reminders() else {
        return;
    };
    let now = Utc::now();
    let mut changed = false;
    for r in reminders.iter_mut() {
        if !r.fired && r.at <= now {
            r.fired = true;
            changed = true;
            let _ = app
                .notification()
                .builder()
                .title("Sticky Notes Reminder")
                .body(format!("⏰ {}", r.note_title))
                .show();
            // Flash the sticky window.
            let label = crate::window_manager::sticky_label(&r.note_id);
            if let Some(win) = app.get_webview_window(&label) {
                let _ = win.show();
                let _ = win.set_focus();
            } else {
                let _ = crate::window_manager::open_sticky_window(app, &r.note_id);
            }
        }
    }
    if changed {
        let _ = store.save_reminders(&reminders);
    }
}
