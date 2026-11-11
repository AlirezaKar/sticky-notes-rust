//! Version history helpers (snapshots live in `versions/<id>/<ts>.md`).
//!
//! Snapshotting itself happens in `storage::snapshot_version` on every body
//! update; this module exposes small helpers for the frontend.

use tauri::AppHandle;

use crate::models::VersionMeta;
use crate::storage::Storage;

#[allow(dead_code)]
pub fn list(app: &AppHandle, id: &str) -> Result<Vec<VersionMeta>, String> {
    Storage::new(crate::storage::data_dir(app)?)?.list_versions(id)
}
