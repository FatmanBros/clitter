<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { numberBadges, categoryIcons } from "$lib/stores/clipboard";
  import { whiteboardState } from "$lib/stores/whiteboard";
  import type { ClipboardContent, Position } from "$lib/types";

  export let item: ClipboardContent;
  export let index: number;

  let isDragging = false;

  async function handleClick() {
    try {
      await invoke("copy_to_clipboard", { content: item });
      await getCurrentWindow().hide();
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  }

  function handleDragStart(event: DragEvent) {
    isDragging = true;
    event.dataTransfer?.setData("application/json", JSON.stringify(item));
    event.dataTransfer!.effectAllowed = "copy";
  }

  function handleDragEnd() {
    isDragging = false;
  }

  function getPreview(): string {
    if (item.data.type === "text") {
      return item.category === "secure"
        ? "••••••••••••"
        : item.data.preview;
    }
    return "";
  }
</script>

<div
  class="clipboard-item flex items-start gap-3 p-3 rounded-lg border cursor-pointer
    hover:border-primary-300 hover:bg-primary-50 transition-colors
    {isDragging ? 'opacity-50' : ''}
    {item.category === 'secure' ? 'border-red-200 bg-red-50' : 'border-gray-200 bg-white'}"
  role="button"
  tabindex="0"
  draggable="true"
  on:click={handleClick}
  on:keypress={(e) => e.key === "Enter" && handleClick()}
  on:dragstart={handleDragStart}
  on:dragend={handleDragEnd}
>
  {#if index < 5}
    <span class="number-badge text-primary-600 font-bold text-lg">
      {numberBadges[index]}
    </span>
  {/if}

  <span class="category-icon text-lg">
    {categoryIcons[item.category]}
  </span>

  <div class="flex-1 min-w-0">
    {#if item.data.type === "image"}
      <img
        src="data:image/{item.data.format};base64,{item.data.base64}"
        alt="Clipboard content"
        class="max-h-16 rounded"
      />
    {:else}
      <p class="text-sm text-gray-700 truncate">
        {getPreview()}
      </p>
    {/if}
  </div>
</div>

<style>
  .clipboard-item:active {
    transform: scale(0.98);
  }
</style>
