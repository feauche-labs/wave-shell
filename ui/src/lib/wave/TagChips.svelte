<script lang="ts">
  /** Wave: a host's tags as small chips (Overview row). */
  import { waveTags } from './tags.svelte';

  let { ids }: { ids: readonly string[] | undefined } = $props();
  const names = $derived(waveTags.names(ids));
</script>

{#if names.length}
  <span class="chips"
    >{#each names as n (n)}<span class="chip">{n}</span>{/each}</span
  >
{/if}

<style>
  .chips {
    display: inline-flex;
    flex-wrap: nowrap;
    gap: 4px;
    min-width: 0;
    overflow: hidden;
  }
  .chip {
    flex: none;
    height: 18px;
    padding: 0 7px;
    border-radius: 99px;
    background: var(--fill-2);
    color: var(--muted);
    font-size: 11px;
    font-weight: 500;
    line-height: 18px;
    white-space: nowrap;
  }
</style>
