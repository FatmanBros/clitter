<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { whiteboardState } from "$lib/stores/whiteboard";
  import { showContextMenu } from "$lib/stores/ui";
  import { categoryIcons } from "$lib/stores/clipboard";
  import type { WhiteboardItem, Position } from "$lib/types";

  export let item: WhiteboardItem;

  let isDragging = false;
  let startPos = { x: 0, y: 0 };
  let startItemPos = { x: 0, y: 0 };

  function handleMouseDown(event: MouseEvent) {
    if (event.button !== 0) return; // Only left click
    isDragging = true;
    startPos = { x: event.clientX, y: event.clientY };
    startItemPos = { x: item.position.x, y: item.position.y };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging) return;
    const dx = event.clientX - startPos.x;
    const dy = event.clientY - startPos.y;

    whiteboardState.update((state) => {
      if (state.items[item.id]) {
        state.items[item.id].position = {
          x: Math.max(0, startItemPos.x + dx),
          y: Math.max(0, startItemPos.y + dy),
        };
      }
      return state;
    });
  }

  async function handleMouseUp() {
    if (!isDragging) return;
    isDragging = false;
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);

    // Save position to backend
    try {
      await invoke("update_whiteboard_item", {
        id: item.id,
        position: item.position,
      });
    } catch (e) {
      console.error("Failed to update position:", e);
    }
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    showContextMenu(event.clientX, event.clientY, { type: "item", id: item.id });
  }

  function getPreview(): string {
    if (item.content.data.type === "text") {
      return item.content.category === "secure"
        ? "••••••••••••"
        : item.content.data.preview;
    }
    return "";
  }

  $: categoryClass = {
    text: "bg-yellow-100 border-yellow-300",
    image: "bg-blue-100 border-blue-300",
    numeric: "bg-green-100 border-green-300",
    secure: "bg-red-100 border-red-300",
  }[item.content.category];
</script>

<div
  class="sticky-note absolute rounded-lg shadow-md border-2 p-3 cursor-move select-none
    {categoryClass}
    {isDragging ? 'shadow-xl z-50' : ''}"
  style="left: {item.position.x}px; top: {item.position.y}px;
         width: {item.size.width}px; min-height: {item.size.height}px;"
  role="button"
  tabindex="0"
  on:mousedown={handleMouseDown}
  on:contextmenu={handleContextMenu}
>
  <!-- Shortcut badge -->
  {#if item.shortcut}
    <div class="absolute -top-2 -right-2 px-2 py-0.5 bg-primary-500 text-white text-xs rounded-full font-bold">
      {item.shortcut}
    </div>
  {/if}

  <!-- Category icon -->
  <div class="flex items-start gap-2">
    <span class="text-lg">{categoryIcons[item.content.category]}</span>
    <div class="flex-1 min-w-0 overflow-hidden">
      {#if item.content.data.type === "image"}
        <img
          src="data:image/{item.content.data.format};base64,{item.content.data.base64}"
          alt="Sticky note content"
          class="max-w-full rounded"
        />
      {:else}
        <p class="text-sm text-gray-700 break-words">
          {getPreview()}
        </p>
      {/if}
    </div>
  </div>
</div>

<style>
  .sticky-note {
    transition: box-shadow 0.2s ease, transform 0.1s ease;
  }

  .sticky-note:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
</style>
