import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../api";
import { createEditor, createSourceEditor, type EditorHandle } from "../editor";
import { renderMarkdown, renderMermaidDiagrams } from "../markdown";
import { resolveColor, type EditorMode, type Note } from "../types";

function debounce<T extends (...args: never[]) => void>(fn: T, ms: number) {
  let t: ReturnType<typeof setTimeout> | undefined;
  return (...args: Parameters<T>) => {
    clearTimeout(t);
    t = setTimeout(() => fn(...args), ms);
  };
}

export async function renderSticky(root: HTMLElement, noteId: string): Promise<void> {
  const win = getCurrentWindow();
  const note: Note = await api.getNote(noteId);
  const bg = resolveColor(note.color);

  root.innerHTML = `
    <div class="sticky" style="background:${bg}; opacity:${note.opacity}">
      <div class="sticky-titlebar" data-tauri-drag-region>
        <span class="color-dot" style="background:${bg}"></span>
        <input class="sticky-title" value="${escapeHtml(note.title)}" spellcheck="false" />
        <div class="sticky-tools">
          <button data-act="mode" title="Toggle Live/Source/Preview (Ctrl+E)">◐</button>
          <button data-act="pin" title="Always on top" class="${note.alwaysOnTop ? "on" : ""}">📌</button>
          <button data-act="open" title="Open in main window">↗</button>
          <button data-act="close" title="Hide (Esc)">✕</button>
        </div>
      </div>
      <div class="sticky-body">
        <div class="editor-host"></div>
        <div class="preview-pane" hidden></div>
      </div>
      <div class="resize-handle" data-tauri-drag-region></div>
    </div>`;

  const host = root.querySelector(".editor-host") as HTMLElement;
  const preview = root.querySelector(".preview-pane") as HTMLElement;
  const titleInput = root.querySelector(".sticky-title") as HTMLInputElement;
  let mode: EditorMode = "live";
  let editor: EditorHandle | null = null;

  const save = debounce(async (body: string) => {
    await api.updateNote(noteId, { body });
  }, 800);

  const saveTitle = debounce(async (title: string) => {
    await api.updateNote(noteId, { title });
  }, 500);

  function mountEditor() {
    editor?.destroy();
    host.innerHTML = "";
    preview.hidden = true;
    host.hidden = false;
    editor =
      mode === "source"
        ? createSourceEditor(host, note.body, save)
        : createEditor(host, note.body, save);
  }

  function showPreview() {
    editor?.destroy();
    editor = null;
    host.hidden = true;
    preview.hidden = false;
    preview.innerHTML = renderMarkdown(note.body);
    void renderMermaidDiagrams(preview);
  }

  function setMode(next: EditorMode) {
    mode = next;
    if (mode === "preview") showPreview();
    else mountEditor();
  }

  mountEditor();
  titleInput.addEventListener("input", () => saveTitle(titleInput.value));
  titleInput.addEventListener("dblclick", () => {
    void api.showMainWindow();
  });

  root.querySelector('[data-act="mode"]')?.addEventListener("click", () => {
    setMode(mode === "live" ? "source" : mode === "source" ? "preview" : "live");
  });
  root.querySelector('[data-act="pin"]')?.addEventListener("click", async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    const next = !btn.classList.contains("on");
    await api.setAlwaysOnTop(noteId, next);
    btn.classList.toggle("on", next);
  });
  root.querySelector('[data-act="open"]')?.addEventListener("click", () => {
    void api.showMainWindow();
  });
  root.querySelector('[data-act="close"]')?.addEventListener("click", () => {
    void win.hide();
  });

  // Wikilink clicks in preview.
  preview.addEventListener("click", (e) => {
    const a = (e.target as HTMLElement).closest("[data-wikilink]") as HTMLElement | null;
    if (a) {
      e.preventDefault();
      void openWikilink(a.dataset.wikilink ?? "");
    }
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") void win.hide();
    if (e.ctrlKey && e.key.toLowerCase() === "e") {
      e.preventDefault();
      setMode(mode === "live" ? "source" : mode === "source" ? "preview" : "live");
    }
    if (e.ctrlKey && e.key.toLowerCase() === "s") {
      e.preventDefault();
      if (editor) void api.updateNote(noteId, { body: editor.getValue() });
    }
  });

  // External updates (e.g. edited in main window).
  const { listen } = await import("@tauri-apps/api/event");
  await listen<Note>("note-updated", (event) => {
    if (event.payload.id === noteId && editor) {
      note.body = event.payload.body;
      note.title = event.payload.title;
      if (document.activeElement !== titleInput) titleInput.value = note.title;
      editor.setValue(note.body);
    }
  });
}

async function openWikilink(title: string): Promise<void> {
  const notes = await api.listNotes();
  const target = notes.find((n) => n.title.toLowerCase() === title.toLowerCase());
  if (target) {
    await api.openStickyWindow(target.id);
  } else {
    const created = await api.createNote(title, undefined, `# ${title}\n\n`);
    await api.openStickyWindow(created.id);
  }
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
}
