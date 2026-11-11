import { invoke } from "@tauri-apps/api/core";
import type { Note, NoteMeta, Reminder, SearchHit, Settings, VersionMeta } from "./types";

export const api = {
  listNotes: (includeTrashed = false) =>
    invoke<NoteMeta[]>("list_notes", { includeTrashed }),
  getNote: (id: string) => invoke<Note>("get_note", { id }),
  createNote: (title?: string, color?: string, body?: string) =>
    invoke<Note>("create_note", { title, color, body }),
  updateNote: (id: string, patch: Partial<Note>) =>
    invoke<Note>("update_note", { id, ...patch }),
  deleteNote: (id: string) => invoke<void>("delete_note", { id }),
  restoreNote: (id: string) => invoke<Note>("restore_note", { id }),
  purgeNote: (id: string) => invoke<void>("purge_note", { id }),
  emptyTrash: () => invoke<void>("empty_trash"),
  searchNotes: (query: string) => invoke<SearchHit[]>("search_notes", { query }),
  getVersions: (id: string) => invoke<VersionMeta[]>("get_versions", { id }),
  restoreVersion: (id: string, timestamp: string) =>
    invoke<Note>("restore_version", { id, timestamp }),
  exportNote: (id: string, format: "md" | "html" | "json") =>
    invoke<string>("export_note", { id, format }),
  exportAll: (format: "md" | "json" | "zip") => invoke<string>("export_all", { format }),
  openStickyWindow: (id: string) => invoke<void>("open_sticky_window", { id }),
  closeStickyWindow: (id: string) => invoke<void>("close_sticky_window", { id }),
  toggleAllStickies: (visible?: boolean) =>
    invoke<void>("toggle_all_stickies", { visible }),
  setAlwaysOnTop: (id: string, enabled: boolean) =>
    invoke<void>("set_always_on_top", { id, enabled }),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  isAutostart: () => invoke<boolean>("is_autostart"),
  setReminder: (id: string, at: string | null) =>
    invoke<void>("set_reminder", { id, at }),
  listReminders: () => invoke<Reminder[]>("list_reminders"),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) =>
    invoke<Settings>("update_settings", { settings }),
  openNotesFolder: () => invoke<void>("open_notes_folder"),
  showMainWindow: () => invoke<void>("show_main_window"),
};
