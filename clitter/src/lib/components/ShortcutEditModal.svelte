<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { shortcutEditModal, closeShortcutEdit } from "$lib/stores/ui";
  import { whiteboardState } from "$lib/stores/whiteboard";

  let inputValue = "";

  $: if ($shortcutEditModal.show) {
    inputValue = $shortcutEditModal.currentShortcut;
  }

  async function handleSave() {
    if (!$shortcutEditModal.targetId) return;

    const shortcut = inputValue.trim() || null;

    try {
      if ($shortcutEditModal.targetType === "item") {
        await invoke("set_item_shortcut", {
          id: $shortcutEditModal.targetId,
          shortcut,
        });
        whiteboardState.update((state) => {
          if (state.items[$shortcutEditModal.targetId!]) {
            state.items[$shortcutEditModal.targetId!].shortcut = shortcut;
          }
          return state;
        });
      } else if ($shortcutEditModal.targetType === "group") {
        await invoke("update_group", {
          id: $shortcutEditModal.targetId,
          name: undefined,
          collapsed: undefined,
          position: undefined,
        });
        // Note: Group shortcut update would need a separate command
        // For now, we update locally
        whiteboardState.update((state) => {
          if (state.groups[$shortcutEditModal.targetId!]) {
            state.groups[$shortcutEditModal.targetId!].shortcut = shortcut;
          }
          return state;
        });
      }
    } catch (e) {
      console.error("Failed to set shortcut:", e);
    }

    closeShortcutEdit();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      handleSave();
    } else if (event.key === "Escape") {
      closeShortcutEdit();
    }
  }
</script>

{#if $shortcutEditModal.show}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg shadow-xl p-6 w-80">
      <h3 class="text-lg font-semibold mb-4">ショートカット設定</h3>

      <div class="mb-4">
        <label class="block text-sm text-gray-600 mb-1">
          ショートカット文字列
        </label>
        <input
          type="text"
          bind:value={inputValue}
          on:keydown={handleKeydown}
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          placeholder="例: da, dev, api"
          autofocus
        />
        <p class="text-xs text-gray-500 mt-1">
          英数字のみ。空欄でショートカット解除。
        </p>
      </div>

      <div class="flex gap-2 justify-end">
        <button
          class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg"
          on:click={closeShortcutEdit}
        >
          キャンセル
        </button>
        <button
          class="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600"
          on:click={handleSave}
        >
          保存
        </button>
      </div>
    </div>
  </div>
{/if}
