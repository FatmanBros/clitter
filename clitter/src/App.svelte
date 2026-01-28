<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { ChevronUp, ChevronDown, ChevronLeft, ChevronRight, Image, Type, Hash, Lock, Grid3x3, X, Settings, GripVertical } from "lucide-svelte";

  import ClipboardList from "$lib/components/ClipboardList.svelte";
  import Whiteboard from "$lib/components/Whiteboard.svelte";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import ShortcutEditModal from "$lib/components/ShortcutEditModal.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";

  import { clipboardHistory, selectedCategory, filteredHistory } from "$lib/stores/clipboard";
  import { currentView, contextMenu, hideContextMenu, openSettings, windowSizes } from "$lib/stores/ui";
  import {
    whiteboardState,
    shortcutInput,
    exactMatch,
    clearShortcutInput,
    appendToShortcutInput,
    backspaceShortcutInput,
    enterGroup,
    exitGroup,
    focusedGroupId,
  } from "$lib/stores/whiteboard";
  import type { ClipboardContent } from "$lib/types";

  async function applyWindowSize(mode: "list" | "whiteboard") {
    const sizes = $windowSizes[mode];
    const window = getCurrentWindow();
    await window.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(sizes.width, sizes.height));
  }

  // Watch for view changes and apply window size
  $: if ($currentView) {
    applyWindowSize($currentView);
  }

  onMount(async () => {
    await listen<ClipboardContent>("clipboard-changed", (event) => {
      clipboardHistory.update((history) => [event.payload, ...history].slice(0, 100));
    });

    const currentWindow = getCurrentWindow();
    await currentWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        currentView.set("list");
        selectedCategory.set(null);
      }
    });

    try {
      const history = await invoke<ClipboardContent[]>("get_recent_items", { count: 100 });
      clipboardHistory.set(history);

      const wb = await invoke<any>("get_whiteboard");
      whiteboardState.set(wb);
    } catch (e) {
      console.error("Failed to load initial data:", e);
    }
  });

  async function startDrag() {
    await getCurrentWindow().startDragging();
  }

  async function copyHistoryItem(index: number) {
    const items = $filteredHistory;
    if (index < items.length) {
      try {
        await invoke("copy_to_clipboard", { content: items[index] });
        await getCurrentWindow().hide();
      } catch (e) {
        console.error("Failed to copy:", e);
      }
    }
  }

  async function copyWhiteboardItem(itemId: string) {
    const item = $whiteboardState.items[itemId];
    if (item) {
      try {
        await invoke("copy_to_clipboard", { content: item.content });
        await getCurrentWindow().hide();
      } catch (e) {
        console.error("Failed to copy:", e);
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if ($contextMenu.show) {
      hideContextMenu();
    }

    if ($currentView === "list") {
      handleListKeydown(event);
    } else {
      handleWhiteboardKeydown(event);
    }
  }

  function handleListKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case "ArrowLeft":
        selectedCategory.set("image");
        event.preventDefault();
        break;
      case "ArrowDown":
        selectedCategory.set("text");
        event.preventDefault();
        break;
      case "ArrowRight":
        selectedCategory.set("numeric");
        event.preventDefault();
        break;
      case "ArrowUp":
        currentView.set("whiteboard");
        event.preventDefault();
        break;
      case "1":
      case "2":
      case "3":
      case "4":
      case "5":
        const index = parseInt(event.key) - 1;
        copyHistoryItem(index);
        event.preventDefault();
        break;
      case "Escape":
        getCurrentWindow().hide();
        event.preventDefault();
        break;
    }
  }

  function handleWhiteboardKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case "ArrowDown":
        if (!$shortcutInput && !$focusedGroupId) {
          currentView.set("list");
          event.preventDefault();
        }
        break;
      case "Escape":
        if ($shortcutInput) {
          clearShortcutInput();
        } else if ($focusedGroupId) {
          exitGroup();
        } else {
          currentView.set("list");
        }
        event.preventDefault();
        break;
      case "Backspace":
        if ($shortcutInput) {
          backspaceShortcutInput();
        } else if ($focusedGroupId) {
          exitGroup();
        }
        event.preventDefault();
        break;
      case "Enter":
        if ($exactMatch) {
          if ($exactMatch.type === "item") {
            copyWhiteboardItem($exactMatch.id);
          } else {
            enterGroup($exactMatch.id);
          }
          clearShortcutInput();
        }
        event.preventDefault();
        break;
      case " ":
        if ($exactMatch && $exactMatch.type === "group") {
          enterGroup($exactMatch.id);
          clearShortcutInput();
        }
        event.preventDefault();
        break;
      default:
        if (event.key.length === 1 && /[a-zA-Z0-9]/.test(event.key)) {
          appendToShortcutInput(event.key);
          event.preventDefault();
        }
        break;
    }
  }

  function handleClick() {
    if ($contextMenu.show) {
      hideContextMenu();
    }
  }

  function selectCategory(cat: "image" | "text" | "numeric" | null) {
    selectedCategory.set(cat);
  }
</script>

