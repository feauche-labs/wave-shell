<script lang="ts">
  import EmptyState from '$lib/components/EmptyState.svelte';
  import { slide } from 'svelte/transition';
  import {
    ChevronRight,
    Folder,
    FolderPlus,
    Pencil,
    Play,
    Plus,
    Search,
    Server,
    TextCursorInput,
    Trash2,
    Zap
  } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { frequent, parseVars, sections } from './model';
  import { snippets } from './snippets.svelte';
  import './snippets-panel.css';

  let q = $state('');
  // Wave: folders are rows of the list that open in place (the open ones are remembered); search opens all.
  const secs = $derived(sections(snippets.items, snippets.layout, q));
  const OPEN_KEY = 'wave.snip.open';
  let open = $state<string[]>(
    (() => {
      try {
        return JSON.parse(localStorage.getItem(OPEN_KEY) ?? '[]') as string[];
      } catch {
        return [];
      }
    })()
  );
  const isOpen = (id: string) => !!q || open.includes(id);
  function toggle(id: string) {
    open = open.includes(id) ? open.filter((x) => x !== id) : [...open, id];
    try {
      localStorage.setItem(OPEN_KEY, JSON.stringify(open));
    } catch {
      /* storage unavailable */
    }
  }
  const top = $derived(q ? [] : frequent(snippets.items, snippets.layout));
  const firstLine = (b: string) => b.split('\n').find((l) => l.trim()) ?? '';
  const countOf = (id: string) =>
    snippets.items.filter((x) => snippets.layout.of[x.id] === id).length;

  // ---- drag & drop on pointer events: the webview's native HTML5 drag never reaches the page
  // inside the app window (Tauri takes it for file drops), so it worked only in the browser. ----
  interface Drag {
    kind: 's' | 'f';
    id: string;
    label: string;
    x: number;
    y: number;
    /** `s:<id>` row or `f:<id>` folder under the pointer. */
    over: string | null;
    /** Pointer in the lower half of the row: drop after it. */
    after: boolean;
  }
  let drag = $state.raw<Drag | null>(null);
  let suppressClick = false;

  function down(e: PointerEvent, kind: 's' | 'f', id: string, label: string) {
    if (e.button !== 0 || (e.target as HTMLElement).closest('.acts, .facts')) return;
    const sx = e.clientX;
    const sy = e.clientY;
    const move = (ev: PointerEvent) => {
      if (!drag && Math.hypot(ev.clientX - sx, ev.clientY - sy) < 5) return;
      if (!drag) window.getSelection()?.removeAllRanges();
      const el = document.elementFromPoint(ev.clientX, ev.clientY);
      const t = el instanceof Element ? el.closest<HTMLElement>('[data-drop]') : null; // Wave: icons are SVG
      let over = t?.dataset.drop ?? null;
      if (kind === 'f' && !over?.startsWith('f:')) over = null;
      let after = false;
      if (t && over?.startsWith('s:')) {
        const r = t.getBoundingClientRect();
        after = ev.clientY > r.top + r.height / 2;
      }
      drag = { kind, id, label, x: ev.clientX, y: ev.clientY, over, after };
    };
    const finish = (commit: boolean) => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      window.removeEventListener('pointercancel', cancel);
      window.removeEventListener('keydown', esc, true);
      const d = drag;
      drag = null;
      if (!d) return;
      suppressClick = true;
      setTimeout(() => (suppressClick = false), 0);
      if (commit && d.over) drop(d);
    };
    const up = () => finish(true);
    const cancel = () => finish(false);
    const esc = (ev: KeyboardEvent) => {
      if (ev.key !== 'Escape') return;
      ev.stopPropagation();
      finish(false);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
    window.addEventListener('pointercancel', cancel);
    window.addEventListener('keydown', esc, true);
  }

  function drop(d: Drag) {
    const [kind, target] = [d.over!.slice(0, 1), d.over!.slice(2)];
    if (d.kind === 's' && kind === 's') {
      const sec = secs.find((x) => x.items.some((s) => s.id === target));
      if (!sec) return;
      const i = sec.items.findIndex((s) => s.id === target);
      const before = d.after ? (sec.items[i + 1]?.id ?? null) : target;
      if (before !== d.id) snippets.move(d.id, sec.group?.id ?? null, before);
    } else if (d.kind === 's') {
      snippets.move(d.id, target, null);
    } else if (d.id !== target) {
      const gs = snippets.layout.groups;
      const from = gs.findIndex((g) => g.id === d.id);
      const to = gs.findIndex((g) => g.id === target);
      for (let i = 0; i < Math.abs(to - from); i++) snippets.moveGroup(d.id, to > from ? 1 : -1);
    }
  }

  /** A press that ended a drag is not also a click (run / filter). */
  function swallow(e: MouseEvent) {
    if (!suppressClick) return;
    e.stopPropagation();
    e.preventDefault();
  }
