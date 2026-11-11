# Architecture

## Overview

```
┌─────────────────────────────────────────────────┐
│  Frontend (Vite + TS, single index.html)        │
│  src/main.ts routes by window label:            │
│    "main"      → Dashboard (list+search+editor) │
│    "note-<id>" → StickyNote (frameless window)  │
│  CodeMirror 6 (live preview) · markdown-it      │
│  KaTeX · Mermaid · DOMPurify                    │
└──────────────────────┬──────────────────────────┘
                       │ invoke (Tauri IPC)
┌──────────────────────▼──────────────────────────┐
│  Rust backend (src-tauri/src/)                  │
│  commands.rs   — 25 Tauri commands (SPEC §8)    │
│  storage.rs    — .md + frontmatter, versions    │
│  window_manager— hybrid windows, geometry       │
│  tray.rs       — tray icon + menu               │
│  hotkey.rs     — Ctrl+Alt+N/S/Q                 │
│  reminders.rs  — 30s checker + notifications    │
│  search.rs     — full-text scan                 │
│  export.rs     — md/html/json/zip via dialog    │
│  settings.rs   — tauri-plugin-store             │
└──────────────────────┬──────────────────────────┘
                       ▼
        %APPDATA%\StickyNotes\  (notes/*.md, versions/, reminders.json)
```

## Key decisions

- **One WebView per window.** Main + each sticky is a separate `WebviewWindow`.
  Sticky windows are cheap (~10–20 MB each) and independently draggable/resizable.
- **Files are the source of truth.** Every note is a `.md` file; the app is a
  thin editor over it. External edits are picked up on next read (add a watcher
  in v1.1 if needed).
- **Debounced saves (800 ms)** + atomic temp-file writes avoid corruption.
- **Versions** snapshot on every body change (keep 50/note).
- **Mermaid/KaTeX lazy-loaded** so startup stays fast; only loaded when a note
  actually uses them.

## Data flow (edit a sticky)

1. User types in CodeMirror → `onChange` → debounced `update_note` invoke.
2. Rust snapshots old body → writes new `.md` → emits `note-updated`.
3. Other windows listening on `note-updated` refresh their editor.

## Security

- All rendered HTML passes through DOMPurify.
- Tauri capabilities restrict fs/shell to app data + save dialogs.
- No network calls from the app (fully offline).
