<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    whiteboardState,
    shortcutInput,
    matchedShortcuts,
    focusedGroupId,
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

  // Get visible items/groups based on focus
  $: visibleItems = Object.values($whiteboardState.items).filter(
    (item) => item.parentGroup === $focusedGroupId
  );

  $: visibleGroups = Object.values($whiteboardState.groups).filter(
    (group) => group.parentGroup === $focusedGroupId
  );

  $: currentGroupName = $focusedGroupId
    ? $whiteboardState.groups[$focusedGroupId]?.name
    : null;
</script>

<div class="flex-1 flex flex-col overflow-hidden">
  <!-- Shortcut input display -->
  <div class="px-4 py-2 bg-gray-100 border-b border-gray-200">
    <div class="flex items-center gap-2">
      {#if currentGroupName}
        <span class="text-sm text-gray-500">📁 {currentGroupName} /</span>
      {/if}
      <span class="font-mono text-lg">
        {$shortcutInput || "_"}
      </span>
      {#if $matchedShortcuts.length > 0 && $shortcutInput}
        <span class="text-sm text-gray-400">
          ({$matchedShortcuts.length}件マッチ)
        </span>
      {/if}
    </div>
    {#if $matchedShortcuts.length > 0 && $shortcutInput}
      <div class="flex gap-2 mt-1 flex-wrap">
        {#each $matchedShortcuts.slice(0, 5) as match}
          <span
            class="text-xs px-2 py-0.5 rounded
              {match.type === 'group' ? 'bg-blue-100 text-blue-700' : 'bg-gray-200 text-gray-700'}"
          >
            [{match.shortcut}] {match.name}
          </span>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Whiteboard canvas -->
  <div
    bind:this={whiteboardEl}
    class="flex-1 relative overflow-auto bg-gray-50"
    on:contextmenu={handleContextMenu}
    on:drop={handleDrop}
    on:dragover={handleDragOver}
  >
    {#if visibleItems.length === 0 && visibleGroups.length === 0}
      <div class="absolute inset-0 flex items-center justify-center text-gray-400">
        <div class="text-center">
          <p class="text-lg mb-2">ホワイトボード</p>
          <p class="text-sm">履歴からアイテムをドラッグ&ドロップ</p>
          <p class="text-sm">右クリックでグループ追加</p>
        </div>
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
