<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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

  // Count children
  $: childCount = Object.values($whiteboardState.items).filter(
    (item) => item.parentGroup === group.id
  ).length + Object.values($whiteboardState.groups).filter(
    (g) => g.parentGroup === group.id
  ).length;
</script>

<div
  class="group-container absolute rounded-lg shadow-md border-2 border-blue-300 bg-blue-50
    cursor-move select-none
    {isDragging ? 'shadow-xl z-50' : ''}"
  style="left: {group.position.x}px; top: {group.position.y}px; min-width: 200px;"
  role="button"
  tabindex="0"
  on:mousedown={handleMouseDown}
  on:contextmenu={handleContextMenu}
  on:dblclick={handleDoubleClick}
>
  <!-- Shortcut badge -->
  {#if group.shortcut}
    <div class="absolute -top-2 -right-2 px-2 py-0.5 bg-blue-500 text-white text-xs rounded-full font-bold">
      {group.shortcut}
    </div>
  {/if}

  <!-- Header -->
  <div class="flex items-center gap-2 px-3 py-2 bg-blue-100 rounded-t-lg border-b border-blue-200">
    <button
      class="text-blue-600 hover:text-blue-800"
      on:click|stopPropagation={toggleCollapse}
    >
      {group.collapsed ? "▶" : "▼"}
    </button>
    <span class="text-lg">📁</span>
    <span class="font-medium text-gray-800 flex-1">{group.name}</span>
    <span class="text-xs text-gray-500">({childCount})</span>
  </div>

  <!-- Content (collapsed indicator) -->
  {#if group.collapsed}
    <div class="px-3 py-2 text-sm text-gray-500 text-center">
      ダブルクリックで展開
    </div>
  {:else}
    <div class="px-3 py-2 text-sm text-gray-500 text-center min-h-[60px]">
      ダブルクリックで中に入る
    </div>
  {/if}
</div>

<style>
  .group-container {
    transition: box-shadow 0.2s ease, transform 0.1s ease;
  }

  .group-container:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
</style>
