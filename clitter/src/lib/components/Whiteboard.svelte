<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { FolderOpen } from "lucide-svelte";
  import {
    whiteboardState,
    shortcutInput,
    matchedShortcuts,
    focusedGroupId,
    groupPath,
  } from "$lib/stores/whiteboard";
  import { showContextMenu } from "$lib/stores/ui";
  import StickyNote from "./StickyNote.svelte";
  import GroupComponent from "./Group.svelte";
  import type { ClipboardContent, Position } from "$lib/types";

  let whiteboardEl: HTMLDivElement;

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    const rect = whiteboardEl.getBoundingClientRect();
    const position: Position = {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
    };
    showContextMenu(event.clientX, event.clientY, { type: "whiteboard", position });
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    const data = event.dataTransfer?.getData("application/json");
    if (!data) return;

    try {
      const content: ClipboardContent = JSON.parse(data);
      const rect = whiteboardEl.getBoundingClientRect();
      const position: Position = {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
      };

      const item = await invoke("add_to_whiteboard", { content, position });
      whiteboardState.update((state) => {
        state.items[(item as any).id] = item as any;
        if (!(item as any).parentGroup) {
          state.rootItems.push((item as any).id);
        }
        return state;
      });
    } catch (e) {
      console.error("Failed to add to whiteboard:", e);
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    event.dataTransfer!.dropEffect = "copy";
  }

  $: visibleItems = Object.values($whiteboardState.items).filter(
    (item) => item.parentGroup === $focusedGroupId
  );

  $: visibleGroups = Object.values($whiteboardState.groups).filter(
    (group) => group.parentGroup === $focusedGroupId
  );

</script>

<div class="whiteboard-wrapper">
  <!-- Shortcut input display -->
  <div class="shortcut-bar">
    <div class="shortcut-input">
      {#if $groupPath.length > 0}
        <span class="path-root">
          <FolderOpen size={12} strokeWidth={1.5} />
        </span>
        {#each $groupPath as segment, i}
          {#if i > 0}
            <span class="separator">/</span>
          {/if}
          <span
            class="group-segment"
            style={segment.color ? `background-color: ${segment.color}25; border-color: ${segment.color}; color: ${segment.color};` : ''}
          >
            {segment.name}
          </span>
        {/each}
        <span class="separator">/</span>
      {/if}
      <span class="input-text">{$shortcutInput || "_"}</span>
      {#if $matchedShortcuts.length > 0 && $shortcutInput}
        <span class="match-count">({$matchedShortcuts.length})</span>
      {/if}
    </div>
    {#if $matchedShortcuts.length > 0 && $shortcutInput}
      <div class="match-list">
        {#each $matchedShortcuts.slice(0, 5) as match}
          <span class="match-item" class:is-group={match.type === "group"}>
            [{match.shortcut}] {match.name}
          </span>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Whiteboard canvas -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={whiteboardEl}
    class="whiteboard-canvas"
    role="application"
    on:contextmenu={handleContextMenu}
    on:drop={handleDrop}
    on:dragover={handleDragOver}
  >
    {#if visibleItems.length === 0 && visibleGroups.length === 0}
      <div class="empty-state">
        <p class="title">Whiteboard</p>
        <p class="hint">Drag items from history</p>
        <p class="hint">Right-click to add group</p>
      </div>
    {/if}

    {#each visibleGroups as group (group.id)}
      <GroupComponent {group} />
    {/each}

    {#each visibleItems as item (item.id)}
      <StickyNote {item} />
    {/each}
  </div>
</div>

<style>
  .whiteboard-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .shortcut-bar {
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .shortcut-input {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 14px;
    flex-wrap: wrap;
  }

  .path-root {
    display: flex;
    align-items: center;
    color: var(--text-muted);
  }

  .group-segment {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 12px;
    color: var(--text-secondary);
    background: var(--bg-hover);
    border: 1px solid var(--border-hover);
  }

  .separator {
    color: var(--text-muted);
  }

  .input-text {
    font-family: monospace;
    color: var(--text-primary);
  }

  .match-count {
    color: var(--text-muted);
    font-size: 11px;
  }

  .match-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }

  .match-item {
    padding: 2px 8px;
    font-size: 11px;
    background: var(--bg-hover);
    border-radius: 4px;
    color: var(--text-secondary);
  }

  .match-item.is-group {
    background: var(--bg-active);
    color: var(--accent);
  }

  .whiteboard-canvas {
    flex: 1;
    position: relative;
    overflow: auto;
    background: var(--bg-secondary);
  }

  .empty-state {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }

  .empty-state .title {
    font-size: 16px;
    margin: 0 0 8px 0;
    color: var(--text-secondary);
  }

  .empty-state .hint {
    font-size: 12px;
    margin: 2px 0;
  }
</style>
