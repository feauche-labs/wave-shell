<script lang="ts">
  /**
   * Wave: the layer over the terminal stage — splitters between panes (drag to resize), the drop preview (the
   * half a dragged tab will take) and the ghost of the dragged tab following the pointer.
   */
  import { untrack } from 'svelte';
  import Flag from '$lib/flags/Flag.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { sessions } from '$lib/stores/sessions.svelte';
  import { half, type Box, type Splitter } from './split-layout';
  import { waveSplit as split } from './split.svelte';

  let stage = $state<HTMLElement | null>(null);
  $effect(() => {
    split.stage = stage;
  });
  // Panes whose server went away (closed, disconnected) leave the layout.
  $effect(() => {
    void sessions.groups;
    untrack(() => split.prune());
  });

  const pct = (v: number) => `${(v * 100).toFixed(4)}%`;
  const preview = $derived.by((): Box | null => {
    const over = split.drag?.over;
    if (!over) return null;
    const b = split.targetBox(over.group);
    return b ? half(b, over.side) : null;
  });

  /** Splitter line: the border between the two halves of its split. Its ends stay 8px clear of the box
   *  corners so a border never lies over the one it meets (you always grab the border you aim at). */
  function line(s: Splitter): Box {
    return s.dir === 'row'
      ? { x: s.box.x + s.box.w * s.ratio, y: s.box.y, w: 0, h: s.box.h }
      : { x: s.box.x, y: s.box.y + s.box.h * s.ratio, w: s.box.w, h: 0 };
  }

  function grab(e: PointerEvent, s: Splitter) {
    if (!stage) return;
    e.preventDefault();
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    split.resizing = true;
    const r = stage.getBoundingClientRect();
    const move = (ev: PointerEvent) => {
      const ratio =
        s.dir === 'row'
          ? ((ev.clientX - r.left) / r.width - s.box.x) / s.box.w
          : ((ev.clientY - r.top) / r.height - s.box.y) / s.box.h;
      split.setRatio(s.path, ratio);
    };
    const up = () => {
      split.resizing = false;
      el.removeEventListener('pointermove', move);
      el.removeEventListener('pointerup', up);
      el.removeEventListener('pointercancel', up);
    };
    el.addEventListener('pointermove', move);
    el.addEventListener('pointerup', up);
    el.addEventListener('pointercancel', up);
  }
</script>

<div class="wave-stage" bind:this={stage} aria-hidden={split.drag ? undefined : 'true'}>
  {#each split.splitters as s (s.path)}
    {@const l = line(s)}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="splitter {s.dir}"
      title={m.wave_split_resize()}
      style:left={s.dir === 'col' ? `calc(${pct(l.x)} + 8px)` : pct(l.x)}
      style:top={s.dir === 'row' ? `calc(${pct(l.y)} + 8px)` : pct(l.y)}
      style:width={s.dir === 'col' ? `calc(${pct(l.w)} - 16px)` : null}
      style:height={s.dir === 'row' ? `calc(${pct(l.h)} - 16px)` : null}
      onpointerdown={(e) => grab(e, s)}
    ></div>
  {/each}
  {#if preview}
    <div
      class="preview"
      style:left="calc({pct(preview.x)} + 3px)"
      style:top="calc({pct(preview.y)} + 3px)"
      style:width="calc({pct(preview.w)} - 6px)"
      style:height="calc({pct(preview.h)} - 6px)"
    >
      <span
        >{split.drag && split.together(split.drag.key, split.drag.over?.group ?? null)
          ? m.wave_split_move()
          : m.wave_split_drop()}</span
      >
    </div>
  {/if}
</div>

{#if split.drag}
  <div class="ghost" style:left="{split.drag.x + 14}px" style:top="{split.drag.y + 10}px">
    <Flag code={split.drag.flag} size={13} />
    <span>{split.drag.label}</span>
    {#if split.drag.toBar}<small>· {m.wave_split_to_tab()}</small>{/if}
  </div>
{/if}

<style>
  .wave-stage {
    position: absolute;
    inset: 0;
    z-index: 5;
    pointer-events: none;
  }
  .splitter {
    position: absolute;
    z-index: 2;
    pointer-events: auto;
  }
  .splitter.row {
    width: 10px;
    margin-left: -5px;
    cursor: col-resize;
  }
  .splitter.col {
    height: 10px;
    margin-top: -5px;
    cursor: row-resize;
  }
  .splitter::after {
    content: '';
    position: absolute;
    border-radius: 2px;
    background: var(--accent);
    opacity: 0;
    transition: opacity 120ms var(--ease);
  }
  .splitter.row::after {
    inset: 8px 4px;
  }
  .splitter.col::after {
    inset: 4px 8px;
  }
  .splitter:hover::after {
    opacity: 0.6;
  }
  .preview {
    position: absolute;
    display: grid;
    place-items: center;
    border: 1.5px solid var(--accent);
    border-radius: 8px;
    background: color-mix(in oklab, var(--accent) 22%, transparent);
    transition:
      left 120ms var(--ease),
      top 120ms var(--ease),
      width 120ms var(--ease),
      height 120ms var(--ease);
  }
  .preview span {
    padding: 4px 10px;
    border-radius: 99px;
    background: color-mix(in oklab, var(--bg) 80%, transparent);
    color: var(--text);
    font-size: 12px;
    font-weight: 500;
  }
  .ghost {
    position: fixed;
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: 380px;
    padding: 5px 10px;
    border: 0.5px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    box-shadow: var(--shadow);
    color: var(--text);
    font-size: 12px;
    font-weight: 500;
    opacity: 0.85;
    pointer-events: none;
    white-space: nowrap;
  }
  .ghost small {
    color: var(--accent);
    font-size: 11.5px;
  }
  .ghost span {
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
