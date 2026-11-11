//! Global hotkeys: Ctrl+Alt+N (new), Ctrl+Alt+S (toggle stickies), Ctrl+Alt+Q (main).

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn register_hotkeys(app: &AppHandle) -> tauri::Result<()> {
    let settings = crate::settings::get_settings(app).unwrap_or_default();
    if !settings.hotkeys_enabled {
        return Ok(());
    }
    let new_note = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyN);
    let toggle = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyS);
    let main = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyQ);

    let handle = app.handle().clone();
    app.global_shortcut()
        .on_shortcut(new_note, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Ok(note) = crate::commands::create_note(handle.clone(), None, None, None) {
                    let _ = crate::window_manager::open_sticky_window(&handle, &note.meta.id);
                }
            }
        })?;
    let handle = app.handle().clone();
    app.global_shortcut()
        .on_shortcut(toggle, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = crate::window_manager::toggle_all_stickies(&handle, None);
            }
        })?;
    let handle = app.handle().clone();
    app.global_shortcut()
        .on_shortcut(main, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = crate::window_manager::show_main_window(&handle);
            }
        })?;
    Ok(())
}
