<script lang="ts">
  /**
   * Wave: the frame of one terminal. Outside a workspace it is `display: contents` — the terminal fills the
   * stage exactly as before. Inside one it becomes a positioned pane with a header (flag, name, back to tab,
   * fill, disconnect). The terminal inside is the same component either way, so it is never re-created.
   */
  import { Maximize2, Minimize2, SquareArrowOutUpRight, X } from '@lucide/svelte';
  import type { Snippet } from 'svelte';
  import Flag from '$lib/flags/Flag.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { groupKey, sessions, type Tab } from '$lib/stores/sessions.svelte';
  import type { Box } from './split-layout';
  import { waveSplit } from './split.svelte';

  let { tab, box, children }: { tab: Tab; box: Box | null; children: Snippet } = $props();

  const key = $derived(groupKey(tab));
  const focused = $derived(sessions.activeId === tab.id);
  const maxed = $derived(waveSplit.current?.max === key);
  const pct = (v: number) => `${(v * 100).toFixed(4)}%`;
</script>

<div
  class="frame"
  class:split={!!box}
  class:focused={!!box && focused}
  data-wave-pane={box ? key : undefined}
  style:left={box ? `calc(${pct(box.x)} + 3px)` : null}
  style:top={box ? `calc(${pct(box.y)} + 3px)` : null}
  style:width={box ? `calc(${pct(box.w)} - 6px)` : null}
  style:height={box ? `calc(${pct(box.h)} - 6px)` : null}
  onpointerdowncapture={() => box && waveSplit.focus(key)}
>
  {#if box}
    <header>
      <Flag code={tab.flag} size={13} />
      <span class="name">{tab.label}</span>
      <span class="sp"></span>
      <button
        type="button"
        class="icon-btn"
        title={m.wave_split_return()}
        aria-label={m.wave_split_return()}
        onclick={() => waveSplit.detach(key)}><SquareArrowOutUpRight size={13} /></button
      >
      <button
        type="button"
        class="icon-btn"
        title={maxed ? m.wave_split_restore() : m.wave_split_max()}
        aria-label={maxed ? m.wave_split_restore() : m.wave_split_max()}
        onclick={() => waveSplit.toggleMax(key)}
        >{#if maxed}<Minimize2 size={13} />{:else}<Maximize2 size={13} />{/if}</button
      >
      <button
        type="button"
        class="icon-btn"
        title={m.wave_split_close()}
        aria-label={m.wave_split_close()}
        onclick={() => waveSplit.close(key)}><X size={13} /></button
      >
    </header>
  {/if}
  <!-- One element either way: the terminal inside is never re-created. -->
  <div class="body">{@render children()}</div>
</div>

<style>
  .frame:not(.split),
  .frame:not(.split) > .body {
    display: contents;
  }
  .frame.split {
    position: absolute;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--sep);
    border-radius: 8px;
    background: var(--t-bg, var(--bg));
  }
  .frame.focused {
    border-color: color-mix(in oklab, var(--accent) 70%, transparent);
  }
  header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 4px 0 10px;
    border-bottom: 0.5px solid var(--sep);
    color: var(--muted);
    font-size: 11.5px;
  }
  .focused header {
    color: var(--text);
  }
  .name {
    overflow: hidden;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sp {
    flex: 1;
  }
  header .icon-btn {
    width: 22px;
    height: 22px;
    opacity: 0.6;
  }
  .frame:hover header .icon-btn,
  .focused header .icon-btn {
    opacity: 1;
  }
  .body {
    position: relative;
    flex: 1;
    min-height: 0;
  }
</style>
