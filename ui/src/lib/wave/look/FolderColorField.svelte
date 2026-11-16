<script lang="ts">
  /** Wave: the folder colour — «без цвета» and ten soft colours. */
  import { X } from '@lucide/svelte';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import * as m from '$lib/paraglide/messages.js';
  import { COLORS } from './catalog';

  let { color = $bindable(null) }: { color: string | null } = $props();
  const ru = getLocale() === 'ru';
</script>

<div class="swatches" role="radiogroup" aria-label={m.wave_color_field()}>
  <button
    type="button"
    class="sw none"
    role="radio"
    aria-checked={!color}
    class:on={!color}
    title={m.wave_color_none()}
    onclick={() => (color = null)}><X size={13} /></button
  >
  {#each COLORS as c (c.id)}
    <button
      type="button"
      class="sw"
      role="radio"
      aria-checked={color === c.id}
      class:on={color === c.id}
      title={ru ? c.ru : c.en}
      style:--hex={c.hex}
      onclick={() => (color = c.id)}><span></span></button
    >
  {/each}
</div>

<style>
  /* Six in a row: «no colour» + five, then five — two even rows in the narrow editor. */
  .swatches {
    display: grid;
    grid-template-columns: repeat(6, 28px);
    gap: 8px 10px;
  }
  .sw {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: color-mix(in oklab, var(--hex) 30%, transparent);
    cursor: pointer;
    transition: transform 120ms var(--ease);
  }
  .sw span {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--hex);
    opacity: 0.9;
  }
  .sw.none {
    background: var(--fill-2);
    color: var(--muted);
  }
  .sw:hover {
    transform: scale(1.08);
  }
  .sw.on {
    box-shadow:
      0 0 0 2px var(--bg),
      0 0 0 3.5px var(--hex, var(--muted));
  }
</style>
