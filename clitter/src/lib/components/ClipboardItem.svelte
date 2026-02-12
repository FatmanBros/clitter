<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Image, Type, Hash, Lock, Link } from "lucide-svelte";
  import type { ClipboardContent } from "$lib/types";

  export let item: ClipboardContent;
  export let index: number;

  let isDragging = false;

  const categoryIcons = {
    text: Type,
    image: Image,
    numeric: Hash,
    secure: Lock,
    url: Link,
  };

  async function handleClick() {
    try {
      // Paste first, then hide (don't wait for hide)
      await invoke("paste_to_previous_window", { content: item });
      getCurrentWindow().hide();
    } catch (e) {
      console.error("Failed to paste:", e);
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

  $: IconComponent = categoryIcons[item.category];
</script>

<div
  class="clipboard-item"
  class:dragging={isDragging}
  class:secure={item.category === "secure"}
  role="button"
  tabindex="0"
  draggable="true"
  on:click={handleClick}
  on:keypress={(e) => e.key === "Enter" && handleClick()}
  on:dragstart={handleDragStart}
  on:dragend={handleDragEnd}
>
  {#if index < 9}
    <span class="index-badge">{index + 1}</span>
  {/if}

  <span class="category-icon">
    <svelte:component this={IconComponent} size={14} strokeWidth={1.5} />
  </span>

  <div class="content">
    {#if item.data.type === "image"}
      <img
        src="data:image/{item.data.format};base64,{item.data.base64}"
        alt="Clipboard content"
        class="preview-image"
      />
    {:else}
      <p class="preview-text">{getPreview()}</p>
    {/if}
  </div>
</div>

<style>
  .clipboard-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    background: var(--bg-hover);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .clipboard-item:hover {
    background: var(--bg-secondary);
    border-color: var(--border-hover);
  }

  .clipboard-item:active {
    transform: scale(0.98);
  }

  .clipboard-item.dragging {
    opacity: 0.5;
  }

  .clipboard-item.secure {
    border-color: rgba(239, 68, 68, 0.2);
    background: rgba(239, 68, 68, 0.05);
  }

  .index-badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: var(--bg-active);
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .category-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .content {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .preview-text {
    margin: 0;
    font-size: 13px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .preview-image {
    max-width: 100%;
    max-height: 48px;
    object-fit: contain;
    border-radius: 4px;
    background: var(--bg-secondary);
  }
</style>
