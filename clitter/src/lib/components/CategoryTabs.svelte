<script lang="ts">
  import { selectedCategory, categoryLabels, categoryIcons } from "$lib/stores/clipboard";
  import type { Category } from "$lib/types";

  const categories: (Category | null)[] = ["image", "text", "numeric", "secure", null];
  const arrows: Record<string, string> = {
    image: "←",
    text: "↓",
    numeric: "→",
    secure: "",
  };
</script>

<div class="flex gap-1 px-4 py-2 border-b border-gray-200 bg-gray-50">
  {#each categories as cat}
    <button
      class="px-3 py-1.5 rounded-md text-sm font-medium transition-colors
        {$selectedCategory === cat
          ? 'bg-primary-500 text-white'
          : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}"
      on:click={() => selectedCategory.set(cat)}
    >
      {#if cat === null}
        全て
      {:else}
        {arrows[cat]} {categoryIcons[cat]} {categoryLabels[cat]}
      {/if}
    </button>
  {/each}
</div>
