<script lang="ts">
  import { filteredHistory, canScrollUp, canScrollDown, listScrollOffset, selectedIndex } from "$lib/stores/clipboard";
  import { Clipboard, ChevronUp, ChevronDown } from "lucide-svelte";
  import ClipboardItem from "./ClipboardItem.svelte";
</script>

<div class="list-container">
  {#if $canScrollUp}
    <div class="scroll-indicator top">
      <ChevronUp size={14} strokeWidth={1.5} />
      <span>+{$listScrollOffset} items above (Shift+↑)</span>
    </div>
  {/if}

  {#each $filteredHistory as item, index (item.id)}
    <ClipboardItem {item} {index} selected={index === $selectedIndex} />
  {:else}
    <div class="empty-state">
      <Clipboard size={32} strokeWidth={1} />
      <p>No items</p>
    </div>
  {/each}

  {#if $canScrollDown}
    <div class="scroll-indicator bottom">
      <ChevronDown size={14} strokeWidth={1.5} />
      <span>more items below (Shift+↓)</span>
    </div>
  {/if}
</div>

<style>
  .list-container {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--text-muted);
  }

  .empty-state p {
    margin: 0;
    font-size: 12px;
  }

  .scroll-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 4px 8px;
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-hover);
    border-radius: 4px;
  }

  .scroll-indicator.top {
    margin-bottom: 4px;
  }

  .scroll-indicator.bottom {
    margin-top: 4px;
  }
</style>
