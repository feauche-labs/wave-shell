<script lang="ts">
  /**
   * Wave: the frame of one terminal. Outside a workspace it is `display: contents` — the terminal fills the
   * stage exactly as before. Inside one it becomes a positioned pane with a header (flag, name, user@host,
   * «Ввод во все окна», back to tab, fill, disconnect). The header drags: onto another pane — the pane moves
   * there; onto the tab bar — it returns to its own tab. The terminal inside is never re-created.
   */
  import { Keyboard, Maximize2, Minimize2, SquareArrowOutUpRight, X } from '@lucide/svelte';
  import type { Snippet } from 'svelte';
  import Flag from '$lib/flags/Flag.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { groupKey, sessions, type Tab } from '$lib/stores/sessions.svelte';
  import { tree } from '$lib/stores/tree.svelte';
  import type { Box } from './split-layout';
  import { waveSplit } from './split.svelte';

  let { tab, box, children }: { tab: Tab; box: Box | null; children: Snippet } = $props();

  const key = $derived(groupKey(tab));
  const focused = $derived(sessions.activeId === tab.id);
  const maxed = $derived(waveSplit.current?.max === key);
  const cast = $derived(!!box && waveSplit.broadcasting(key));
  const lifted = $derived(waveSplit.drag?.key === key && !waveSplit.drag.over);
  const host = $derived(tab.hostId ? tree.data.hosts.find((h) => h.id === tab.hostId) : undefined);
  const meta = $derived(
    host
      ? `${host.username ? host.username + '@' : ''}${host.hostname}${host.port && host.port !== 22 ? ':' + host.port : ''}`
      : ''
  );
  /** Typing goes on in the terminal after a header button. */
  function backToTerminal(e: Event) {
    const el = (e.currentTarget as Element).closest('.frame');
    el?.querySelector<HTMLElement>('.xterm-helper-textarea')?.focus();
  }
  const pct = (v: number) => `${(v * 100).toFixed(4)}%`;
</script>

<div
  class="frame"
  class:split={!!box}
  class:focused={!!box && focused}
  class:cast
  class:lifted
  class:resizing={waveSplit.resizing}
  data-wave-pane={box ? key : undefined}
  style:left={box ? `calc(${pct(box.x)} + 3px)` : null}
  style:top={box ? `calc(${pct(box.y)} + 3px)` : null}
  style:width={box ? `calc(${pct(box.w)} - 6px)` : null}
  style:height={box ? `calc(${pct(box.h)} - 6px)` : null}
  onpointerdowncapture={() => box && waveSplit.focus(key)}
>
  {#if box}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header
      title={m.wave_split_grab()}
      onpointerdown={(e) => waveSplit.grab(e, key, tab.label, tab.flag)}
    >
      <span class="who">
        <Flag code={tab.flag} size={14} />
        <span class="name">{tab.label}</span>
      </span>
      {#if meta}<span class="meta mono">{meta}</span>{/if}
      <span class="sp"></span>
      <div class="tools">
        <button
          type="button"
          class="cast-btn"
          class:on={cast}
          aria-pressed={cast}
          title={cast ? m.wave_split_broadcast_on() : m.wave_split_broadcast_off()}
          aria-label={m.wave_split_broadcast()}
          onclick={(e) => {
            waveSplit.toggleBroadcast(key);
            backToTerminal(e);
          }}
        >
          <Keyboard size={13} /><span>{m.wave_split_broadcast()}</span>
        </button>
        <span class="div"></span>
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
      </div>
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
    --cast: var(--warn);
    position: absolute;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--sep);
    border-radius: 10px;
    background: var(--t-bg, var(--bg));
    transition:
      left 200ms var(--ease),
      top 200ms var(--ease),
      width 200ms var(--ease),
      height 200ms var(--ease),
      border-color 120ms var(--ease),
      opacity 120ms var(--ease);
  }
  .frame.split.resizing {
    transition: none;
  }
  .frame.focused {
    border-color: color-mix(in oklab, var(--accent) 70%, transparent);
  }
  .frame.cast {
    border-color: color-mix(in oklab, var(--cast) 75%, transparent);
  }
  .frame.lifted {
    opacity: 0.55;
  }
  header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 6px 0 12px;
    color: var(--muted);
    font-size: 12px;
    cursor: grab;
    user-select: none;
  }
  header:active {
    cursor: grabbing;
  }
  .who {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
  }
  .name {
    overflow: hidden;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .focused .name {
    color: var(--text);
  }
  .meta {
    overflow: hidden;
    font-size: 11px;
    opacity: 0.75;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sp {
    flex: 1;
  }
  .tools {
    display: flex;
    flex: none;
    align-items: center;
    gap: 2px;
    cursor: default;
  }
  .tools .icon-btn {
    width: 24px;
    height: 24px;
    border-radius: 6px;
  }
  .div {
    width: 0.5px;
    height: 14px;
    margin: 0 4px;
    background: var(--sep);
  }
  .cast-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    padding: 0 9px;
    border: 0.5px solid var(--sep);
    border-radius: 99px;
    background: none;
    color: var(--muted);
    font: inherit;
    font-size: 11px;
    white-space: nowrap;
    cursor: pointer;
  }
  .cast-btn:hover {
    color: var(--text);
    background: var(--fill-1, rgba(127, 127, 127, 0.12));
  }
  .cast-btn.on {
    border-color: transparent;
    background: color-mix(in oklab, var(--cast) 22%, transparent);
    color: var(--cast);
  }
  .body {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  /* Like Termius: a slightly lifted bar; the tools of a pane you are not in are dimmed. */
  header {
    background: color-mix(in oklab, var(--text) 4%, var(--t-bg, var(--bg)));
    border-bottom: 0.5px solid var(--sep);
  }
  .tools .icon-btn,
  .cast-btn:not(.on) {
    opacity: 0.55;
  }
  .frame.split:hover .tools .icon-btn,
  .frame.focused .tools .icon-btn,
  .frame.split:hover .cast-btn,
  .frame.focused .cast-btn {
    opacity: 1;
  }
</style>
