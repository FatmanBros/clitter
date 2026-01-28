import { writable, derived } from "svelte/store";
import type { ClipboardContent, Category } from "$lib/types";

// Clipboard history (volatile, from backend)
export const clipboardHistory = writable<ClipboardContent[]>([]);

// Selected category filter
export const selectedCategory = writable<Category | null>(null);

// Filtered history based on category
export const filteredHistory = derived(
  [clipboardHistory, selectedCategory],
  ([$history, $category]) => {
    if ($category === null) {
      return $history.slice(0, 5); // Show recent 5 items when no filter
    }
    return $history.filter((item) => item.category === $category);
  }
);

// Number badges for quick access
export const numberBadges = ["①", "②", "③", "④", "⑤"];

// Category icons
export const categoryIcons: Record<Category, string> = {
  text: "📋",
  image: "🖼️",
  numeric: "🔢",
  secure: "🔒",
};

// Category labels
export const categoryLabels: Record<Category, string> = {
  text: "文字",
  image: "画像",
  numeric: "数字",
  secure: "セキュア",
};
