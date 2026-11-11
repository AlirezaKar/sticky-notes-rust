import { getCurrentWindow } from "@tauri-apps/api/window";
import { renderDashboard } from "./components/Dashboard";
import { renderSticky } from "./components/StickyNote";

async function boot() {
  const root = document.getElementById("app")!;
  const label = getCurrentWindow().label;
  if (label.startsWith("note-")) {
    const noteId = label.slice("note-".length);
    await renderSticky(root, noteId);
  } else {
    await renderDashboard(root);
  }
}

boot().catch((err) => {
  document.getElementById("app")!.innerHTML =
    `<div class="empty">Failed to start: ${err}</div>`;
});
