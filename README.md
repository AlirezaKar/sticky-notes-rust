# Sticky Notes

A fast, Obsidian-style sticky-notes app for Windows — built with **Tauri 2 + Rust**.
Multiple floating notes, live Markdown editing, `.md` files on disk, tray + autostart + hotkeys.

> Full product spec: [SPEC.md](./SPEC.md) · Architecture: [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) · Windows setup: [docs/WINDOWS_SETUP.md](./docs/WINDOWS_SETUP.md)

## Features

- **Hybrid windows** — main dashboard + one frameless sticky window per note
- **Live Markdown** — type `## Heading` and the `#`s hide (Obsidian-style); full GFM + tables + checklists + 3-level lists
- **Extensions** — wikilinks `[[Note]]`, math (KaTeX), Mermaid diagrams
- **Colors** — 8 presets + custom hex picker
- **`.md` files** — one file per note with YAML frontmatter; open them in Obsidian/VS Code
- **System** — start at Windows login, system tray, global hotkeys (`Ctrl+Alt+N/S/Q`), always-on-top pin, restore positions on boot
- **Extras** — full-text search, dark/light, trash, version history, reminders, export (MD/HTML/JSON/ZIP)

## Quick start (Windows)

Prerequisites: [Rust](https://rustup.rs/) (stable), [Node.js 20+](https://nodejs.org/), WebView2 (preinstalled on Windows 10/11).

```powershell
npm install
npm run tauri dev      # dev mode
npm run tauri build    # release installer (NSIS + MSI)
```

The installer lands in `src-tauri/target/release/bundle/`.

## Data location

`%APPDATA%\StickyNotes\` — `notes\*.md`, `settings.json`, `reminders.json`, `versions\`.
Point the folder at OneDrive/Dropbox to sync for free.

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Alt+N` | New note |
| `Ctrl+Alt+S` | Show/hide all stickies |
| `Ctrl+Alt+Q` | Toggle main window |
| `Ctrl+K` | Search (main window) |
| `Ctrl+E` | Toggle Live Preview / Source / Preview |
| `Ctrl+S` | Save now |
| `Esc` | Hide sticky to tray |

## Project layout

```
src-tauri/   Rust backend (storage, tray, hotkeys, autostart, reminders…)
src/         Frontend (Vite + TypeScript, CodeMirror 6, markdown-it)
SPEC.md      Product spec
docs/        Architecture + Windows setup guides
```
