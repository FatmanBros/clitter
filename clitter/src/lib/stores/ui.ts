import { writable } from "svelte/store";
import type { ViewMode, ContextMenuState } from "$lib/types";

// Theme settings
export type ThemeMode = "system" | "light" | "dark";

function loadTheme(): ThemeMode {
  try {
    const saved = localStorage.getItem("clitter-theme");
    if (saved === "system" || saved === "light" || saved === "dark") {
      return saved;
    }
  } catch (e) {
    console.error("Failed to load theme:", e);
  }
  return "system";
}

function saveTheme(theme: ThemeMode) {
  try {
    localStorage.setItem("clitter-theme", theme);
  } catch (e) {
    console.error("Failed to save theme:", e);
  }
}

export const themeMode = writable<ThemeMode>(loadTheme());

themeMode.subscribe((theme) => {
  saveTheme(theme);
});

export function setTheme(theme: ThemeMode) {
  themeMode.set(theme);
}

// Current view mode
export const currentView = writable<ViewMode>("list");

// Context menu state
export const contextMenu = writable<ContextMenuState>({
  show: false,
  x: 0,
  y: 0,
  target: null,
});

// Shortcut edit modal
export const shortcutEditModal = writable<{
  show: boolean;
  targetType: "item" | "group" | null;
  targetId: string | null;
  currentShortcut: string;
}>({
  show: false,
  targetType: null,
  targetId: null,
  currentShortcut: "",
});

// Settings modal
export const settingsModal = writable<{ show: boolean }>({ show: false });

// Window sizes
export interface WindowSizes {
  list: { width: number; height: number };
  whiteboard: { width: number; height: number };
}

const DEFAULT_SIZES: WindowSizes = {
  list: { width: 400, height: 500 },
  whiteboard: { width: 600, height: 700 },
};

function loadSizes(): WindowSizes {
  try {
    const saved = localStorage.getItem("clitter-window-sizes");
    if (saved) {
      return JSON.parse(saved);
    }
  } catch (e) {
    console.error("Failed to load window sizes:", e);
  }
  return DEFAULT_SIZES;
}

function saveSizes(sizes: WindowSizes) {
  try {
    localStorage.setItem("clitter-window-sizes", JSON.stringify(sizes));
  } catch (e) {
    console.error("Failed to save window sizes:", e);
  }
}

export const windowSizes = writable<WindowSizes>(loadSizes());

windowSizes.subscribe((sizes) => {
  saveSizes(sizes);
});

// Window positions (separate for each mode)
export interface WindowPosition {
  x: number;
  y: number;
}

export interface WindowPositions {
  list: WindowPosition;
  whiteboard: WindowPosition;
}

const DEFAULT_POSITIONS: WindowPositions = {
  list: { x: 100, y: 100 },
  whiteboard: { x: 100, y: 100 },
};

function loadPositions(): WindowPositions {
  try {
    const saved = localStorage.getItem("clitter-window-positions");
    if (saved) {
      return JSON.parse(saved);
    }
  } catch (e) {
    console.error("Failed to load window positions:", e);
  }
  return DEFAULT_POSITIONS;
}

function savePositions(positions: WindowPositions) {
  try {
    localStorage.setItem("clitter-window-positions", JSON.stringify(positions));
  } catch (e) {
    console.error("Failed to save window positions:", e);
  }
}

export const windowPositions = writable<WindowPositions>(loadPositions());

windowPositions.subscribe((positions) => {
  savePositions(positions);
});

export function updateWindowPosition(mode: "list" | "whiteboard", x: number, y: number) {
  windowPositions.update((positions) => ({
    ...positions,
    [mode]: { x, y },
  }));
}

// Actions
export function showContextMenu(
  x: number,
  y: number,
  target: ContextMenuState["target"]
) {
  contextMenu.set({ show: true, x, y, target });
}

export function hideContextMenu() {
  contextMenu.update((s) => ({ ...s, show: false }));
}

export function openShortcutEdit(
  type: "item" | "group",
  id: string,
  currentShortcut: string
) {
  shortcutEditModal.set({
    show: true,
    targetType: type,
    targetId: id,
    currentShortcut,
  });
}

export function closeShortcutEdit() {
  shortcutEditModal.update((s) => ({ ...s, show: false }));
}

export function openSettings() {
  settingsModal.set({ show: true });
}

export function closeSettings() {
  settingsModal.set({ show: false });
}

export function updateWindowSize(mode: "list" | "whiteboard", width: number, height: number) {
  windowSizes.update((sizes) => ({
    ...sizes,
    [mode]: { width, height },
  }));
}
