<script lang="ts">
  import { filteredHistory } from "$lib/stores/clipboard";
  import ClipboardItem from "./ClipboardItem.svelte";
</script>

<div class="flex-1 overflow-y-auto p-4 space-y-2">
  {#each $filteredHistory as item, index (item.id)}
    <ClipboardItem {item} {index} />
  {:else}
    <div class="text-center text-gray-400 py-8">
      履歴がありません
    </div>
  {/each}
</div>
