# Sticky Notes — Product Spec (v1)

> A fast, Obsidian-style sticky-notes app for Windows. Multiple floating notes,
> live Markdown editing, `.md` files on disk, system-tray + autostart + hotkeys.

| Decision | Choice |
|---|---|
| Stack | **Tauri 2** (Rust backend + WebView2 frontend) |
| Performance target | **Balanced** — small binary, fast startup, smooth typing |
| Window model | **Hybrid** — main dashboard window + one frameless window per sticky note |
| Storage | **`.md` files** (one per note, YAML frontmatter for metadata) |
| System integrations | Startup at login, system tray, global hotkeys, always-on-top pin, restore-on-boot |
| v1 extras | Search, dark/light, trash, versions, reminders, export, Markdown extras |
| Markdown extensions | Core + wikilinks `[[…]]`, math (KaTeX), Mermaid diagrams |
| Colors | 8 presets + custom hex picker |

---

## 1. Why Tauri 2 + Rust (vs alternatives)

| Option | Binary size | Memory | Notes |
|---|---|---|---|
| **Tauri 2 (chosen)** | ~8–12 MB | ~40–80 MB | Rust backend = fast file I/O, search, reminders. WebView2 UI = full HTML/CSS. |
| Electron | ~100–150 MB | ~150–300 MB | Heavier; Chromium bundled. Fine for prototyping, wasteful for a sticky-notes app. |
| Native WinUI 3 / WPF | ~5–15 MB | ~30–60 MB | Fastest, but no shared web UI; Markdown rendering + CodeMirror would need rework. |
| Flutter | ~20–40 MB | ~60–100 MB | Good, but desktop plugin ecosystem for tray/hotkey/autostart is thinner than Tauri's. |

**Verdict:** Tauri 2 gives near-native footprint with a modern web editor stack
(CodeMirror 6, markdown-it, KaTeX, Mermaid) and first-class plugins for
autostart, global shortcuts, notifications, store, dialog, and fs.

---

## 2. Window model — Hybrid

### 2.1 Main window (`label: "main"`)
- Dashboard: note list + search + editor + versions + trash + settings.
- Size: 1100×700 (min 800×500), decorated, resizable.
- Hidden to tray on close when `closeToTray` is on (default on).
- Opened via tray icon, hotkey `Ctrl+Alt+Q`, or double-click tray.

### 2.2 Sticky windows (`label: "note-<uuid>"`)
- One `WebviewWindow` per note. Frameless, transparent, rounded corners.
- Drag by title bar; resize by bottom-right handle (native resize region).
- Per-note `alwaysOnTop` toggle (pin icon). Opacity slider (40–100%).
- Double-click title bar → open in main window editor.
- `Esc` hides to tray (does not delete). Close button hides window (note persists).
- Position/size saved on move/resize and restored on boot ("pinned boot").

### 2.3 Lifecycle
```
create_note → save .md → open sticky window
close sticky window → hide (note stays on disk)
delete note → move to trash (trashed=true, window closed)
restore → un-trash, re-open window
```

---

## 3. Storage layout

```
%APPDATA%\StickyNotes\            (Windows; XDG on other OSes)
├── settings.json                 (theme, autostart, hotkeys, opacity…)
├── reminders.json                (scheduled reminders)
├── notes\
│   ├── <uuid>.md                 (one file per note)
│   └── …
├── versions\
│   └── <uuid>\
│       └── 2026-11-11T19-30-00-123Z.md
└── trash\                        (or trashed flag in frontmatter; see §6)
```

### 3.1 Note file format (`.md` + YAML frontmatter)

```markdown
---
id: "3f8a1c2e-…"
title: "Shopping list"
color: "#FFD966"          # preset name or custom hex
x: 120
y: 80
width: 340
height: 260
pinned: false             # always-on-top
alwaysOnTop: false
opacity: 1.0
createdAt: "2026-11-11T19:00:00Z"
updatedAt: "2026-11-11T19:05:00Z"
trashed: false
tags: ["personal"]
---

## Groceries
- [x] Milk
- [ ] Eggs
  - [ ] Free-range
- [ ] [[Pantry]]
```

- Body is **pure Markdown** — users can open the file in Obsidian/VS Code.
- Frontmatter is stripped from the editor view and managed by the app.
- File watcher (debounced) reloads notes edited externally.

### 3.2 Settings (`settings.json` via `tauri-plugin-store`)

```json
{
  "theme": "system | light | dark",
  "autostart": false,
  "closeToTray": true,
  "defaultColor": "yellow",
  "defaultOpacity": 1.0,
  "hotkeysEnabled": true,
  "openAllOnBoot": true,
  "trashRetentionDays": 30
}
```

---

## 4. Markdown engine

### 4.1 Editing — Obsidian-style Live Preview (CodeMirror 6)
- Type `## Heading` → `#` markers hide, text renders as H2.
- `**bold**`, `*italic*`, `` `code` ``, `~~strike~~`, `> quote`, links, images.
- Lists with **3 levels of indentation** (2 spaces per level).
- Task lists: `- [ ]` / `- [x]` with clickable checkboxes in preview mode.
- Toggle: `Ctrl+E` or toolbar button switches **Live Preview ↔ Source ↔ Preview**.

### 4.2 Rendering (preview mode, `markdown-it`)
- **Core:** headings, emphasis, code blocks, blockquotes, tables, lists,
  task lists, strikethrough, links, images, horizontal rules.
