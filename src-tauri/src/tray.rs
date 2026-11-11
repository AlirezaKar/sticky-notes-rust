//! System-tray icon + menu (Show, New, Toggle stickies, Open folder, Quit).

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

const MENU_SHOW: &str = "show";
const MENU_NEW: &str = "new";
const MENU_TOGGLE: &str = "toggle";
const MENU_FOLDER: &str = "folder";
const MENU_QUIT: &str = "quit";

pub fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, MENU_SHOW, "Show Sticky Notes", true, None::<&str>)?;
    let new = MenuItem::with_id(app, MENU_NEW, "New Note  (Ctrl+Alt+N)", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, MENU_TOGGLE, "Show/Hide All Stickies", true, None::<&str>)?;
    let folder = MenuItem::with_id(app, MENU_FOLDER, "Open Notes Folder", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &new, &toggle, &folder, &quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Sticky Notes")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_SHOW => {
                let _ = crate::window_manager::show_main_window(app);
            }
            MENU_NEW => {
                let _ = crate::commands::create_note(app.clone(), None, None, None)
                    .map(|note| {
                        let _ = crate::window_manager::open_sticky_window(app, &note.meta.id);
                    });
            }
            MENU_TOGGLE => {
                let _ = crate::window_manager::toggle_all_stickies(app, None);
            }
            MENU_FOLDER => {
                let _ = crate::commands::open_notes_folder(app.clone());
            }
            MENU_QUIT => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = crate::window_manager::show_main_window(app);
            }
        })
        .build(app)?;
    Ok(())
}

#[allow(dead_code)]
pub fn handle(app: &AppHandle) -> tauri::Result<()> {
    let _ = app;
    Ok(())
}
