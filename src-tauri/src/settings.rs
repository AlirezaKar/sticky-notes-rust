//! Settings persistence via tauri-plugin-store (`settings.json`).

use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::models::Settings;

const STORE_FILE: &str = "settings.json";
const KEY: &str = "settings";

pub fn get_settings(app: &AppHandle) -> Result<Settings, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let value = store.get(KEY);
    match value {
        Some(v) => serde_json::from_value(v).map_err(|e| e.to_string()),
        None => Ok(Settings::default()),
    }
}

pub fn update_settings(app: &AppHandle, settings: Settings) -> Result<Settings, String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    store.set(KEY, value);
    store.save().map_err(|e| e.to_string())?;
    Ok(settings)
}