- **Wikilinks:** `[[Note Title]]` → clickable link that opens/focuses that note.
- **Math:** `$inline$` and `$$block$$` via KaTeX.
- **Mermaid:** ```` ```mermaid ```` blocks rendered as SVG diagrams.
- HTML is sanitized with DOMPurify before injection.

### 4.3 Supported syntax checklist
- [x] ATX headings `#`–`######`
- [x] Bold / italic / strikethrough / highlight
- [x] Inline + fenced code
- [x] Blockquotes (nested)
- [x] Ordered / unordered lists, 3-level nesting
- [x] Task lists with checkboxes
- [x] Tables (GFM)
- [x] Links, images, autolinks
- [x] Wikilinks `[[…]]`
- [x] Math `$…$` / `$$…$$`
- [x] Mermaid diagrams

---

## 5. Colors

- **8 presets:** Yellow `#FFD966`, Orange `#FFB347`, Pink `#FF8FAB`,
  Green `#8CE99A`, Blue `#74C0FC`, Purple `#D0BFFF`, Gray `#CED4DA`,
  White `#FFFFFF`.
- **Custom:** color-picker input → any hex, stored in frontmatter.
- Title bar + note background tinted; text contrast auto-adjusted for dark/light.

---

## 6. System integrations

| Feature | Implementation | Default |
|---|---|---|
| **Startup at login** | `tauri-plugin-autostart` (registry `Run` key) | Off (opt-in in Settings) |
| **System tray** | Tray icon + menu: Show, New Note, Toggle Stickies, Open Folder, Quit | Always |
| **Global hotkeys** | `tauri-plugin-global-shortcut` | `Ctrl+Alt+N` new, `Ctrl+Alt+S` toggle stickies, `Ctrl+Alt+Q` main window |
| **Always-on-top pin** | Per-note `alwaysOnTop` → `window.setAlwaysOnTop()` | Off |
| **Pinned boot** | Restore all sticky positions/sizes on launch | On |
| **Close to tray** | Main window close → hide; stickies close → hide | On |
| **Notifications** | `tauri-plugin-notification` for reminders | On |

---

## 7. v1 extras

### 7.1 Search
- Full-text across title + body (Rust-side scan of `.md` files).
- Live filter in dashboard; `Ctrl+K` focuses search.
- Results show snippet with match highlighted.

### 7.2 Dark / light
- `system | light | dark` setting; respects OS when `system`.
- CSS variables; sticky notes tint accordingly.

### 7.3 Trash
- Delete → `trashed: true` + window closed. Trash view in dashboard.
- Restore or permanently delete. Auto-purge after 30 days (configurable).

### 7.4 Versions
- Every save (debounced 1s) snapshots previous body to `versions/<id>/<ts>.md`.
- Keep last 50 per note. Versions panel: preview + restore.

### 7.5 Reminders
- Per-note reminder datetime → `reminders.json` + background checker (30s tick).
- Fires native notification + flashes sticky window + focuses it.
- List/cancel reminders in dashboard.

### 7.6 Export
- Single note: `.md`, `.html`, `.json`.
- All notes: `.zip` of `.md` files + `notes.json`.
- Print to PDF via browser print (dashboard editor).

---

## 8. Commands (Tauri invoke surface)

| Command | Args | Returns |
|---|---|---|
| `list_notes` | `{ includeTrashed?: boolean }` | `NoteMeta[]` |
| `get_note` | `{ id }` | `Note` |
| `create_note` | `{ title?, color?, body? }` | `Note` |
| `update_note` | `{ id, title?, body?, color?, … }` | `Note` |
| `delete_note` | `{ id }` | `void` (→ trash) |
| `restore_note` | `{ id }` | `Note` |
| `purge_note` | `{ id }` | `void` |
| `empty_trash` | — | `void` |
| `search_notes` | `{ query }` | `SearchHit[]` |
| `get_versions` | `{ id }` | `VersionMeta[]` |
| `restore_version` | `{ id, timestamp }` | `Note` |
| `export_note` | `{ id, format }` | `string` (saved path) |
| `export_all` | `{ format }` | `string` (saved path) |
| `open_sticky_window` | `{ id }` | `void` |
| `close_sticky_window` | `{ id }` | `void` |
| `toggle_all_stickies` | `{ visible?: boolean }` | `void` |
| `set_always_on_top` | `{ id, enabled }` | `void` |
| `set_autostart` | `{ enabled }` | `void` |
| `is_autostart` | — | `boolean` |
| `set_reminder` | `{ id, at: string \| null }` | `void` |
| `list_reminders` | — | `Reminder[]` |
| `get_settings` / `update_settings` | — | `Settings` |
| `open_notes_folder` | — | `void` |

---

## 9. Performance budget

| Metric | Target |
|---|---|
| Cold start (main window) | < 500 ms |
| New sticky window | < 150 ms |
| Typing latency (CodeMirror) | 60 fps, no jank on 10k-line notes |
| Idle memory (10 stickies) | < 150 MB total |
| Binary (release, stripped) | < 15 MB |

Techniques: debounced saves (800 ms), incremental search, lazy Mermaid/KaTeX
loading, single WebView per window (no Electron runtime).

---

## 10. Non-goals (v1)
- Cloud sync / collaboration (files are local; sync via OneDrive/Dropbox folder).
- Mobile apps (Windows-first; Tauri mobile is a future stretch).
- Plugins / themes marketplace.

---

## 11. Milestones
1. **M1 — Core:** storage, CRUD, sticky windows, tray, autostart.
2. **M2 — Editor:** CodeMirror live preview, markdown-it rendering, colors.
3. **M3 — Extras:** search, trash, versions, reminders, export, dark/light.
4. **M4 — Polish:** hotkeys, pinned boot, installer (NSIS/MSI), CI.
