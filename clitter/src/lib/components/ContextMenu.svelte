<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { contextMenu, hideContextMenu, openShortcutEdit } from "$lib/stores/ui";
  import { whiteboardState } from "$lib/stores/whiteboard";
  import type { Position } from "$lib/types";

  async function handleAddGroup() {
    if ($contextMenu.target?.type !== "whiteboard") return;

    const position = $contextMenu.target.position;
    const name = prompt("グループ名を入力してください", "新規グループ");
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
      try {
        await invoke("remove_from_whiteboard", { id: $contextMenu.target.id });
        whiteboardState.update((state) => {
          delete state.items[$contextMenu.target!.id];
          state.rootItems = state.rootItems.filter((id) => id !== $contextMenu.target!.id);
          return state;
        });
      } catch (e) {
        console.error("Failed to delete item:", e);
      }
    } else if ($contextMenu.target.type === "group") {
      try {
        await invoke("delete_group", { id: $contextMenu.target.id });
        whiteboardState.update((state) => {
          delete state.groups[$contextMenu.target!.id];
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
  <div
    class="fixed bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-50 min-w-[160px]"
    style="left: {$contextMenu.x}px; top: {$contextMenu.y}px;"
  >
    {#if $contextMenu.target?.type === "whiteboard"}
      <button
        class="w-full px-4 py-2 text-left text-sm hover:bg-gray-100 flex items-center gap-2"
        on:click={handleAddGroup}
      >
        📁 グループを追加
      </button>
    {:else}
      <button
        class="w-full px-4 py-2 text-left text-sm hover:bg-gray-100 flex items-center gap-2"
        on:click={handleSetShortcut}
      >
        ⌨️ ショートカット設定
      </button>
      <hr class="my-1 border-gray-200" />
      <button
        class="w-full px-4 py-2 text-left text-sm hover:bg-red-50 text-red-600 flex items-center gap-2"
        on:click={handleDelete}
      >
        🗑️ 削除
      </button>
    {/if}
  </div>
{/if}
