import { writable, derived, get } from "svelte/store";
import type {
  WhiteboardState,
  WhiteboardItem,
  ShortcutMatch,
} from "$lib/types";

// Whiteboard state
export const whiteboardState = writable<WhiteboardState>({
  items: {},
  groups: {},
  rootItems: [],
});

// Currently dragged item
export const draggedItem = writable<string | null>(null);

// Current shortcut input (for incremental search)
export const shortcutInput = writable<string>("");

// Currently focused group (for hierarchical navigation)
export const focusedGroupId = writable<string | null>(null);

// Get all shortcuts for matching
export const allShortcuts = derived(
  [whiteboardState, focusedGroupId],
  ([$state, $focusedGroup]) => {
    const shortcuts: ShortcutMatch[] = [];

    // If focused on a group, only show its children's shortcuts
    if ($focusedGroup) {
      const group = $state.groups[$focusedGroup];
      if (group) {
        // Add child items
        for (const [id, item] of Object.entries($state.items)) {
          if (item.parentGroup === $focusedGroup && item.shortcut) {
            shortcuts.push({
              type: "item",
              id,
              shortcut: item.shortcut,
              name: getItemPreview(item),
              parentGroup: item.parentGroup,
            });
          }
        }
        // Add child groups
        for (const [id, g] of Object.entries($state.groups)) {
          if (g.parentGroup === $focusedGroup && g.shortcut) {
            shortcuts.push({
              type: "group",
              id,
              shortcut: g.shortcut,
              name: g.name,
              parentGroup: g.parentGroup,
            });
          }
        }
      }
    } else {
      // Show root level shortcuts
      for (const [id, item] of Object.entries($state.items)) {
        if (!item.parentGroup && item.shortcut) {
          shortcuts.push({
            type: "item",
            id,
            shortcut: item.shortcut,
            name: getItemPreview(item),
            parentGroup: null,
          });
        }
      }
      for (const [id, group] of Object.entries($state.groups)) {
        if (!group.parentGroup && group.shortcut) {
          shortcuts.push({
            type: "group",
            id,
            shortcut: group.shortcut,
            name: group.name,
            parentGroup: null,
          });
        }
      }
    }

    return shortcuts;
  }
);

// Matched shortcuts based on input
export const matchedShortcuts = derived(
  [allShortcuts, shortcutInput],
  ([$shortcuts, $input]) => {
    if (!$input) return $shortcuts;
    return $shortcuts.filter((s) =>
      s.shortcut.toLowerCase().startsWith($input.toLowerCase())
    );
  }
);

// Exact match (for Enter key action)
export const exactMatch = derived(
  [allShortcuts, shortcutInput],
  ([$shortcuts, $input]) => {
    if (!$input) return null;
    return (
      $shortcuts.find(
        (s) => s.shortcut.toLowerCase() === $input.toLowerCase()
      ) || null
    );
  }
);

function getItemPreview(item: WhiteboardItem): string {
  if (item.content.data.type === "text") {
    return item.content.data.preview.substring(0, 30);
  }
  return "[画像]";
}

// Get the full path of groups from root to current focused group
export const groupPath = derived(
  [whiteboardState, focusedGroupId],
  ([$state, $focusedGroup]) => {
    const path: Array<{ id: string; name: string; color: string | null }> = [];
    let currentId = $focusedGroup;

    while (currentId) {
      const group = $state.groups[currentId];
      if (group) {
        path.unshift({ id: group.id, name: group.name, color: group.color });
        currentId = group.parentGroup;
      } else {
        break;
      }
    }

    return path;
  }
);

// Actions
export function clearShortcutInput() {
  shortcutInput.set("");
}

export function appendToShortcutInput(char: string) {
  // Handle "/" to enter group
  if (char === "/") {
    const currentInput = get(shortcutInput);
    if (currentInput) {
      // Find matching group
      const shortcuts = get(allShortcuts);
      const match = shortcuts.find(
        (s) => s.type === "group" && s.shortcut.toLowerCase() === currentInput.toLowerCase()
      );
      if (match) {
        enterGroup(match.id);
        return;
      }
    }
    // If no match or empty input, ignore "/"
    return;
  }
  shortcutInput.update((s) => s + char);
}

export function backspaceShortcutInput() {
  shortcutInput.update((s) => s.slice(0, -1));
}

export function enterGroup(groupId: string) {
  const state = get(whiteboardState);
  const group = state.groups[groupId];
  if (group) {
    group.collapsed = false;
    whiteboardState.set(state);
    focusedGroupId.set(groupId);
    shortcutInput.set("");
  }
}

export function exitGroup() {
  const currentGroup = get(focusedGroupId);
  if (currentGroup) {
    const state = get(whiteboardState);
    const group = state.groups[currentGroup];
    if (group?.parentGroup) {
      focusedGroupId.set(group.parentGroup);
    } else {
      focusedGroupId.set(null);
    }
    shortcutInput.set("");
  }
}
