<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { FolderPlus, Keyboard, Trash2, ClipboardPaste } from "lucide-svelte";
  import { contextMenu, hideContextMenu, openShortcutEdit } from "$lib/stores/ui";
  import { whiteboardState } from "$lib/stores/whiteboard";
  import { clipboardHistory } from "$lib/stores/clipboard";
  import { get } from "svelte/store";

  async function handleAddGroup() {
    if ($contextMenu.target?.type !== "whiteboard") return;

    const position = $contextMenu.target.position;
    const name = prompt("Group name", "New Group");
    if (!name) return;

    try {
      const group = await invoke("create_group", { name, position });
      whiteboardState.update((state) => {
        state.groups[(group as any).id] = group as any;
        return state;
      });
    } catch (e) {
      console.error("Failed to create group:", e);
    }

    hideContextMenu();
  }

  async function handlePasteItem() {
    console.log("handlePasteItem called, target:", $contextMenu.target);
    if ($contextMenu.target?.type !== "whiteboard") {
      console.log("Target is not whiteboard, returning");
      return;
    }

    const history = get(clipboardHistory);
    console.log("Clipboard history length:", history.length);
    if (history.length === 0) {
      alert("No items in clipboard history");
      hideContextMenu();
      return;
    }

    const position = $contextMenu.target.position;
    const content = history[0]; // Use most recent item
    console.log("Adding item at position:", position, "content:", content);

    try {
      const item = await invoke("add_to_whiteboard", { content, position });
      console.log("Item added successfully:", item);
      whiteboardState.update((state) => {
        state.items[(item as any).id] = item as any;
        if (!(item as any).parentGroup) {
          state.rootItems.push((item as any).id);
        }
        return state;
      });
    } catch (e) {
      console.error("Failed to add item to whiteboard:", e);
    }

    hideContextMenu();
  }

  function handleSetShortcut() {
    if (!$contextMenu.target) return;

    if ($contextMenu.target.type === "item") {
      const item = $whiteboardState.items[$contextMenu.target.id];
      openShortcutEdit("item", $contextMenu.target.id, item?.shortcut || "");
    } else if ($contextMenu.target.type === "group") {
      const group = $whiteboardState.groups[$contextMenu.target.id];
      openShortcutEdit("group", $contextMenu.target.id, group?.shortcut || "");
    }

    hideContextMenu();
  }

  async function handleDelete() {
    if (!$contextMenu.target) return;

    if ($contextMenu.target.type === "item") {
      const targetId = $contextMenu.target.id;
      try {
        await invoke("remove_from_whiteboard", { id: targetId });
        whiteboardState.update((state) => {
          delete state.items[targetId];
          state.rootItems = state.rootItems.filter((id) => id !== targetId);
          return state;
        });
      } catch (e) {
        console.error("Failed to delete item:", e);
      }
    } else if ($contextMenu.target.type === "group") {
      const targetId = $contextMenu.target.id;
      try {
        await invoke("delete_group", { id: targetId });
        whiteboardState.update((state) => {
          delete state.groups[targetId];
          return state;
        });
      } catch (e) {
        console.error("Failed to delete group:", e);
      }
    }

    hideContextMenu();
  }
</script>

{#if $contextMenu.show}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="context-menu"
    style="left: {$contextMenu.x}px; top: {$contextMenu.y}px;"
    on:click|stopPropagation
  >
    {#if $contextMenu.target?.type === "whiteboard"}
      <button class="menu-item" on:click={handlePasteItem}>
        <ClipboardPaste size={14} strokeWidth={1.5} />
        Paste Item
      </button>
      <button class="menu-item" on:click={handleAddGroup}>
        <FolderPlus size={14} strokeWidth={1.5} />
        Add Group
      </button>
    {:else}
      <button class="menu-item" on:click={handleSetShortcut}>
        <Keyboard size={14} strokeWidth={1.5} />
        Set Shortcut
      </button>
      <hr class="divider" />
      <button class="menu-item danger" on:click={handleDelete}>
        <Trash2 size={14} strokeWidth={1.5} />
        Delete
      </button>
    {/if}
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    background: rgba(39, 39, 42, 0.95);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 4px;
    min-width: 150px;
    z-index: 100;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .menu-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: #d4d4d8;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s ease;
  }

  .menu-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .menu-item.danger {
    color: #f87171;
  }

  .menu-item.danger:hover {
    background: rgba(239, 68, 68, 0.15);
  }

  .divider {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    margin: 4px 0;
  }
</style>
