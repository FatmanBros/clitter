<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { ChevronUp, ChevronDown, ChevronLeft, ChevronRight, Image, Type, Hash, Lock, Grid3x3, X, Settings } from "lucide-svelte";

  import ClipboardList from "$lib/components/ClipboardList.svelte";
  import Whiteboard from "$lib/components/Whiteboard.svelte";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import ShortcutEditModal from "$lib/components/ShortcutEditModal.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";

  import { clipboardHistory, selectedCategory, filteredHistory, listScrollOffset, scrollListUp, scrollListDown, canScrollUp, canScrollDown } from "$lib/stores/clipboard";
  import { currentView, contextMenu, hideContextMenu, openSettings, windowSizes, updateWindowSize, windowPositions, updateWindowPosition, shortcutEditModal, settingsModal, themeMode } from "$lib/stores/ui";
  import {
    whiteboardState,
    shortcutInput,
    exactMatch,
    matchedShortcuts,
    clearShortcutInput,
    appendToShortcutInput,
    backspaceShortcutInput,
    enterGroup,
    exitGroup,
    reenterLastGroup,
    focusedGroupId,
    lastExitedGroupId,
  } from "$lib/stores/whiteboard";
  import type { ClipboardContent, ViewMode } from "$lib/types";

  let lastAppliedView: ViewMode | null = null;
  let isResizing = false;
  let wasHidden = true;
  let LogicalSize: typeof import("@tauri-apps/api/dpi").LogicalSize;
  let LogicalPosition: typeof import("@tauri-apps/api/dpi").LogicalPosition;
  let systemTheme: "light" | "dark" = "dark";

  // Cleanup references
  let unlistenClipboard: UnlistenFn | null = null;
  let unlistenResized: UnlistenFn | null = null;
  let unlistenMoved: UnlistenFn | null = null;
  let unlistenFocusChanged: UnlistenFn | null = null;
  let mediaQueryList: MediaQueryList | null = null;
  let mediaQueryHandler: ((e: MediaQueryListEvent) => void) | null = null;

  // Detect system theme
  function detectSystemTheme() {
    if (typeof window !== "undefined" && window.matchMedia) {
      systemTheme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }
  }

  // Compute actual theme based on settings
  $: actualTheme = $themeMode === "system" ? systemTheme : $themeMode;

  async function applyWindowSizeAndPosition(mode: "list" | "whiteboard") {
    if (isResizing || !LogicalSize || !LogicalPosition) return;
    const sizes = $windowSizes[mode];
    const pos = $windowPositions[mode];
    const window = getCurrentWindow();
    await window.setSize(new LogicalSize(sizes.width, sizes.height));
    await window.setPosition(new LogicalPosition(pos.x, pos.y));
  }

  // Watch for view changes and apply window size and position
  $: if ($currentView && $currentView !== lastAppliedView) {
    lastAppliedView = $currentView;
    applyWindowSizeAndPosition($currentView);
  }

  onMount(async () => {
    const dpi = await import("@tauri-apps/api/dpi");
    LogicalSize = dpi.LogicalSize;
    LogicalPosition = dpi.LogicalPosition;

    // Detect and listen for system theme changes
    detectSystemTheme();
    if (typeof window !== "undefined" && window.matchMedia) {
      mediaQueryList = window.matchMedia("(prefers-color-scheme: dark)");
      mediaQueryHandler = (e: MediaQueryListEvent) => {
        systemTheme = e.matches ? "dark" : "light";
      };
      mediaQueryList.addEventListener("change", mediaQueryHandler);
    }

    unlistenClipboard = await listen<ClipboardContent>("clipboard-changed", (event) => {
      clipboardHistory.update((history) => [event.payload, ...history].slice(0, 100));
    });

    const currentWindow = getCurrentWindow();

    // Save window size when user resizes by dragging
    let resizeTimeout: ReturnType<typeof setTimeout>;
    unlistenResized = await currentWindow.onResized(async ({ payload: physicalSize }) => {
      isResizing = true;
      clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(async () => {
        // Convert physical pixels to logical pixels
        const scaleFactor = await currentWindow.scaleFactor();
        const logicalWidth = Math.round(physicalSize.width / scaleFactor);
        const logicalHeight = Math.round(physicalSize.height / scaleFactor);
        const mode = $currentView;
        updateWindowSize(mode, logicalWidth, logicalHeight);
        isResizing = false;
      }, 200);
    });

    // Save window position when moved
    let moveTimeout: ReturnType<typeof setTimeout>;
    unlistenMoved = await currentWindow.onMoved(async ({ payload: physicalPosition }) => {
      clearTimeout(moveTimeout);
      moveTimeout = setTimeout(async () => {
        const scaleFactor = await currentWindow.scaleFactor();
        const logicalX = Math.round(physicalPosition.x / scaleFactor);
        const logicalY = Math.round(physicalPosition.y / scaleFactor);
        const mode = $currentView;
        updateWindowPosition(mode, logicalX, logicalY);
      }, 200);
    });

    unlistenFocusChanged = await currentWindow.onFocusChanged(({ payload: focused }) => {
      if (focused && wasHidden) {
        wasHidden = false;
        currentView.set("list");
        selectedCategory.set(null);
        listScrollOffset.set(0); // Reset scroll position
        // Apply saved position for list view
        const pos = $windowPositions.list;
        currentWindow.setPosition(new LogicalPosition(pos.x, pos.y));
      }
    });

    // Apply initial position for list view
    const initialPos = $windowPositions.list;
    await currentWindow.setPosition(new LogicalPosition(initialPos.x, initialPos.y));

    try {
      const history = await invoke<ClipboardContent[]>("get_recent_items", { count: 100 });
      clipboardHistory.set(history);

      const wb = await invoke<any>("get_whiteboard");
      whiteboardState.set(wb);
    } catch (e) {
      console.error("Failed to load initial data:", e);
    }
  });

  onDestroy(() => {
    // Clean up all event listeners
    unlistenClipboard?.();
    unlistenResized?.();
    unlistenMoved?.();
    unlistenFocusChanged?.();
    if (mediaQueryList && mediaQueryHandler) {
      mediaQueryList.removeEventListener("change", mediaQueryHandler);
    }
  });

  async function startDrag() {
    await getCurrentWindow().startDragging();
  }

  function hideWindow() {
    wasHidden = true;
    getCurrentWindow().hide();
  }

  async function copyHistoryItem(index: number) {
    const items = $filteredHistory;
    if (index < items.length) {
      try {
        // Paste first, then hide (don't wait for hide)
        await invoke("paste_to_previous_window", { content: items[index] });
        hideWindow();
      } catch (e) {
        console.error("Failed to paste:", e);
      }
    }
  }

  async function copyWhiteboardItem(itemId: string) {
    const item = $whiteboardState.items[itemId];
    if (item) {
      try {
        // Paste first, then hide (don't wait for hide)
        await invoke("paste_to_previous_window", { content: item.content });
        hideWindow();
      } catch (e) {
        console.error("Failed to paste:", e);
      }
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Skip when modals are open
    if ($shortcutEditModal.show || $settingsModal.show) {
      return;
    }

    // Only close context menu on Escape
    if ($contextMenu.show) {
      if (event.key === "Escape") {
        hideContextMenu();
        event.preventDefault();
        return;
      }
      // Don't process other keys while context menu is open
      return;
    }

    if ($currentView === "list") {
      handleListKeydown(event);
    } else {
      handleWhiteboardKeydown(event);
    }
  }

  function handleListKeydown(event: KeyboardEvent) {
    // Handle scroll with Shift or Alt + Arrow keys
    if (event.shiftKey || event.altKey) {
      if (event.key === "ArrowUp") {
        scrollListUp();
        event.preventDefault();
        return;
      }
      if (event.key === "ArrowDown") {
        scrollListDown();
        event.preventDefault();
        return;
      }
    }

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
      case "w":
      case "W":
        currentView.set("whiteboard");
        event.preventDefault();
        break;
      case "a":
      case "A":
        selectedCategory.set(null);
        event.preventDefault();
        break;
      case "s":
      case "S":
        selectedCategory.set("secure");
        event.preventDefault();
        break;
      case "1":
      case "2":
      case "3":
      case "4":
      case "5":
      case "6":
      case "7":
      case "8":
      case "9":
        const index = parseInt(event.key) - 1;
        copyHistoryItem(index);
        event.preventDefault();
        break;
      case "Escape":
        hideWindow();
        event.preventDefault();
        break;
    }
  }

  function handleWhiteboardKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case "ArrowDown":
        // Always go back to list view
        clearShortcutInput();
        focusedGroupId.set(null);
        currentView.set("list");
        event.preventDefault();
        break;
      case "l":
      case "L":
        // Go to list if no input, otherwise append to input
        if (!$shortcutInput) {
          clearShortcutInput();
          focusedGroupId.set(null);
          currentView.set("list");
          event.preventDefault();
        } else {
          appendToShortcutInput(event.key);
          event.preventDefault();
        }
        return; // Early return to skip default case
      case "ArrowLeft":
        // Exit current group (go to parent)
        if ($focusedGroupId) {
          exitGroup();
        }
        event.preventDefault();
        break;
      case "ArrowRight":
        // Re-enter last exited group
        if ($lastExitedGroupId) {
          reenterLastGroup();
        }
        event.preventDefault();
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
      case "Tab":
        // Autocomplete to first matched shortcut
        if ($shortcutInput && $matchedShortcuts.length > 0) {
          const firstMatch = $matchedShortcuts[0];
          // Use shortcut if available, otherwise use label or name
          const completion = firstMatch.shortcut || firstMatch.label || firstMatch.name;
          shortcutInput.set(completion.toLowerCase());
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
        if (event.key.length === 1 && /[a-zA-Z0-9\/]/.test(event.key)) {
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

<main class="app-container theme-{actualTheme}">
  <!-- Title bar (draggable) -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="title-bar" on:mousedown={startDrag}>
    <div class="title-spacer"></div>
    <button class="title-btn" on:mousedown|stopPropagation on:click={openSettings}>
      <Settings size={14} strokeWidth={1.5} />
    </button>
    <button class="title-btn close" on:mousedown|stopPropagation on:click={() => hideWindow()}>
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
        <span>リスト</span>
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
    backdrop-filter: blur(12px);
    user-select: none;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    /* Default dark theme */
    --bg-primary: rgba(24, 24, 27, 0.95);
    --bg-secondary: rgba(0, 0, 0, 0.2);
    --bg-hover: rgba(255, 255, 255, 0.05);
    --bg-active: rgba(59, 130, 246, 0.1);
    --border-color: rgba(255, 255, 255, 0.06);
    --border-hover: rgba(255, 255, 255, 0.12);
    --text-primary: #e4e4e7;
    --text-secondary: #a1a1aa;
    --text-muted: #71717a;
    --accent: #3b82f6;
    --accent-bg: rgba(59, 130, 246, 0.3);
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .app-container.theme-light {
    --bg-primary: rgba(255, 255, 255, 0.95);
    --bg-secondary: rgba(0, 0, 0, 0.05);
    --bg-hover: rgba(0, 0, 0, 0.05);
    --bg-active: rgba(59, 130, 246, 0.1);
    --border-color: rgba(0, 0, 0, 0.08);
    --border-hover: rgba(0, 0, 0, 0.15);
    --text-primary: #18181b;
    --text-secondary: #52525b;
    --text-muted: #71717a;
    --accent: #3b82f6;
    --accent-bg: rgba(59, 130, 246, 0.2);
  }

  .title-bar {
    display: flex;
    align-items: center;
    height: 32px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
    cursor: grab;
  }

  .title-bar:active {
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
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .title-btn:hover {
    background: var(--bg-hover);
    color: var(--text-secondary);
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
    color: var(--text-muted);
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.15s ease;
    padding: 0;
  }

  .nav-btn:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }

  .nav-btn.active {
    color: var(--accent);
    background: var(--bg-active);
  }

  .nav-top {
    grid-area: top;
    flex-direction: row;
    border-bottom: 1px solid var(--border-color);
  }

  .nav-left {
    grid-area: left;
    flex-direction: column;
    border-right: 1px solid var(--border-color);
  }

  .nav-right {
    grid-area: right;
    flex-direction: column;
    border-left: 1px solid var(--border-color);
  }

  .nav-bottom {
    grid-area: bottom;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border-top: 1px solid var(--border-color);
    padding: 0 12px;
  }

  .nav-btn-inline {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .nav-btn-inline:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
    border-color: var(--border-hover);
  }

  .nav-btn-inline.active {
    color: var(--accent);
    background: var(--bg-active);
    border-color: var(--accent-bg);
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
    border-top: 1px solid var(--border-color);
  }
</style>
