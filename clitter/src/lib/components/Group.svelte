<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Folder, ChevronRight, ChevronDown } from "lucide-svelte";
  import { whiteboardState, enterGroup } from "$lib/stores/whiteboard";
  import { showContextMenu } from "$lib/stores/ui";
  import type { Group } from "$lib/types";

  export let group: Group;

  let isDragging = false;
  let startPos = { x: 0, y: 0 };
  let startGroupPos = { x: 0, y: 0 };

  function handleMouseDown(event: MouseEvent) {
    if (event.button !== 0) return;
    isDragging = true;
    startPos = { x: event.clientX, y: event.clientY };
    startGroupPos = { x: group.position.x, y: group.position.y };
    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging) return;
    const dx = event.clientX - startPos.x;
    const dy = event.clientY - startPos.y;

    whiteboardState.update((state) => {
      if (state.groups[group.id]) {
        state.groups[group.id].position = {
          x: Math.max(0, startGroupPos.x + dx),
          y: Math.max(0, startGroupPos.y + dy),
        };
      }
      return state;
    });
  }

  async function handleMouseUp() {
    if (!isDragging) return;
    isDragging = false;
    window.removeEventListener("mousemove", handleMouseMove);
    window.removeEventListener("mouseup", handleMouseUp);

    try {
      await invoke("update_group", {
        id: group.id,
        position: group.position,
      });
    } catch (e) {
      console.error("Failed to update group position:", e);
    }
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    showContextMenu(event.clientX, event.clientY, { type: "group", id: group.id });
  }

  async function toggleCollapse() {
    const newCollapsed = !group.collapsed;
    whiteboardState.update((state) => {
      if (state.groups[group.id]) {
        state.groups[group.id].collapsed = newCollapsed;
      }
      return state;
    });

    try {
      await invoke("update_group", {
        id: group.id,
        collapsed: newCollapsed,
      });
    } catch (e) {
      console.error("Failed to toggle collapse:", e);
    }
  }

  function handleDoubleClick() {
    enterGroup(group.id);
  }

  $: childCount = Object.values($whiteboardState.items).filter(
    (item) => item.parentGroup === group.id
  ).length + Object.values($whiteboardState.groups).filter(
    (g) => g.parentGroup === group.id
  ).length;
</script>

<div
  class="group-container"
  class:dragging={isDragging}
  style="left: {group.position.x}px; top: {group.position.y}px;"
  role="button"
  tabindex="0"
  on:mousedown={handleMouseDown}
  on:contextmenu={handleContextMenu}
  on:dblclick={handleDoubleClick}
>
  {#if group.shortcut}
    <div class="shortcut-badge">{group.shortcut}</div>
  {/if}

  <div class="group-header">
    <button class="collapse-btn" on:click|stopPropagation={toggleCollapse}>
      {#if group.collapsed}
        <ChevronRight size={14} strokeWidth={1.5} />
      {:else}
        <ChevronDown size={14} strokeWidth={1.5} />
      {/if}
    </button>
    <Folder size={14} strokeWidth={1.5} />
    <span class="group-name">{group.name}</span>
    <span class="child-count">{childCount}</span>
  </div>

  <div class="group-body">
    {#if group.collapsed}
      <p>Double-click to expand</p>
    {:else}
      <p>Double-click to enter</p>
    {/if}
  </div>
</div>

<style>
  .group-container {
    position: absolute;
    min-width: 180px;
    background: rgba(59, 130, 246, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.3);
    border-radius: 8px;
    cursor: move;
    user-select: none;
    transition: box-shadow 0.15s ease;
  }

  .group-container:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .group-container.dragging {
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 50;
  }

  .shortcut-badge {
    position: absolute;
    top: -8px;
    right: -8px;
    padding: 2px 6px;
    background: #3b82f6;
    color: white;
    font-size: 10px;
    font-weight: 600;
    border-radius: 4px;
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    background: rgba(59, 130, 246, 0.15);
    border-bottom: 1px solid rgba(59, 130, 246, 0.2);
    border-radius: 8px 8px 0 0;
    color: #60a5fa;
  }

  .collapse-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: #60a5fa;
    cursor: pointer;
  }

  .collapse-btn:hover {
    background: rgba(59, 130, 246, 0.2);
  }

  .group-name {
    flex: 1;
    font-size: 13px;
    font-weight: 500;
    color: #d4d4d8;
  }

  .child-count {
    font-size: 11px;
    color: #71717a;
  }

  .group-body {
    padding: 12px;
    text-align: center;
  }

  .group-body p {
    margin: 0;
    font-size: 11px;
    color: #52525b;
  }
</style>
