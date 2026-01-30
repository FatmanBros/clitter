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

// History of exited groups (stack for right arrow navigation)
export const exitedGroupHistory = writable<string[]>([]);

// For backward compatibility
export const lastExitedGroupId = derived(exitedGroupHistory, ($history) =>
  $history.length > 0 ? $history[$history.length - 1] : null
);

// Get all shortcuts for matching
export const allShortcuts = derived(
  [whiteboardState, focusedGroupId],
  ([$state, $focusedGroup]) => {
    const shortcuts: ShortcutMatch[] = [];

    // If focused on a group, only show its children's shortcuts
    if ($focusedGroup) {
      const group = $state.groups[$focusedGroup];
      if (group) {
        // Add child items (include items with shortcut OR label)
        for (const [id, item] of Object.entries($state.items)) {
          if (item.parentGroup === $focusedGroup && (item.shortcut || item.label)) {
            shortcuts.push({
              type: "item",
              id,
              shortcut: item.shortcut || "",
              name: getItemPreview(item),
              label: item.label,
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
              label: null,
              parentGroup: g.parentGroup,
            });
          }
        }
      }
    } else {
      // Show root level shortcuts
      for (const [id, item] of Object.entries($state.items)) {
        if (!item.parentGroup && (item.shortcut || item.label)) {
          shortcuts.push({
            type: "item",
            id,
            shortcut: item.shortcut || "",
            name: getItemPreview(item),
            label: item.label,
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
            label: null,
            parentGroup: null,
          });
        }
      }
    }

    return shortcuts;
  }
);

// Matched shortcuts based on input
// Priority: name/label matches first, then alias matches
export const matchedShortcuts = derived(
  [allShortcuts, shortcutInput],
  ([$shortcuts, $input]) => {
    if (!$input) return $shortcuts;
    const inputLower = $input.toLowerCase();

    // Separate matches by type: name/label matches vs alias matches
    const nameMatches: ShortcutMatch[] = [];
    const aliasMatches: ShortcutMatch[] = [];

    for (const s of $shortcuts) {
      const nameMatch = (s.type === "group" && s.name.toLowerCase().startsWith(inputLower)) ||
                        (s.type === "item" && s.label && s.label.toLowerCase().startsWith(inputLower));
      const aliasMatch = s.shortcut && s.shortcut.toLowerCase().startsWith(inputLower);

      if (nameMatch) {
        nameMatches.push(s);
      } else if (aliasMatch) {
        aliasMatches.push(s);
      }
    }

    // Name matches take priority over alias matches
    return [...nameMatches, ...aliasMatches];
  }
);

// Set of matched IDs for highlighting (empty means no filtering active)
export const matchedIds = derived(
  [matchedShortcuts, shortcutInput],
  ([$matched, $input]) => {
    if (!$input) return new Set<string>();
    return new Set($matched.map((m) => m.id));
  }
);

// Whether filtering is active
export const isFiltering = derived(shortcutInput, ($input) => $input.length > 0);

// Exact match (for Enter key action)
export const exactMatch = derived(
  [allShortcuts, shortcutInput],
  ([$shortcuts, $input]) => {
    if (!$input) return null;
    const inputLower = $input.toLowerCase();

    // First try to match by shortcut
    let match = $shortcuts.find(
      (s) => s.shortcut && s.shortcut.toLowerCase() === inputLower
    );

    // If no shortcut match, try to match groups by name
    if (!match) {
      match = $shortcuts.find(
        (s) => s.type === "group" && s.name.toLowerCase() === inputLower
      );
    }

    // If no match yet, try to match items by label
    if (!match) {
      match = $shortcuts.find(
        (s) => s.type === "item" && s.label && s.label.toLowerCase() === inputLower
      );
    }

    return match || null;
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
      const shortcuts = get(allShortcuts);
      const inputLower = currentInput.toLowerCase();

      // First try to match by shortcut
      let match = shortcuts.find(
        (s) => s.type === "group" && s.shortcut.toLowerCase() === inputLower
      );

      // If no shortcut match, try to match by group name
      if (!match) {
        match = shortcuts.find(
          (s) => s.type === "group" && s.name.toLowerCase() === inputLower
        );
      }

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

export function enterGroup(groupId: string, fromHistory: boolean = false) {
  const state = get(whiteboardState);
  const group = state.groups[groupId];
  if (group) {
    group.collapsed = false;
    whiteboardState.set(state);
    focusedGroupId.set(groupId);
    shortcutInput.set("");
    // Clear forward history when entering a group directly (not from right arrow)
    if (!fromHistory) {
      exitedGroupHistory.set([]);
    }
  }
}

export function exitGroup() {
  const currentGroup = get(focusedGroupId);
  if (currentGroup) {
    // Push to history stack so we can re-enter with right arrow
    exitedGroupHistory.update((history) => [...history, currentGroup]);

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

export function reenterLastGroup() {
  const history = get(exitedGroupHistory);
  if (history.length > 0) {
    const lastGroup = history[history.length - 1];
    const state = get(whiteboardState);
    if (state.groups[lastGroup]) {
      // Pop from history
      exitedGroupHistory.update((h) => h.slice(0, -1));
      // Enter group without clearing history
      enterGroup(lastGroup, true);
    }
  }
}
