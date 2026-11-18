<!--
  Wave: the macOS window buttons drawn by us, in the header of the left menu (like in Arc). The system ones are
  pinned to the window and cannot slide with the menu; these are part of it. They behave like the system ones:
  signs on hover, grey in an inactive window, green — full screen (with ⌥ — zoom). Shell hides the system ones.
-->
<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { ui } from '$lib/stores/ui.svelte';

  const win = getCurrentWindow();
  let focused = $state(document.hasFocus());
  let full = $state(false);

  onMount(() => {
    let live = true;
    const sync = () =>
      void win
        .isFullscreen()
        .then((v) => live && (full = v))
        .catch(() => undefined);
    const offs = [
      win.onFocusChanged(({ payload }) => (focused = payload)),
      // Full screen comes and goes with a resize.
      win.onResized(sync)
    ];
    sync();
    return () => {
      live = false;
      for (const off of offs) void off.then((stop) => stop()).catch(() => undefined);
    };
  });

  function run(action: () => Promise<void>) {
    void action().catch(() => ui.toast(m.err_generic(), 'error'));
  }
</script>

<div class="lights" class:blur={!focused} role="group" aria-label={m.window_controls()}>
  <button
    type="button"
    class="light close"
    aria-label={m.window_close()}
    onclick={() => run(() => win.close())}
  >
    <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M3.5 3.5l5 5M8.5 3.5l-5 5" /></svg>
  </button>
  <button
    type="button"
    class="light min"
    aria-label={m.window_minimize()}
    onclick={() => run(() => win.minimize())}
  >
    <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.8 6h6.4" /></svg>
  </button>
  <button
    type="button"
    class="light zoom"
    aria-label={full ? m.window_restore() : m.wave_menu_fullscreen()}
    onclick={(e) => run(() => (e.altKey ? win.toggleMaximize() : win.setFullscreen(!full)))}
  >
    <svg viewBox="0 0 12 12" aria-hidden="true" class="fill">
      {#if full}
        <path d="M5.4 5.4V2.6L2.6 5.4zM6.6 6.6v2.8l2.8-2.8z" />
      {:else}
        <path d="M3.3 3.3h3.4L3.3 6.7zM8.7 8.7H5.3l3.4-3.4z" />
      {/if}
    </svg>
  </button>
</div>

<style>
  .lights {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: none;
    margin: 0 10px 0 4px;
  }
  .light {
    position: relative;
    display: grid;
    place-items: center;
    width: 12px;
    height: 12px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: var(--c);
    box-shadow: inset 0 0 0 0.5px var(--ring);
    cursor: default;
  }
  .light:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .light:active {
    filter: brightness(0.82);
  }
  .close {
    --c: #ff5f57;
    --ring: #e2463f;
  }
  .min {
    --c: #febc2e;
    --ring: #e1a116;
  }
  .zoom {
    --c: #28c840;
    --ring: #1aac2c;
  }
  /* An inactive window: grey, until the pointer comes. */
  .blur:not(:hover) .light {
    --c: color-mix(in oklab, var(--text) 20%, transparent);
    --ring: color-mix(in oklab, var(--text) 10%, transparent);
  }
  svg {
    width: 12px;
    height: 12px;
    opacity: 0;
    fill: none;
    stroke: rgb(0 0 0 / 0.55);
    stroke-width: 1.2;
    stroke-linecap: round;
  }
  svg.fill {
    fill: rgb(0 0 0 / 0.55);
    stroke: none;
  }
  .lights:hover svg,
  .light:focus-visible svg {
    opacity: 1;
  }
</style>
