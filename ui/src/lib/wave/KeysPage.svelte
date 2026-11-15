<script lang="ts">
  /** Wave: the Keys section — key tiles; a click opens the key on the right (KeySheet). */
  import { KeyRound, Plus, Sparkles } from '@lucide/svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import '$lib/fleetpages/page.css';
  import * as m from '$lib/paraglide/messages.js';
  import { scrollEdge } from '$lib/util/scroll-edge.svelte';
  import KeySheet from './KeySheet.svelte';
  import { waveKeys as keys } from './keys.svelte';

  void keys.load();
  const selected = $derived(keys.panel?.kind === 'edit' ? keys.panel.id : null);
</script>

<section class="fpage" aria-label={m.wave_keys_title()} use:scrollEdge>
  <header>
    <h1>{m.wave_keys_title()}</h1>
    <p class="subtitle">{m.wave_keys_hint()}</p>
  </header>

  <div class="bar">
    <button type="button" class="btn btn-primary" onclick={() => (keys.panel = { kind: 'new' })}
      ><Plus size={15} />{m.wave_keys_new()}</button
    >
    <button type="button" class="btn" onclick={() => (keys.panel = { kind: 'generate' })}
      ><Sparkles size={14} />{m.wave_keys_generate()}</button
    >
  </div>

  {#if keys.list.length}
    <div class="grid">
      {#each keys.list as k (k.id)}
        <button
          type="button"
          class="tile"
          class:sel={selected === k.id}
          onclick={() => (keys.panel = { kind: 'edit', id: k.id })}
        >
          <span class="ico"><KeyRound size={17} /></span>
          <span class="txt">
            <b>{k.label}</b>
            <small>{m.wave_keys_type({ type: keys.type(k) })}</small>
          </span>
        </button>
      {/each}
    </div>
  {:else}
    <EmptyState icon={KeyRound} title={m.wave_keys_empty_title()} hint={m.wave_keys_empty_hint()} />
  {/if}
</section>

{#if keys.panel}
  {#key keys.panel.kind === 'edit' ? keys.panel.id : keys.panel.kind}
    <KeySheet panel={keys.panel} />
  {/key}
{/if}

<style>
  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 16px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 10px;
  }
  .tile {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    padding: 12px;
    border: 0.5px solid var(--border);
    border-radius: var(--r-inner);
    background: var(--surface);
    box-shadow: var(--lift);
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: border-color 120ms var(--ease);
  }
  .tile:hover {
    background: var(--hover);
  }
  .tile.sel {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .ico {
    flex: none;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: var(--accent);
    color: #fff;
  }
  .txt {
    display: grid;
    gap: 1px;
    min-width: 0;
  }
  .txt b {
    overflow: hidden;
    font-size: 13px;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .txt small {
    color: var(--muted);
    font-size: 11px;
  }
</style>
