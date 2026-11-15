<script lang="ts">
  /**
   * Wave: tags of a host as chips inside one field (like Termius). Typing shows existing tags with their host
   * counts; Enter or a click adds the highlighted one, or creates a new tag; Backspace in an empty field removes
   * the last chip. Works with names — the editor turns them into ids on save (waveTags.ensure).
   */
  import { Tag as TagIcon, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { waveTags } from './tags.svelte';

  let { names = $bindable([]), id }: { names: string[]; id?: string } = $props();

  let text = $state('');
  let open = $state(false);
  let active = $state(0);
  let input = $state<HTMLInputElement | null>(null);

  const typed = $derived(text.trim());
  const options = $derived(waveTags.suggest(typed, names).slice(0, 8));
  const canCreate = $derived(
    !!typed &&
      !names.some((n) => n.toLocaleLowerCase() === typed.toLocaleLowerCase()) &&
      !waveTags.list.some((t) => t.name.toLocaleLowerCase() === typed.toLocaleLowerCase())
  );
  /** Rows of the list: existing tags, then «create». */
  const rows = $derived([
    ...options.map((t) => ({ name: t.name, count: waveTags.counts.get(t.id) ?? 0, create: false })),
    ...(canCreate ? [{ name: typed, count: 0, create: true }] : [])
  ]);

  function add(name: string) {
    const n = name.trim();
    if (n && !names.some((x) => x.toLocaleLowerCase() === n.toLocaleLowerCase()))
      names = [...names, n];
    text = '';
    active = 0;
    input?.focus();
  }

  function remove(i: number) {
    names = names.filter((_, j) => j !== i);
    input?.focus();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      if (!typed && e.key === 'Enter') return; // let the form submit
      e.preventDefault();
      const row = open ? rows[active] : undefined;
      add(row ? row.name : typed);
    } else if (e.key === 'Backspace' && !text && names.length) {
      e.preventDefault();
      names = names.slice(0, -1);
    } else if (e.key === 'ArrowDown' && rows.length) {
      e.preventDefault();
      open = true;
      active = (active + 1) % rows.length;
    } else if (e.key === 'ArrowUp' && rows.length) {
      e.preventDefault();
      active = (active - 1 + rows.length) % rows.length;
    } else if (e.key === 'Escape' && open) {
      e.preventDefault();
      e.stopPropagation();
      open = false;
    }
  }
</script>

<div class="wrap">
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="input tags" onclick={() => input?.focus()}>
    <span class="ic"><TagIcon size={14} /></span>
    {#each names as n, i (n)}
      <span class="chip"
        >{n}<button
          type="button"
          aria-label={m.wave_tags_remove({ name: n })}
          onclick={(e) => (e.stopPropagation(), remove(i))}><X size={11} /></button
        ></span
      >
    {/each}
    <input
      {id}
      bind:this={input}
      bind:value={text}
      placeholder={names.length ? '' : m.wave_tags_placeholder()}
      autocomplete="off"
      spellcheck="false"
      role="combobox"
      aria-expanded={open && rows.length > 0}
      aria-controls="wave-tag-list"
      oninput={() => ((open = true), (active = 0))}
      onfocus={() => (open = true)}
      onblur={() => setTimeout(() => (open = false), 120)}
      {onkeydown}
    />
  </div>
  {#if open && rows.length}
    <div class="drop" id="wave-tag-list" role="listbox">
      {#each rows as r, i (r.create ? '+' + r.name : r.name)}
        <button
          type="button"
          role="option"
          aria-selected={i === active}
          class:on={i === active}
          class:create={r.create}
          onmousedown={(e) => e.preventDefault()}
          onmouseenter={() => (active = i)}
          onclick={() => add(r.name)}
        >
          {#if r.create}{m.wave_tags_create({ name: r.name })}{:else}<span>{r.name}</span><small
              >{r.count || ''}</small
            >{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .wrap {
    position: relative;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 5px;
    height: auto;
    min-height: var(--control-h);
    padding: 4px 8px;
    cursor: text;
  }
  .ic {
    display: grid;
    place-items: center;
    color: var(--muted);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    height: 22px;
    padding: 0 3px 0 9px;
    border-radius: 99px;
    background: var(--fill-2);
    color: var(--text);
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }
  .chip button {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: none;
    color: var(--muted);
    cursor: pointer;
  }
  .chip button:hover {
    background: var(--hover);
    color: var(--text);
  }
  input {
    flex: 1;
    min-width: 80px;
    height: 22px;
    padding: 0;
    border: 0;
    outline: 0;
    background: none;
    color: var(--text);
    font: inherit;
  }
  .drop {
    position: absolute;
    left: 0;
    right: 0;
    top: calc(100% + 4px);
    z-index: 60;
    display: grid;
    max-height: 240px;
    overflow: auto;
    padding: 4px;
    border: 0.5px solid var(--border);
    border-radius: var(--r-control);
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  .drop button {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }
  .drop button small {
    color: var(--muted);
    font-size: 11px;
  }
  .drop button.on {
    background: var(--accent);
    color: #fff;
  }
  .drop button.on small {
    color: rgb(255 255 255 / 0.8);
  }
  .drop button.create:not(.on) {
    color: var(--accent);
  }
</style>
