<script lang="ts">
  /**
   * Wave: an icon or an emoji in place of a flag (`icon:server`, `emoji:🏠`), drawn by Flag.svelte. The icon
   * takes `--look-color` (a folder colour) or the muted text colour.
   */
  import { ICON_BY_ID, parseLook } from './catalog';

  let { look, size = 16, title }: { look: string; size?: number; title?: string } = $props();

  const l = $derived(parseLook(look));
  const entry = $derived(l?.kind === 'icon' ? ICON_BY_ID.get(l.value) : undefined);
</script>

{#if l?.kind === 'emoji'}
  <span
    class="look emoji"
    style:width="{size}px"
    style:height="{size}px"
    style:font-size="{Math.round(size * 0.9)}px"
    role={title ? 'img' : undefined}
    aria-label={title}
    aria-hidden={title ? undefined : 'true'}>{l.value}</span
  >
{:else if entry}
  {@const Icon = entry.icon}
  <span
    class="look ic"
    style:width="{size}px"
    style:height="{size}px"
    role={title ? 'img' : undefined}
    aria-label={title}
    aria-hidden={title ? undefined : 'true'}><Icon {size} strokeWidth={1.9} /></span
  >
{/if}

<style>
  .look {
    display: inline-grid;
    flex: none;
    place-items: center;
    line-height: 1;
    vertical-align: middle;
  }
  .ic {
    color: var(--look-color, var(--muted));
  }
</style>
