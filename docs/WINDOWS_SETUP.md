# Windows setup

## Prerequisites
- Windows 10/11 (WebView2 is preinstalled; if missing, install
  [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)).
- [Rust stable](https://rustup.rs/) — `rustup default stable`
- [Node.js 20+](https://nodejs.org/)

## Dev
```powershell
npm install
npm run tauri dev
```

## Build installer
```powershell
npm run tauri build
```
Outputs (NSIS + MSI) land in `src-tauri\target\release\bundle\`.

## Enable startup at login
In the app: **Settings → Start at Windows login** (uses the registry `Run` key
via `tauri-plugin-autostart`). Verify with `shell:startup` or Task Manager →
Startup apps.

## Hotkeys
- `Ctrl+Alt+N` — new note · `Ctrl+Alt+S` — show/hide stickies · `Ctrl+Alt+Q` — main window
- If a hotkey conflicts, disable hotkeys in Settings and use the tray instead.

## Data & sync
Notes live in `%APPDATA%\StickyNotes\notes\`. To sync, point that folder at
OneDrive/Dropbox, or move it and symlink. Each note is a plain `.md` file you
can open in Obsidian/VS Code.

## Icons
Before first build, generate icons:
```powershell
npm run tauri icon src-tauri/icons/icon.png
```
(Place a 512×512 PNG at `src-tauri/icons/icon.png` first.)
