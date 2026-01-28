<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { X } from "lucide-svelte";
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
    // Stop propagation to prevent App.svelte's keydown handler
    event.stopPropagation();
    if (event.key === "Enter") {
      handleSave();
    } else if (event.key === "Escape") {
      closeShortcutEdit();
    }
  }

  function handleModalKeydown(event: KeyboardEvent) {
    // Stop all keyboard events from reaching App.svelte
    event.stopPropagation();
  }
</script>

{#if $shortcutEditModal.show}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" on:keydown={handleModalKeydown}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal" on:click|stopPropagation on:keydown|stopPropagation>
      <div class="modal-header">
        <h3>Set Shortcut</h3>
        <button class="close-btn" on:click={closeShortcutEdit}>
          <X size={16} strokeWidth={1.5} />
        </button>
      </div>

      <div class="modal-body">
        <label class="label" for="shortcut-input">Shortcut key</label>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          id="shortcut-input"
          type="text"
          bind:value={inputValue}
          on:keydown={handleKeydown}
          class="input"
          placeholder="e.g. da, dev, api"
          autofocus
        />
        <p class="hint">Alphanumeric only. Leave empty to remove.</p>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" on:click={closeShortcutEdit}>
          Cancel
        </button>
        <button class="btn btn-primary" on:click={handleSave}>
          Save
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal {
    background: rgba(39, 39, 42, 0.98);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    width: 320px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: #e4e4e7;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #71717a;
    cursor: pointer;
    transition: all 0.1s ease;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #a1a1aa;
  }

  .modal-body {
    padding: 16px;
  }

  .label {
    display: block;
    font-size: 12px;
    color: #a1a1aa;
    margin-bottom: 6px;
  }

  .input {
    width: 100%;
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #e4e4e7;
    font-size: 14px;
    outline: none;
    transition: border-color 0.15s ease;
    user-select: text;
    -webkit-user-select: text;
  }

  .input:focus {
    border-color: #3b82f6;
  }

  .input::placeholder {
    color: #52525b;
  }

  .hint {
    margin: 8px 0 0 0;
    font-size: 11px;
    color: #52525b;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-secondary {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #a1a1aa;
  }

  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .btn-primary {
    background: #3b82f6;
    border: none;
    color: white;
  }

  .btn-primary:hover {
    background: #2563eb;
  }
</style>
