<script lang="ts">
  /**
   * Wave: the Snippets page as blocks (like Termius): folder tiles and snippet tiles instead of one long list.
   * A click on a snippet runs it (asks where); the side panel next to a terminal keeps the compact list.
   */
  import {
    Folder,
    FolderPlus,
    Pencil,
    Plus,
    Search,
    Server,
    SquareTerminal,
    Trash2,
    Zap
  } from '@lucide/svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { pluralForm } from '$lib/stats/format';
  import { parseVars, sections } from '$lib/snippets/model';
  import { snippets } from '$lib/snippets/snippets.svelte';

  type Snip = (typeof snippets.items)[number];

  let q = $state('');
  /** The folder unfolded under the tiles (remembered). */
  const OPEN_KEY = 'wave.snip.board.open';
  let opened = $state<string | null>(
    (() => {
      try {
        return localStorage.getItem(OPEN_KEY);
      } catch {
        return null;
      }
    })()
  );
  $effect(() => {
    try {
      if (opened) localStorage.setItem(OPEN_KEY, opened);
      else localStorage.removeItem(OPEN_KEY);
    } catch {
      /* storage unavailable */
    }
  });

  const secs = $derived(sections(snippets.items, snippets.layout, q));
  const groups = $derived(snippets.layout.groups);
  const loose = $derived(secs.find((x) => !x.group)?.items ?? []);
  const itemsOf = (id: string) => secs.find((x) => x.group?.id === id)?.items ?? [];
  const found = $derived(q ? secs.flatMap((x) => x.items) : []);
  const openGroup = $derived(groups.find((g) => g.id === opened) ?? null);
  const firstLine = (b: string) => b.split('\n').find((l) => l.trim()) ?? '';
  // ---- drag a snippet: onto a folder tile (or the open folder) — into it; onto «Сниппеты» — out of folders;
  // onto another snippet — next to it. Pointer events: the webview's native drag goes to Tauri file drops.
  interface Drag {
    id: string;
    label: string;
    x: number;
    y: number;
    /** `s:<id>` snippet, `f:<id>` folder, `n:` no folder. */
    over: string | null;
    /** Pointer on the right half of a snippet tile: drop after it. */
    after: boolean;
  }
  let drag = $state.raw<Drag | null>(null);
  let suppressClick = false;

  function down(e: PointerEvent, sn: Snip) {
    if (e.button !== 0 || (e.target as Element).closest('.tacts')) return;
    const [sx, sy] = [e.clientX, e.clientY];
    const move = (ev: PointerEvent) => {
      if (!drag && Math.hypot(ev.clientX - sx, ev.clientY - sy) < 5) return;
      if (!drag) window.getSelection()?.removeAllRanges();
      const el = document.elementFromPoint(ev.clientX, ev.clientY);
      const t = el instanceof Element ? el.closest<HTMLElement>('[data-drop]') : null;
      const over = t?.dataset.drop ?? null;
      let after = false;
      if (t && over?.startsWith('s:')) {
        const r = t.getBoundingClientRect();
        after = ev.clientX > r.left + r.width / 2;
      }
      drag = { id: sn.id, label: sn.name, x: ev.clientX, y: ev.clientY, over, after };
    };
    const finish = (commit: boolean) => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      window.removeEventListener('pointercancel', cancel);
      const d = drag;
      drag = null;
      if (!d) return;
      suppressClick = true;
      setTimeout(() => (suppressClick = false), 0);
      if (commit && d.over) drop(d);
    };
    const up = () => finish(true);
    const cancel = () => finish(false);
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
    window.addEventListener('pointercancel', cancel);
  }

  function drop(d: Drag) {
    const [kind, target] = [d.over!.slice(0, 1), d.over!.slice(2)];
    if (kind === 'f') snippets.move(d.id, target, null);
    else if (kind === 'n') snippets.move(d.id, null, null);
    else if (target !== d.id) {
      const sec = secs.find((x) => x.items.some((i) => i.id === target));
      if (!sec) return;
      const i = sec.items.findIndex((x) => x.id === target);
      const before = d.after ? (sec.items[i + 1]?.id ?? null) : target;
      if (before !== d.id) snippets.move(d.id, sec.group?.id ?? null, before);
    }
  }

  /** A press that ended a drag is not also a click (run / open folder). */
  function swallow(e: MouseEvent) {
    if (!suppressClick) return;
    e.stopPropagation();
    e.preventDefault();
  }
  const hit = (key: string) => drag?.over === key;

  const countText = (count: number) => {
    const f = pluralForm(count, getLocale());
    return f === 'one'
      ? m.snip_count_one({ count })
      : f === 'few'
        ? m.snip_count_few({ count })
        : m.snip_count_many({ count });
  };
