import { writable } from "svelte/store";
import type { ViewMode, ContextMenuState } from "$lib/types";

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
