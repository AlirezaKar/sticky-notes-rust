//! Sticky Notes — Tauri 2 backend entry point.

mod commands;
mod export;
mod hotkey;
mod models;
mod reminders;
mod search;
mod settings;
mod storage;
mod tray;
mod versions;
mod window_manager;

use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_notes,
            commands::get_note,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::restore_note,
            commands::purge_note,
            commands::empty_trash,
            commands::search_notes,
            commands::get_versions,
            commands::restore_version,
            commands::export_note,
            commands::export_all,
            commands::open_sticky_window,
            commands::close_sticky_window,
            commands::toggle_all_stickies,
            commands::set_always_on_top,
            commands::set_autostart,
            commands::is_autostart,
            commands::set_reminder,
            commands::list_reminders,
            commands::get_settings,
            commands::update_settings,
            commands::open_notes_folder,
            commands::show_main_window,
        ])
        .setup(|app| {
            // Ensure data dirs exist.
            let _ = storage::data_dir(app.handle());
            // Tray + hotkeys + reminders.
            tray::setup_tray(app)?;
            hotkey::register_hotkeys(app.handle())?;
            reminders::spawn_checker(app.handle().clone());
            // Pinned boot: restore all stickies (unless --autostart + setting off).
            let settings = settings::get_settings(app.handle()).unwrap_or_default();
            if settings.open_all_on_boot {
                if let Ok(dir) = storage::data_dir(app.handle()) {
                    if let Ok(store) = storage::Storage::new(dir) {
                        for id in store.list_ids().unwrap_or_default() {
                            if let Ok(note) = store.read_note(&id) {
                                if !note.meta.trashed {
                                    let _ = window_manager::open_sticky_window(app.handle(), &id);
                                }
                            }
                        }
                    }
                }
            }
            // Show main window on first launch / when tray clicked.
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                window_manager::handle_window_event(window.app_handle(), window, event);
            } else if matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
                window_manager::handle_window_event(window.app_handle(), window, event);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Sticky Notes");
}