</script>

{#snippet row(s: (typeof snippets.items)[number])}
  {@const hasVars = parseVars(s.body).length > 0}
  <div
    class="row"
    class:over={drag?.kind === 's' && drag.over === 's:' + s.id && drag.id !== s.id}
    class:after={drag?.after}
    class:dragging={drag?.id === s.id}
    role="listitem"
    data-drop={'s:' + s.id}
    onpointerdown={(e) => down(e, 's', s.id, s.name)}
  >
    <button class="main" onclick={() => snippets.run(s)} title={m.snip_run_hint()}>
      <span class="name"
        >{s.name}{#if hasVars}<span class="tag">{'{…}'}</span>{/if}</span
      >
      <span class="cmd mono">{firstLine(s.body)}</span>
    </button>
    <span class="acts">
      <button
        class="mini"
        onclick={() => snippets.run(s, 'here', false)}
        aria-label={m.snip_insert()}
        title={m.snip_insert()}><TextCursorInput size={13} /></button
      >
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

<div class="snips" class:dragging={drag} onclickcapture={swallow}>
  <div class="toolbar">
    <label class="search"
      ><Search size={14} /><input
        type="search"
        bind:value={q}
        placeholder={m.snip_search()}
        aria-label={m.snip_search()}
      /></label
    >
    <button
      class="icon-btn"
      onclick={() => (snippets.folderDialog = { id: null })}
      aria-label={m.snip_new_folder()}
      title={m.snip_new_folder()}><FolderPlus size={16} /></button
    >
    <button
      class="icon-btn add"
      onclick={() => (snippets.editing = { snippet: null, group: null })}
      aria-label={m.snip_new()}
      title={m.snip_new()}><Plus size={16} /></button
    >
  </div>

  {#if !snippets.items.length && snippets.loaded}
    <EmptyState icon={Zap} title={m.snip_empty_title()} hint={m.snip_empty_hint()}>
      <button
        class="btn btn-primary btn-sm"
        onclick={() => (snippets.editing = { snippet: null, group: null })}
        ><Plus size={14} />{m.snip_new()}</button
      >
    </EmptyState>
  {:else}
    {#if top.length}
      <div class="frequent" aria-label={m.snip_frequent()}>
        <span class="cap">{m.snip_frequent()}</span>
        <div class="pills">
          {#each top as s (s.id)}
            <button class="pill" onclick={() => snippets.run(s)} title={s.body}
              ><Play size={11} />{s.name}</button
            >
          {/each}
        </div>
      </div>
    {/if}

    <div class="list">
      {#each secs as sec (sec.group?.id ?? '_')}
        {#if sec.group}
          {@const g = sec.group}
          <div
            class="frow"
            class:over={drag?.over === 'f:' + g.id && drag.id !== g.id}
            class:lifted={drag?.id === g.id}
            role="group"
            data-drop={'f:' + g.id}
            onpointerdown={(e) => down(e, 'f', g.id, g.name)}
          >
            <button
              class="fmain"
              aria-expanded={isOpen(g.id)}
              onclick={() => toggle(g.id)}
              ondblclick={() => (snippets.folderDialog = { id: g.id })}
            >
              <span class="chev" class:open={isOpen(g.id)}><ChevronRight size={14} /></span>
              <Folder size={15} />
              <span class="fname">{g.name}</span>
              <span class="fmeta tnum">{countOf(g.id)}</span>
            </button>
            <span class="facts">
              <button
                class="mini"
                onclick={() => (snippets.editing = { snippet: null, group: g.id })}
                aria-label={m.snip_new()}
                title={m.snip_new()}><Plus size={12} /></button
              >
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
          {#if isOpen(g.id)}
            <div class="inside" transition:slide={{ duration: 180 }}>
              {#each sec.items as s (s.id)}
                {@render row(s)}
              {:else}
                <div
                  class="drop-hint"
                  class:over={drag?.kind === 's' && drag.over === 'f:' + g.id}
                  role="listitem"
                  data-drop={'f:' + g.id}
                >
                  {m.snip_folder_empty()}
                </div>
              {/each}
            </div>
          {/if}
        {:else}
          {#each sec.items as s (s.id)}
            {@render row(s)}
          {/each}
        {/if}
      {/each}
      {#if q && !secs.length}
        <p class="none">{m.dash_no_results()}</p>
      {/if}
    </div>
  {/if}
</div>

{#if drag}
  <div class="snips-ghost" style:left="{drag.x + 12}px" style:top="{drag.y + 8}px">
    {drag.label}
  </div>
{/if}
