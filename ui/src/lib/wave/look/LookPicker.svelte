<script lang="ts">
  /**
   * Wave: «Значок» — a flag, an icon or an emoji for a host or a folder (showcase: variant A). Search on top,
   * tabs «Значки · Флаги · Смайлики», recent ones, a grid. The value goes to the same field the flag used.
   */
  import { Popover } from 'bits-ui';
  import { ChevronDown, Search, X } from '@lucide/svelte';
  import Flag from '$lib/flags/Flag.svelte';
  import { flagName, searchFlags, type FlagLocale } from '$lib/flags/flags';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import * as m from '$lib/paraglide/messages.js';
  import { settings } from '$lib/stores/settings.svelte';
  import { lookName, parseLook, searchEmoji, searchIcons } from './catalog';

  interface Props {
    /** Own look (null = none / inherit): `de`, `icon:server`, `emoji:🏠`. */
    value: string | null;
    /** Inherited look shown dimmed when `value` is null. */
    inherited?: { value: string; from: string | null } | null;
    onchange: (v: string | null) => void;
  }
  let { value, inherited = null, onchange }: Props = $props();

  const locale = getLocale() as FlagLocale;
  let open = $state(false);
  let q = $state('');
  let tab = $state<'icon' | 'flag' | 'emoji'>('icon');

  /** Flags are stored bare (`de`) as before; icons and emoji with their prefix. */
  const asLook = (v: string) => (parseLook(v)?.kind === 'flag' ? 'flag:' + v : v);
  const name = (v: string) => lookName(asLook(v), locale, flagName);
  const shown = $derived(value ?? inherited?.value ?? null);

  const list = $derived(
    tab === 'icon' ? searchIcons(q) : tab === 'emoji' ? searchEmoji(q) : searchFlags(q)
  );
  const recent = $derived(settings.recentFlags.filter((v) => !!parseLook(v)).slice(0, 8));

  function choose(v: string | null) {
    if (v) settings.pushRecentFlag(v);
    onchange(v);
    open = false;
  }
  function key(e: KeyboardEvent) {
    if (e.key !== 'Enter') return;
    e.preventDefault();
    if (list[0]) choose(list[0]);
  }
  const focusOnMount = (n: HTMLInputElement) => n.focus();
  // A search for a country switches to flags, for something else back to icons.
  $effect(() => {
    if (!q.trim()) return;
    if (tab !== 'flag' && !list.length && searchFlags(q).length) tab = 'flag';
  });
</script>

<Popover.Root bind:open onOpenChange={(o) => o && (q = '')}>
  <Popover.Trigger class="input lp-trigger" aria-label={m.wave_look_field()}>
    <span class="lp-val" class:inh={!value && inherited}>
      <Flag code={shown} size={18} />
      <span class="lp-name">
        {#if value}{name(value)}{:else if inherited}{m.inherited_from({
            name: inherited.from ?? '',
            value: name(inherited.value)
          })}{:else}{m.flag_none()}{/if}
      </span>
    </span>
    <ChevronDown size={14} />
  </Popover.Trigger>
  <Popover.Portal>
    <Popover.Content class="menu lp-pop" sideOffset={6} align="start">
      <label class="lp-search"
        ><Search size={14} /><input
          type="text"
          placeholder={m.wave_look_search()}
          bind:value={q}
          onkeydown={key}
          use:focusOnMount
        /></label
      >
      <div class="lp-seg" role="tablist">
        <button
          role="tab"
          aria-selected={tab === 'icon'}
          class:on={tab === 'icon'}
          onclick={() => (tab = 'icon')}>{m.wave_look_icons()}</button
        >
        <button
          role="tab"
          aria-selected={tab === 'flag'}
          class:on={tab === 'flag'}
          onclick={() => (tab = 'flag')}>{m.wave_look_flags()}</button
        >
        <button
          role="tab"
          aria-selected={tab === 'emoji'}
          class:on={tab === 'emoji'}
          onclick={() => (tab = 'emoji')}>{m.wave_look_emoji()}</button
        >
      </div>
      {#if !q && recent.length}
        <div class="lp-sub">{m.flag_recent()}</div>
        <div class="lp-grid">
          {#each recent as v (v)}
            <button class="lp-cell" class:on={value === v} title={name(v)} onclick={() => choose(v)}
              ><Flag code={v} size={20} /></button
            >
          {/each}
        </div>
      {/if}
      <div class="lp-grid lp-scroll">
        {#each list as v (v)}
          <button class="lp-cell" class:on={value === v} title={name(v)} onclick={() => choose(v)}
            ><Flag code={v} size={20} /></button
          >
        {:else}
          <p class="lp-none">{m.flag_no_results()}</p>
        {/each}
      </div>
      <button class="lp-clear" onclick={() => choose(null)}
        ><X size={13} />{m.wave_look_none()}</button
      >
    </Popover.Content>
  </Popover.Portal>
</Popover.Root>

<style>
  :global(.lp-trigger) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    height: var(--control-h);
    cursor: pointer;
    text-align: left;
  }
  .lp-val {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .lp-val.inh :global(.flag),
  .lp-val.inh :global(.look) {
    opacity: 0.7;
  }
  .lp-val.inh .lp-name {
    color: var(--muted);
  }
  .lp-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :global(.lp-pop) {
    display: grid;
    gap: 8px;
    width: 316px;
    padding: 8px;
  }
  .lp-search {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 9px;
    border-radius: 8px;
    background: var(--fill-2);
    color: var(--muted);
  }
  .lp-search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 13px;
  }
  .lp-seg {
    display: flex;
    padding: 2px;
    border-radius: 8px;
    background: var(--fill-2);
  }
  .lp-seg button {
    flex: 1;
    height: 24px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--muted);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .lp-seg button.on {
    background: var(--surface);
    box-shadow: var(--card-shadow);
    color: var(--text);
  }
  .lp-sub {
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
  }
  .lp-grid {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 2px;
  }
  .lp-scroll {
    max-height: 220px;
    overflow-y: auto;
  }
  .lp-cell {
    display: grid;
    place-items: center;
    aspect-ratio: 1;
    border: 0;
    border-radius: 8px;
    background: none;
    cursor: pointer;
  }
  .lp-cell:hover {
    background: var(--hover);
  }
  .lp-cell.on {
    background: color-mix(in oklab, var(--accent) 24%, transparent);
  }
  .lp-none {
    grid-column: 1 / -1;
    margin: 6px 4px;
    color: var(--muted);
    font-size: 12px;
  }
  .lp-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 28px;
    border: 0;
    border-radius: 8px;
    background: var(--fill-2);
    color: var(--muted);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .lp-clear:hover {
    color: var(--text);
  }
</style>
