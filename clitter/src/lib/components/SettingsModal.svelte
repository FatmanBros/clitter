<script lang="ts">
  import { X } from "lucide-svelte";
  import { settingsModal, closeSettings, windowSizes, updateWindowSize } from "$lib/stores/ui";

  let listWidth = $windowSizes.list.width;
  let listHeight = $windowSizes.list.height;
  let whiteboardWidth = $windowSizes.whiteboard.width;
  let whiteboardHeight = $windowSizes.whiteboard.height;

  $: if ($settingsModal.show) {
    listWidth = $windowSizes.list.width;
    listHeight = $windowSizes.list.height;
    whiteboardWidth = $windowSizes.whiteboard.width;
    whiteboardHeight = $windowSizes.whiteboard.height;
  }

  function handleSave() {
    updateWindowSize("list", listWidth, listHeight);
    updateWindowSize("whiteboard", whiteboardWidth, whiteboardHeight);
    closeSettings();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeSettings();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $settingsModal.show}
  <div class="modal-backdrop" on:click={closeSettings}>
    <div class="modal" on:click|stopPropagation>
      <div class="modal-header">
        <h3>Settings</h3>
        <button class="close-btn" on:click={closeSettings}>
          <X size={16} strokeWidth={1.5} />
        </button>
      </div>

      <div class="modal-body">
        <div class="section">
          <h4>List Mode Size</h4>
          <div class="size-inputs">
            <label>
              <span>Width</span>
              <input type="number" bind:value={listWidth} min="200" max="1200" />
            </label>
            <label>
              <span>Height</span>
              <input type="number" bind:value={listHeight} min="200" max="1200" />
            </label>
          </div>
        </div>

        <div class="section">
          <h4>Whiteboard Mode Size</h4>
          <div class="size-inputs">
            <label>
              <span>Width</span>
              <input type="number" bind:value={whiteboardWidth} min="200" max="1600" />
            </label>
            <label>
              <span>Height</span>
              <input type="number" bind:value={whiteboardHeight} min="200" max="1200" />
            </label>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" on:click={closeSettings}>
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
    width: 360px;
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

  .section {
    margin-bottom: 20px;
  }

  .section:last-child {
    margin-bottom: 0;
  }

  .section h4 {
    margin: 0 0 12px 0;
    font-size: 13px;
    font-weight: 500;
    color: #a1a1aa;
  }

  .size-inputs {
    display: flex;
    gap: 12px;
  }

  .size-inputs label {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .size-inputs span {
    font-size: 11px;
    color: #71717a;
  }

  .size-inputs input {
    width: 100%;
    padding: 8px 10px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .size-inputs input:focus {
    border-color: #3b82f6;
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
