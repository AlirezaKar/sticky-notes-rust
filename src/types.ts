export interface Note {
  id: string;
  title: string;
  color: string;
  x: number;
  y: number;
  width: number;
  height: number;
  pinned: boolean;
  alwaysOnTop: boolean;
  opacity: number;
  createdAt: string;
  updatedAt: string;
  trashed: boolean;
  tags: string[];
  body: string;
}

export interface NoteMeta {
  id: string;
  title: string;
  color: string;
  x: number;
  y: number;
  width: number;
  height: number;
  pinned: boolean;
  alwaysOnTop: boolean;
  opacity: number;
  createdAt: string;
  updatedAt: string;
  trashed: boolean;
  tags: string[];
  preview: string;
}

export interface Settings {
  theme: "system" | "light" | "dark";
  autostart: boolean;
  closeToTray: boolean;
  defaultColor: string;
  defaultOpacity: number;
  hotkeysEnabled: boolean;
  openAllOnBoot: boolean;
  trashRetentionDays: number;
}

export interface Reminder {
  id: string;
  noteId: string;
  noteTitle: string;
  at: string;
  fired: boolean;
}

export interface VersionMeta {
  timestamp: string;
  path: string;
  size: number;
}

export interface SearchHit {
  id: string;
  title: string;
  color: string;
  snippet: string;
  updatedAt: string;
}

export type EditorMode = "live" | "source" | "preview";

export const COLOR_PRESETS: Record<string, { hex: string; label: string }> = {
  yellow: { hex: "#FFD966", label: "Yellow" },
  orange: { hex: "#FFB347", label: "Orange" },
  pink: { hex: "#FF8FAB", label: "Pink" },
  green: { hex: "#8CE99A", label: "Green" },
  blue: { hex: "#74C0FC", label: "Blue" },
  purple: { hex: "#D0BFFF", label: "Purple" },
  gray: { hex: "#CED4DA", label: "Gray" },
  white: { hex: "#FFFFFF", label: "White" },
};

export function resolveColor(color: string): string {
  return COLOR_PRESETS[color]?.hex ?? color;
}
