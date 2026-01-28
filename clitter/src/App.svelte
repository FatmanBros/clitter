<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import ClipboardList from "$lib/components/ClipboardList.svelte";
  import Whiteboard from "$lib/components/Whiteboard.svelte";
  import CategoryTabs from "$lib/components/CategoryTabs.svelte";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import ShortcutEditModal from "$lib/components/ShortcutEditModal.svelte";

  import { clipboardHistory, selectedCategory, filteredHistory } from "$lib/stores/clipboard";
  import { currentView, contextMenu, hideContextMenu } from "$lib/stores/ui";
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
    focusedGroupId,
  } from "$lib/stores/whiteboard";
  import type { ClipboardContent } from "$lib/types";

  onMount(async () => {
    // Listen for clipboard changes from backend
    await listen<ClipboardContent>("clipboard-changed", (event) => {
      clipboardHistory.update((history) => [event.payload, ...history].slice(0, 100));
    });

    // Load initial data
    try {
      const history = await invoke<ClipboardContent[]>("get_recent_items", { count: 100 });
      clipboardHistory.set(history);

      const wb = await invoke<any>("get_whiteboard");
      whiteboardState.set(wb);
    } catch (e) {
      console.error("Failed to load initial data:", e);
    }
  });

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
    // Hide context menu on any key
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
      case " ": // Space
        if ($exactMatch && $exactMatch.type === "group") {
          enterGroup($exactMatch.id);
          clearShortcutInput();
        }
        event.preventDefault();
        break;
      default:
        // Alphanumeric input for shortcut matching
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
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="h-screen flex flex-col bg-white" on:click={handleClick}>
  <header class="px-4 py-2 border-b border-gray-200 flex items-center justify-between">
    <h1 class="text-lg font-semibold text-gray-800">Clitter</h1>
    {#if $currentView === "whiteboard"}
      <button
        class="text-sm text-gray-500 hover:text-gray-700"
        on:click={() => currentView.set("list")}
      >
        ← 戻る
      </button>
    {/if}
  </header>

  {#if $currentView === "list"}
    <CategoryTabs />
    <ClipboardList />
    <footer class="px-4 py-2 border-t border-gray-200 text-center text-sm text-gray-500">
      ↑ ホワイトボードを開く
    </footer>
  {:else}
    <Whiteboard />
  {/if}

  <ContextMenu />
  <ShortcutEditModal />
</main>

<style>
  main {
    user-select: none;
  }
</style>
