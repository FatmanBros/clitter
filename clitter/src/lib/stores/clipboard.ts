import { writable, derived, get } from "svelte/store";
import type { ClipboardContent, Category } from "$lib/types";

// Clipboard history (volatile, from backend)
export const clipboardHistory = writable<ClipboardContent[]>([]);

// Selected category filter
export const selectedCategory = writable<Category | null>(null);

// Scroll offset for list view (how many items to skip)
export const listScrollOffset = writable<number>(0);

// Reset scroll offset when category changes
selectedCategory.subscribe(() => {
  listScrollOffset.set(0);
});

// All items matching current filter (not limited)
export const allFilteredHistory = derived(
  [clipboardHistory, selectedCategory],
  ([$history, $category]) => {
    if ($category === null) {
      return $history;
    }
    return $history.filter((item) => item.category === $category);
  }
);

// Visible items based on scroll offset (9 items max)
export const filteredHistory = derived(
  [allFilteredHistory, listScrollOffset],
  ([$history, $offset]) => {
    return $history.slice($offset, $offset + 9);
  }
);

// Check if can scroll up/down
export const canScrollUp = derived(listScrollOffset, ($offset) => $offset > 0);
export const canScrollDown = derived(
  [allFilteredHistory, listScrollOffset],
  ([$history, $offset]) => $offset + 9 < $history.length
);

// Scroll functions
export function scrollListUp() {
  listScrollOffset.update((offset) => Math.max(0, offset - 1));
}

export function scrollListDown() {
  const history = get(allFilteredHistory);
  listScrollOffset.update((offset) =>
    Math.min(Math.max(0, history.length - 9), offset + 1)
  );
}

export function scrollListPageUp() {
  listScrollOffset.update((offset) => Math.max(0, offset - 9));
}

export function scrollListPageDown() {
  const history = get(allFilteredHistory);
  listScrollOffset.update((offset) =>
    Math.min(Math.max(0, history.length - 9), offset + 9)
  );
}

// Category labels
export const categoryLabels: Record<Category, string> = {
  text: "Text",
  image: "Image",
  numeric: "Numeric",
  secure: "Secure",
};