<svelte:window on:keydown={handleKeydown} on:click={handleClick} />

<main class="app-container">
  <!-- Title bar -->
  <div class="title-bar">
    <button class="drag-handle" on:mousedown={startDrag}>
      <GripVertical size={14} strokeWidth={1.5} />
    </button>
    <div class="title-spacer"></div>
    <button class="title-btn" on:click={openSettings}>
      <Settings size={14} strokeWidth={1.5} />
    </button>
    <button class="title-btn close" on:click={() => getCurrentWindow().hide()}>
      <X size={14} strokeWidth={2} />
    </button>
  </div>

  {#if $currentView === "list"}
    <!-- List View -->
    <div class="layout-grid">
      <!-- Top: Whiteboard navigation -->
      <button
        class="nav-btn nav-top"
        on:click={() => currentView.set("whiteboard")}
      >
        <ChevronUp size={16} strokeWidth={1.5} />
        <span>Whiteboard</span>
      </button>

      <!-- Left: Image category -->
      <button
        class="nav-btn nav-left"
        class:active={$selectedCategory === "image"}
        on:click={() => selectCategory("image")}
      >
        <ChevronLeft size={14} strokeWidth={1.5} />
        <Image size={16} strokeWidth={1.5} />
      </button>

      <!-- Center: Main content -->
      <div class="main-content">
        <ClipboardList />
      </div>

      <!-- Right: Numeric category -->
      <button
        class="nav-btn nav-right"
        class:active={$selectedCategory === "numeric"}
        on:click={() => selectCategory("numeric")}
      >
        <Hash size={16} strokeWidth={1.5} />
        <ChevronRight size={14} strokeWidth={1.5} />
      </button>

      <!-- Bottom: Text category / All -->
      <div class="nav-bottom">
        <button
          class="nav-btn-inline"
          class:active={$selectedCategory === "text"}
          on:click={() => selectCategory("text")}
        >
          <Type size={16} strokeWidth={1.5} />
          <ChevronDown size={14} strokeWidth={1.5} />
        </button>
        <button
          class="nav-btn-inline"
          class:active={$selectedCategory === null}
          on:click={() => selectCategory(null)}
        >
          <Grid3x3 size={16} strokeWidth={1.5} />
          <span>All</span>
        </button>
        <button
          class="nav-btn-inline"
          class:active={$selectedCategory === "secure"}
          on:click={() => selectedCategory.set("secure")}
        >
          <Lock size={16} strokeWidth={1.5} />
        </button>
      </div>
    </div>
  {:else}
    <!-- Whiteboard View -->
    <div class="whiteboard-container">
      <Whiteboard />
      <button
        class="nav-btn nav-bottom-single"
        on:click={() => currentView.set("list")}
      >
        <ChevronDown size={16} strokeWidth={1.5} />
        <span>Back to List</span>
      </button>
    </div>
  {/if}

  <ContextMenu />
  <ShortcutEditModal />
  <SettingsModal />
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent;
  }

  .app-container {
    height: 100vh;
    width: 100vw;
    background: rgba(24, 24, 27, 0.95);
    backdrop-filter: blur(12px);
    color: #e4e4e7;
    user-select: none;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .title-bar {
    display: flex;
    align-items: center;
    height: 32px;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
  }

  .drag-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    color: #52525b;
    cursor: grab;
    transition: color 0.15s ease;
  }

  .drag-handle:hover {
    color: #71717a;
  }

  .drag-handle:active {
    cursor: grabbing;
  }

  .title-spacer {
    flex: 1;
  }

  .title-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    color: #71717a;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .title-btn:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #a1a1aa;
  }

  .title-btn.close:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
  }

  .layout-grid {
    display: grid;
    grid-template-areas:
      ". top ."
      "left center right"
      ". bottom .";
    grid-template-columns: 48px 1fr 48px;
    grid-template-rows: 40px 1fr 48px;
    flex: 1;
    gap: 0;
  }

  .nav-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    background: transparent;
    border: none;
    color: #71717a;
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.15s ease;
    padding: 0;
  }

  .nav-btn:hover {
    color: #a1a1aa;
    background: rgba(255, 255, 255, 0.04);
  }

  .nav-btn.active {
    color: #3b82f6;
    background: rgba(59, 130, 246, 0.1);
  }

  .nav-top {
    grid-area: top;
    flex-direction: row;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .nav-left {
    grid-area: left;
    flex-direction: column;
    border-right: 1px solid rgba(255, 255, 255, 0.06);
  }

  .nav-right {
    grid-area: right;
    flex-direction: column;
    border-left: 1px solid rgba(255, 255, 255, 0.06);
  }

  .nav-bottom {
    grid-area: bottom;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    padding: 0 12px;
  }

  .nav-btn-inline {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    color: #71717a;
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .nav-btn-inline:hover {
    color: #a1a1aa;
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .nav-btn-inline.active {
    color: #3b82f6;
    background: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .main-content {
    grid-area: center;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .whiteboard-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .nav-bottom-single {
    flex-direction: row;
    padding: 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
</style>
