<script lang="ts">
  import { X, Maximize2, Layout } from "lucide-svelte";
  import { settingsModal, closeSettings, windowSizes, updateWindowSize } from "$lib/stores/ui";

  let activeTab: "list" | "whiteboard" = "list";
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
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-backdrop" on:click={closeSettings}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal" on:click|stopPropagation>
      <div class="modal-header">
        <h3>Settings</h3>
        <button class="close-btn" on:click={closeSettings}>
          <X size={16} strokeWidth={1.5} />
        </button>
      </div>

      <div class="modal-content">
        <nav class="sidebar">
          <button
            class="nav-item"
            class:active={activeTab === "list"}
            on:click={() => (activeTab = "list")}
          >
            <Layout size={16} strokeWidth={1.5} />
            <span>List Mode</span>
          </button>
          <button
            class="nav-item"
            class:active={activeTab === "whiteboard"}
            on:click={() => (activeTab = "whiteboard")}
          >
            <Maximize2 size={16} strokeWidth={1.5} />
            <span>Whiteboard</span>
          </button>
        </nav>

        <div class="settings-panel">
          {#if activeTab === "list"}
            <div class="panel-header">
              <h4>List Mode Size</h4>
              <p class="hint">Window size for clipboard list view</p>
            </div>
            <div class="form-group">
              <label>
                <span class="label-text">Width</span>
                <div class="input-wrapper">
                  <input type="number" bind:value={listWidth} min="200" max="1200" />
                  <span class="unit">px</span>
                </div>
              </label>
              <label>
                <span class="label-text">Height</span>
                <div class="input-wrapper">
                  <input type="number" bind:value={listHeight} min="200" max="1200" />
                  <span class="unit">px</span>
                </div>
              </label>
            </div>
          {:else}
            <div class="panel-header">
              <h4>Whiteboard Size</h4>
              <p class="hint">Window size for whiteboard view</p>
            </div>
            <div class="form-group">
              <label>
                <span class="label-text">Width</span>
                <div class="input-wrapper">
                  <input type="number" bind:value={whiteboardWidth} min="200" max="1600" />
                  <span class="unit">px</span>
                </div>
              </label>
              <label>
                <span class="label-text">Height</span>
                <div class="input-wrapper">
                  <input type="number" bind:value={whiteboardHeight} min="200" max="1200" />
                  <span class="unit">px</span>
                </div>
              </label>
            </div>
          {/if}
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
    width: 480px;
    max-height: 80vh;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
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

  .modal-content {
    display: flex;
    flex: 1;
    min-height: 200px;
  }

  .sidebar {
    width: 140px;
    background: rgba(0, 0, 0, 0.2);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    border-right: 1px solid rgba(255, 255, 255, 0.06);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #a1a1aa;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }

  .nav-item:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #d4d4d8;
  }

  .nav-item.active {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
  }

  .settings-panel {
    flex: 1;
    padding: 20px;
  }

  .panel-header {
    margin-bottom: 20px;
  }

  .panel-header h4 {
    margin: 0 0 4px 0;
    font-size: 14px;
    font-weight: 600;
    color: #e4e4e7;
  }

  .panel-header .hint {
    margin: 0;
    font-size: 12px;
    color: #71717a;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-group label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label-text {
    font-size: 12px;
    font-weight: 500;
    color: #a1a1aa;
  }

  .input-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .input-wrapper input {
    flex: 1;
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #e4e4e7;
    font-size: 14px;
    outline: none;
    transition: border-color 0.15s ease;
    user-select: text;
    -webkit-user-select: text;
  }

  .input-wrapper input:focus {
    border-color: #3b82f6;
  }

  .unit {
    font-size: 12px;
    color: #71717a;
    min-width: 20px;
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
