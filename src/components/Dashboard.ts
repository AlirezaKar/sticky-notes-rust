import { api } from "../api";
import { createEditor, createSourceEditor, type EditorHandle } from "../editor";
import { renderMarkdown, renderMermaidDiagrams } from "../markdown";
import {
  COLOR_PRESETS,
  resolveColor,
  type EditorMode,
  type Note,
  type NoteMeta,
  type Settings,
  type VersionMeta,
} from "../types";

function debounce<T extends (...args: never[]) => void>(fn: T, ms: number) {
  let t: ReturnType<typeof setTimeout> | undefined;
  return (...args: Parameters<T>) => {
    clearTimeout(t);
    t = setTimeout(() => fn(...args), ms);
  };
}

export async function renderDashboard(root: HTMLElement): Promise<void> {
  let settings: Settings = await api.getSettings();
  let notes: NoteMeta[] = await api.listNotes();
  let trashed: NoteMeta[] = await api.listNotes(true).then((all) => all.filter((n) => n.trashed));
  let selectedId: string | null = notes[0]?.id ?? null;
  let selected: Note | null = selectedId ? await api.getNote(selectedId) : null;
  let mode: EditorMode = "live";
  let showTrash = false;
  let showSettings = false;
  let versions: VersionMeta[] = [];
  let editor: EditorHandle | null = null;
  let searchQuery = "";

  root.innerHTML = `
    <div class="dashboard">
      <aside class="sidebar">
        <div class="sidebar-header">
          <input class="search-input" placeholder="Search notes… (Ctrl+K)" />
          <div style="display:flex;gap:6px">
            <button class="btn primary" data-act="new" style="flex:1">+ New note</button>
            <button class="btn" data-act="trash" title="Trash">🗑</button>
            <button class="btn" data-act="settings" title="Settings">⚙</button>
          </div>
        </div>
        <div class="note-list"></div>
      </aside>
      <main class="main-pane">
        <div class="toolbar">
          <button class="btn" data-act="mode">◐ <span class="mode-label">Live</span></button>
          <button class="btn" data-act="pin" title="Always on top">📌</button>
          <button class="btn" data-act="show" title="Show sticky window">🗔 Show</button>
          <button class="btn" data-act="reminder" title="Set reminder">⏰</button>
          <div class="spacer"></div>
          <button class="btn" data-act="export-md">Export MD</button>
          <button class="btn" data-act="export-html">HTML</button>
          <button class="btn danger" data-act="delete">Delete</button>
        </div>
        <div class="editor-wrap">
          <section class="editor-pane">
            <input class="title-input" placeholder="Note title" />
            <div class="editor-host"></div>
            <div class="preview-pane" hidden></div>
          </section>
          <aside class="side-panel"></aside>
        </div>
      </main>
    </div>
    <div class="modal-root"></div>`;

  const listEl = root.querySelector(".note-list") as HTMLElement;
  const host = root.querySelector(".editor-host") as HTMLElement;
  const preview = root.querySelector(".preview-pane") as HTMLElement;
  const titleInput = root.querySelector(".title-input") as HTMLInputElement;
  const sidePanel = root.querySelector(".side-panel")!;
  const searchInput = root.querySelector(".search-input") as HTMLInputElement;
  const modeLabel = root.querySelector(".mode-label")!;

  function visibleNotes(): NoteMeta[] {
    const base = showTrash ? trashed : notes;
    if (!searchQuery.trim()) return base;
    const q = searchQuery.toLowerCase();
    return base.filter(
      (n) => n.title.toLowerCase().includes(q) || n.preview.toLowerCase().includes(q),
    );
  }

  function renderList() {
    const items = visibleNotes();
    listEl.innerHTML = items.length
      ? items
          .map(
            (n) => `
        <div class="note-item ${n.id === selectedId ? "active" : ""}" data-id="${n.id}">
          <div class="t"><span class="color-dot" style="background:${resolveColor(n.color)}"></span>${escapeHtml(n.title) || "Untitled"}</div>
          <div class="p">${escapeHtml(n.preview)}</div>
        </div>`,
          )
          .join("")
      : `<div class="empty">${showTrash ? "Trash is empty." : "No notes yet. Click + New note."}</div>`;
    listEl.querySelectorAll(".note-item").forEach((el) => {
      el.addEventListener("click", () => void select((el as HTMLElement).dataset.id!));
    });
  }

  async function select(id: string) {
    selectedId = id;
    selected = await api.getNote(id);
    titleInput.value = selected.title;
    versions = await api.getVersions(id);
    renderList();
    renderSidePanel();
    mountEditor();
  }

  const saveBody = debounce(async (body: string) => {
    if (!selectedId) return;
    selected = await api.updateNote(selectedId, { body });
    notes = await api.listNotes();
    renderList();
  }, 800);

  const saveTitle = debounce(async (title: string) => {
    if (!selectedId) return;
    selected = await api.updateNote(selectedId, { title });
    notes = await api.listNotes();
    renderList();
  }, 500);

  function mountEditor() {
    editor?.destroy();
    host.innerHTML = "";
    preview.hidden = true;
    host.hidden = false;
    if (!selected) return;
    editor =
      mode === "source"
        ? createSourceEditor(host, selected.body, saveBody)
        : createEditor(host, selected.body, saveBody);
  }

  function showPreview() {
    editor?.destroy();
    editor = null;
    host.hidden = true;
    preview.hidden = false;
    preview.innerHTML = selected ? renderMarkdown(selected.body) : "";
    void renderMermaidDiagrams(preview);
  }

  function setMode(next: EditorMode) {
    mode = next;
    modeLabel.textContent = next === "live" ? "Live" : next === "source" ? "Source" : "Preview";
    if (next === "preview") showPreview();
    else mountEditor();
  }

  function renderSidePanel() {
    if (!selected) {
      sidePanel.innerHTML = "";
      return;
    }
    const swatches = Object.entries(COLOR_PRESETS)
      .map(
        ([name, c]) =>
          `<button class="color-swatch ${selected!.color === name ? "active" : ""}" data-color="${name}" style="background:${c.hex}" title="${c.label}"></button>`,
      )
      .join("");
    sidePanel.innerHTML = `
      <div><h3>Color</h3><div class="color-grid">${swatches}</div>
        <div class="row"><input type="color" data-custom-color value="${toHexColor(selected.color)}" style="width:100%;height:32px" /></div></div>
      <div><h3>Versions (${versions.length})</h3>
        ${versions.length ? versions.map((v) => `<div class="version-item"><div>${escapeHtml(v.timestamp)}</div><button class="btn" data-restore="${escapeHtml(v.timestamp)}">Restore</button></div>`).join("") : "<div class='empty'>No versions yet.</div>"}
      </div>
      <div><h3>Reminders</h3><div class="row"><input type="datetime-local" data-reminder-at style="flex:1" /><button class="btn" data-reminder-set>Set</button></div></div>`;
    sidePanel.querySelectorAll("[data-color]").forEach((el) => {
      el.addEventListener("click", async () => {
        if (!selectedId) return;
        selected = await api.updateNote(selectedId, { color: (el as HTMLElement).dataset.color });
        notes = await api.listNotes();
        renderList();
        renderSidePanel();
      });
    });
    sidePanel.querySelector("[data-custom-color]")?.addEventListener("change", async (e) => {
      if (!selectedId) return;
      const hex = (e.target as HTMLInputElement).value;
      selected = await api.updateNote(selectedId, { color: hex });
      notes = await api.listNotes();
      renderList();
    });
    sidePanel.querySelectorAll("[data-restore]").forEach((el) => {
      el.addEventListener("click", async () => {
        if (!selectedId) return;
        selected = await api.restoreVersion(selectedId, (el as HTMLElement).dataset.restore!);
        versions = await api.getVersions(selectedId);
        mountEditor();
        renderSidePanel();
      });
    });
    sidePanel.querySelector("[data-reminder-set]")?.addEventListener("click", async () => {
      const input = sidePanel.querySelector("[data-reminder-at]") as HTMLInputElement;
      if (!selectedId || !input.value) return;
      await api.setReminder(selectedId, new Date(input.value).toISOString());
    });
  }

  titleInput.addEventListener("input", () => saveTitle(titleInput.value));
  searchInput.addEventListener("input", () => {
    searchQuery = searchInput.value;
    renderList();
  });

  root.querySelector('[data-act="new"]')?.addEventListener("click", async () => {
    const created = await api.createNote("Untitled", settings.defaultColor, "## Welcome\n\nStart typing…\n");
    notes = await api.listNotes();
    await select(created.id);
    await api.openStickyWindow(created.id);
  });
  root.querySelector('[data-act="mode"]')?.addEventListener("click", () => {
    setMode(mode === "live" ? "source" : mode === "source" ? "preview" : "live");
  });
  root.querySelector('[data-act="pin"]')?.addEventListener("click", async () => {
    if (!selectedId || !selected) return;
    const next = !selected.alwaysOnTop;
    selected = await api.updateNote(selectedId, { alwaysOnTop: next });
    await api.setAlwaysOnTop(selectedId, next);
  });
  root.querySelector('[data-act="show"]')?.addEventListener("click", async () => {
    if (selectedId) await api.openStickyWindow(selectedId);
  });
  root.querySelector('[data-act="reminder"]')?.addEventListener("click", () => {
    sidePanel.querySelector("[data-reminder-at]")?.scrollIntoView({ behavior: "smooth" });
  });
  root.querySelector('[data-act="export-md"]')?.addEventListener("click", async () => {
    if (selectedId) await api.exportNote(selectedId, "md");
  });
  root.querySelector('[data-act="export-html"]')?.addEventListener("click", async () => {
    if (selectedId) await api.exportNote(selectedId, "html");
  });
  root.querySelector('[data-act="delete"]')?.addEventListener("click", async () => {
    if (!selectedId) return;
    await api.deleteNote(selectedId);
    notes = await api.listNotes();
    trashed = await api.listNotes(true).then((all) => all.filter((n) => n.trashed));
    selectedId = notes[0]?.id ?? null;
    selected = selectedId ? await api.getNote(selectedId) : null;
    renderList();
    renderSidePanel();
    mountEditor();
  });
  root.querySelector('[data-act="trash"]')?.addEventListener("click", () => {
    showTrash = !showTrash;
    renderList();
  });
  root.querySelector('[data-act="settings"]')?.addEventListener("click", () => {
    showSettings = !showSettings;
    if (showSettings) openSettingsModal();
  });

  preview.addEventListener("click", (e) => {
    const a = (e.target as HTMLElement).closest("[data-wikilink]") as HTMLElement | null;
    if (a) {
      e.preventDefault();
      void openWikilink(a.dataset.wikilink ?? "");
    }
  });

  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.key.toLowerCase() === "k") {
      e.preventDefault();
      searchInput.focus();
    }
    if (e.ctrlKey && e.key.toLowerCase() === "e") {
      e.preventDefault();
      setMode(mode === "live" ? "source" : mode === "source" ? "preview" : "live");
    }
    if (e.ctrlKey && e.key.toLowerCase() === "s") {
      e.preventDefault();
      if (selectedId && editor) void api.updateNote(selectedId, { body: editor.getValue() });
    }
  });

  function openSettingsModal() {
    const modalRoot = root.querySelector(".modal-root")!;
    modalRoot.innerHTML = `
      <div class="modal-backdrop"><div class="modal">
        <h2>Settings</h2>
        <div class="row"><label>Theme</label>
          <select data-s="theme">
            <option value="system" ${settings.theme === "system" ? "selected" : ""}>System</option>
            <option value="light" ${settings.theme === "light" ? "selected" : ""}>Light</option>
            <option value="dark" ${settings.theme === "dark" ? "selected" : ""}>Dark</option>
          </select></div>
        <div class="row"><label>Start at Windows login</label><input type="checkbox" data-s="autostart" ${settings.autostart ? "checked" : ""} /></div>
        <div class="row"><label>Close to tray</label><input type="checkbox" data-s="closeToTray" ${settings.closeToTray ? "checked" : ""} /></div>
        <div class="row"><label>Open all stickies on boot</label><input type="checkbox" data-s="openAllOnBoot" ${settings.openAllOnBoot ? "checked" : ""} /></div>
        <div class="row"><label>Enable global hotkeys</label><input type="checkbox" data-s="hotkeysEnabled" ${settings.hotkeysEnabled ? "checked" : ""} /></div>
        <div class="row"><label>Default color</label>
          <select data-s="defaultColor">${Object.keys(COLOR_PRESETS).map((k) => `<option value="${k}" ${settings.defaultColor === k ? "selected" : ""}>${k}</option>`).join("")}</select></div>
        <div class="row"><button class="btn" data-s="folder">Open notes folder</button><button class="btn" data-s="export-all">Export all (.zip)</button><button class="btn" data-s="empty-trash">Empty trash</button></div>
        <div class="row" style="justify-content:flex-end"><button class="btn primary" data-s="close">Done</button></div>
      </div></div>`;
    const close = () => {
      modalRoot.innerHTML = "";
    };
    modalRoot.querySelector("[data-s='close']")?.addEventListener("click", close);
    modalRoot.querySelector(".modal-backdrop")?.addEventListener("click", (e) => {
      if (e.target === e.currentTarget) close();
    });
    modalRoot.querySelectorAll("[data-s]").forEach((el) => {
      el.addEventListener("change", async () => {
        const key = (el as HTMLElement).dataset.s!;
        if (key === "theme" || key === "defaultColor") {
          settings[key] = (el as HTMLSelectElement).value as never;
        } else if (key === "autostart") {
          const on = (el as HTMLInputElement).checked;
          await api.setAutostart(on);
          settings.autostart = on;
          return;
        } else {
          settings[key as keyof Settings] = (el as HTMLInputElement).checked as never;
        }
        settings = await api.updateSettings(settings);
        applyTheme(settings.theme);
      });
    });
    modalRoot.querySelector("[data-s='folder']")?.addEventListener("click", () => void api.openNotesFolder());
    modalRoot.querySelector("[data-s='export-all']")?.addEventListener("click", () => void api.exportAll("zip"));
    modalRoot.querySelector("[data-s='empty-trash']")?.addEventListener("click", async () => {
      await api.emptyTrash();
      trashed = [];
      renderList();
    });
  }

  renderList();
  renderSidePanel();
  mountEditor();
  applyTheme(settings.theme);
}

async function openWikilink(title: string): Promise<void> {
  const notes = await api.listNotes();
  const target = notes.find((n) => n.title.toLowerCase() === title.toLowerCase());
  if (target) await api.openStickyWindow(target.id);
  else {
    const created = await api.createNote(title, undefined, `# ${title}\n\n`);
    await api.openStickyWindow(created.id);
  }
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
}

function toHexColor(color: string): string {
  const hex = resolveColor(color);
  return /^#[0-9a-fA-F]{6}$/.test(hex) ? hex : "#FFD966";
}

export function applyTheme(theme: "system" | "light" | "dark"): void {
  const root = document.documentElement;
  if (theme === "system") {
    const dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.dataset.theme = dark ? "dark" : "light";
  } else {
    root.dataset.theme = theme;
  }
}
