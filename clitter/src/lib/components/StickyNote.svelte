<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Image, Type, Hash, Lock } from "lucide-svelte";
  import { whiteboardState } from "$lib/stores/whiteboard";
  import { showContextMenu } from "$lib/stores/ui";
  import type { WhiteboardItem } from "$lib/types";

  export let item: WhiteboardItem;

  let isDragging = false;
  let hasMoved = false;
  let startPos = { x: 0, y: 0 };
  let startItemPos = { x: 0, y: 0 };

  const categoryIcons = {
    text: Type,
    image: Image,
    numeric: Hash,
    secure: Lock,
  };

  function handleMouseDown(event: MouseEvent) {
    if (event.button !== 0) return;
    isDragging = true;
    hasMoved = false;
    startPos = { x: event.clientX, y: event.clientY };
    startItemPos = { x: item.position.x, y: item.position.y };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging) return;
    const dx = event.clientX - startPos.x;
    const dy = event.clientY - startPos.y;

    // Only consider it a move if we've moved more than 5 pixels
    if (Math.abs(dx) > 5 || Math.abs(dy) > 5) {
      hasMoved = true;
    }

    if (hasMoved) {
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
  }

  async function handleMouseUp() {
    if (!isDragging) return;
    isDragging = false;
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);

    if (hasMoved) {
      // Save position after drag
      try {
        await invoke("update_whiteboard_item", {
          id: item.id,
          position: item.position,
        });
      } catch (e) {
        console.error("Failed to update position:", e);
      }
    } else {
      // It was a click - paste the content
      await pasteContent();
    }
  }

  async function pasteContent() {
    try {
      await invoke("paste_to_previous_window", { content: item.content });
      getCurrentWindow().hide();
    } catch (e) {
      console.error("Failed to paste:", e);
    }
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    showContextMenu(event.clientX, event.clientY, { type: "item", id: item.id });
  }

  function getPreview(): string {
    if (item.content.data.type === "text") {
      if (item.content.category === "secure") {
        return "••••••••••••";
      }
      const text = item.content.data.text;
      // If there's a label, show truncated value (10 chars)
      if (item.label) {
        return text.length > 10 ? text.substring(0, 10) + "..." : text;
      }
      return item.content.data.preview;
    }
    return "";
  }

  $: IconComponent = categoryIcons[item.content.category];
  $: categoryClass = {
    text: "category-text",
    image: "category-image",
    numeric: "category-numeric",
    secure: "category-secure",
  }[item.content.category];
</script>

<div
  class="sticky-note {categoryClass}"
  class:dragging={isDragging}
  style="left: {item.position.x}px; top: {item.position.y}px;
         width: {item.size.width}px;"
  role="button"
  tabindex="0"
  on:mousedown={handleMouseDown}
  on:contextmenu={handleContextMenu}
>
  {#if item.shortcut}
    <div class="shortcut-badge">{item.shortcut}</div>
  {/if}

  <div class="note-content">
    <span class="category-icon">
      <svelte:component this={IconComponent} size={12} strokeWidth={1.5} />
    </span>
    <div class="preview">
      {#if item.label}
        <p class="label-text">{item.label}</p>
      {/if}
      {#if item.content.data.type === "image"}
        <img
          src="data:image/{item.content.data.format};base64,{item.content.data.base64}"
          alt="Sticky note content"
          class="preview-image"
        />
      {:else}
        <p class="preview-text">{getPreview()}</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .sticky-note {
    position: absolute;
    border-radius: 4px;
    padding: 6px;
    cursor: pointer;
    user-select: none;
    transition: box-shadow 0.15s ease, transform 0.1s ease;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .sticky-note:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .sticky-note.dragging {
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 50;
  }

  .category-text {
    background: rgba(250, 204, 21, 0.15);
    border-color: rgba(250, 204, 21, 0.3);
  }

  .category-image {
    background: rgba(59, 130, 246, 0.15);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .category-numeric {
    background: rgba(34, 197, 94, 0.15);
    border-color: rgba(34, 197, 94, 0.3);
  }

  .category-secure {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.3);
  }

  .shortcut-badge {
    position: absolute;
    top: -6px;
    right: -6px;
    padding: 1px 4px;
    background: #3b82f6;
    color: white;
    font-size: 9px;
    font-weight: 600;
    border-radius: 3px;
  }

  .note-content {
    display: flex;
    align-items: flex-start;
    gap: 4px;
  }

  .category-icon {
    color: #71717a;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .preview {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .label-text {
    margin: 0 0 2px 0;
    font-size: 10px;
    font-weight: 600;
    color: #a1a1aa;
  }

  .preview-text {
    margin: 0;
    font-size: 10px;
    color: #d4d4d8;
    word-wrap: break-word;
    line-height: 1.3;
  }

  .preview-image {
    max-width: 100%;
    max-height: 40px;
    border-radius: 2px;
    object-fit: contain;
  }
</style>