</script>

{#snippet tile(s: Snip)}
  <div
    class="tile snip"
    class:over={hit('s:' + s.id) && drag?.id !== s.id}
    class:after={drag?.after}
    class:lifted={drag?.id === s.id}
    role="listitem"
    data-drop={'s:' + s.id}
    onpointerdown={(e) => down(e, s)}
  >
    <button class="tmain" onclick={() => snippets.run(s)} title={m.snip_run_hint()}>
      <span class="ico code"><SquareTerminal size={17} /></span>
      <span class="ttext">
        <span class="tname"
          >{s.name}{#if parseVars(s.body).length}<span class="var">{'{…}'}</span>{/if}</span
        >
        <span class="tsub mono">{firstLine(s.body)}</span>
      </span>
    </button>
    <span class="tacts">
      <button
        class="mini"
        onclick={() => snippets.run(s, 'many')}
        aria-label={m.snip_run_many()}
        title={m.snip_run_many()}><Server size={13} /></button
      >
      <button
        class="mini"
        onclick={() => (snippets.editing = { snippet: s, group: null })}
        aria-label={m.snip_edit()}
        title={m.snip_edit()}><Pencil size={12} /></button
      >
    </span>
  </div>
{/snippet}

{#snippet folderTile(g: { id: string; name: string }, on = false)}
  <div
    class="tile folder"
    class:on
    class:over={hit('f:' + g.id)}
    role="group"
    data-drop={'f:' + g.id}
  >
    <button
      class="tmain"
      aria-expanded={on}
      onclick={() => (opened = opened === g.id ? null : g.id)}
      ondblclick={() => (snippets.folderDialog = { id: g.id })}
    >
      <span class="ico fold"><Folder size={17} /></span>
      <span class="ttext">
        <span class="tname">{g.name}</span>
        <span class="tsub tnum">{countText(itemsOf(g.id).length)}</span>
      </span>
    </button>
    <span class="tacts">
      <button
        class="mini"
        onclick={() => (snippets.folderDialog = { id: g.id })}
        aria-label={m.menu_rename()}
        title={m.menu_rename()}><Pencil size={12} /></button
      >
      <button
        class="mini"
        onclick={() => (snippets.folderDelete = g.id)}
        aria-label={m.snip_delete_folder()}
        title={m.snip_delete_folder()}><Trash2 size={12} /></button
      >
    </span>
  </div>
{/snippet}

<div class="board" class:dragging={drag} onclickcapture={swallow}>
  <div class="toolbar">
    <label class="search"
      ><Search size={14} /><input
        type="search"
        bind:value={q}
        placeholder={m.snip_search()}
        aria-label={m.snip_search()}
      /></label
    >
    <button class="btn btn-sm" onclick={() => (snippets.folderDialog = { id: null })}
      ><FolderPlus size={14} />{m.snip_new_folder()}</button
    >
    <button
      class="btn btn-primary btn-sm"
      onclick={() =>
        (snippets.editing = {
          snippet: null,
          group: openGroup?.id ?? null
        })}><Plus size={14} />{m.snip_new()}</button
    >
  </div>

  {#if !snippets.items.length && !groups.length && snippets.loaded}
    <EmptyState icon={Zap} title={m.snip_empty_title()} hint={m.snip_empty_hint()} />
  {:else if q}
    <h2>{m.wave_snip_found()} · {found.length}</h2>
    {#if found.length}
      <div class="grid">
        {#each found as s (s.id)}{@render tile(s)}{/each}
      </div>
    {:else}
      <p class="none">{m.dash_no_results()}</p>
    {/if}
  {:else}
    {#if groups.length}
      <h2>{m.wave_snip_folders()}</h2>
      <div class="grid">
        {#each groups as g (g.id)}{@render folderTile(g, opened === g.id)}{/each}
      </div>
      {#if openGroup}
        <!-- The folder unfolds under the tiles; a second click folds it. -->
        <div
          class="tray"
          class:over={hit('f:' + openGroup.id)}
          role="group"
          data-drop={'f:' + openGroup.id}
        >
          <div class="tray-head">
            <Folder size={14} /><b>{openGroup.name}</b>
            <span class="sp"></span>
            <button
              class="mini"
              onclick={() => (snippets.editing = { snippet: null, group: openGroup.id })}
              aria-label={m.snip_new()}
              title={m.snip_new()}><Plus size={13} /></button
            >
          </div>
          <div class="grid">
            {#each itemsOf(openGroup.id) as s (s.id)}{@render tile(s)}{:else}
              <p class="none">{m.snip_folder_empty()}</p>
            {/each}
          </div>
        </div>
      {/if}
    {/if}
    {#if loose.length || (drag && groups.length)}
      <section class="loose" class:over={hit('n:')} data-drop="n:" aria-label={m.wave_snip_loose()}>
        <h2>{m.wave_snip_loose()}</h2>
        <div class="grid">
          {#each loose as s (s.id)}{@render tile(s)}{:else}
            <p class="none">{m.wave_snip_nofolder()}</p>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>

{#if drag}
  <div class="ghost" style:left="{drag.x + 12}px" style:top="{drag.y + 8}px">{drag.label}</div>
{/if}

<style>
  .board {
    display: grid;
    gap: 12px;
    max-width: 1180px;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }
  .search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: 420px;
    height: var(--control-h);
    padding: 0 10px;
    border-radius: var(--r-control);
    background: var(--surface);
    box-shadow: var(--card-shadow);
    color: var(--muted);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: none;
    color: var(--text);
    font: inherit;
  }
  h2 {
    margin: 10px 0 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 10px;
  }
  .tile {
    position: relative;
    display: flex;
    align-items: center;
    min-width: 0;
    border-radius: 12px;
    background: var(--surface);
    box-shadow: var(--card-shadow);
    transition:
      box-shadow 120ms var(--ease),
      transform 120ms var(--ease);
  }
  .tile:hover {
    box-shadow:
      var(--card-shadow),
      0 0 0 1px var(--sep);
  }
  .tile:active {
    transform: scale(0.99);
  }
  .tile.on {
    box-shadow:
      var(--card-shadow),
      0 0 0 1.5px var(--accent);
  }
  .tmain {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    padding: 12px 14px;
    border: 0;
    border-radius: inherit;
    background: none;
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .ico {
    display: grid;
    place-items: center;
    flex: none;
    width: 36px;
    height: 36px;
    border-radius: 10px;
  }
  .ico.fold {
    background: color-mix(in oklab, var(--accent) 20%, transparent);
    color: var(--accent);
  }
  .ico.code {
    background: var(--fill-2);
    color: var(--muted);
  }
  .ttext {
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .tname {
    overflow: hidden;
    font-size: 13.5px;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .var {
    margin-left: 6px;
    padding: 0 5px;
    border-radius: 5px;
    background: var(--surface-2);
    color: var(--muted);
    font: 10px var(--font-mono);
  }
  .tsub {
    overflow: hidden;
    color: var(--muted);
    font-size: 11.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tacts {
    position: absolute;
    top: 50%;
    right: 8px;
    display: none;
    gap: 1px;
    padding: 2px;
    border-radius: 9px;
    background: var(--surface);
    box-shadow: var(--card-shadow);
    transform: translateY(-50%);
  }
  .tile:hover .tacts,
  .tile:focus-within .tacts {
    display: flex;
  }
  .mini {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 7px;
    background: none;
    color: var(--muted);
    cursor: pointer;
  }
  .mini:hover {
    background: var(--hover);
    color: var(--text);
  }
  .none {
    margin: 4px 0;
    color: var(--muted);
    font-size: 13px;
  }
  .sp {
    flex: 1;
  }

  /* Drag and drop */
  .board.dragging,
  .board.dragging :global(*) {
    cursor: grabbing !important;
    user-select: none;
  }
  .tile.lifted {
    opacity: 0.4;
  }
  .tile.folder.over,
  .tray.over,
  .loose.over {
    box-shadow:
      var(--card-shadow),
      0 0 0 2px var(--accent);
  }
  .loose {
    display: grid;
    gap: 12px;
    margin: 0 -8px;
    padding: 0 8px 8px;
    border-radius: 14px;
  }
  .tile.snip.over::before {
    content: '';
    position: absolute;
    top: 8px;
    bottom: 8px;
    left: -6px;
    width: 2px;
    border-radius: 2px;
    background: var(--accent);
  }
  .tile.snip.over.after::before {
    right: -6px;
    left: auto;
  }
  .ghost {
    position: fixed;
    z-index: 1000;
    max-width: 260px;
    padding: 6px 12px;
    overflow: hidden;
    border: 0.5px solid var(--border);
    border-radius: 9px;
    background: var(--surface);
    box-shadow: var(--shadow);
    color: var(--text);
    font-size: 12.5px;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
    pointer-events: none;
  }

  /* The folder unfolds under the tiles */
  .tray {
    display: grid;
    gap: 10px;
    padding: 12px;
    border-radius: 14px;
    background: color-mix(in oklab, var(--accent) 7%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in oklab, var(--accent) 25%, transparent);
    animation: tray 200ms var(--ease);
  }
  @keyframes tray {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
  }
  .tray-head {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--accent);
    font-size: 13px;
  }
  .tray-head b {
    color: var(--text);
  }
</style>
