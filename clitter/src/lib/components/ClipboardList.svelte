<script lang="ts">
  import { filteredHistory } from "$lib/stores/clipboard";
  import { Clipboard } from "lucide-svelte";
  import ClipboardItem from "./ClipboardItem.svelte";
</script>

<div class="list-container">
  {#each $filteredHistory as item, index (item.id)}
    <ClipboardItem {item} {index} />
  {:else}
    <div class="empty-state">
      <Clipboard size={32} strokeWidth={1} />
      <p>No items</p>
    </div>
  {/each}
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
    color: #52525b;
  }

  .empty-state p {
    margin: 0;
    font-size: 12px;
  }
</style>
