<script lang="ts">
  /** Wave: the update sheet — what is new, then download with progress, install and relaunch. */
  import Sheet from '$lib/components/Sheet.svelte';
  import { api } from '$lib/api';
  import * as m from '$lib/paraglide/messages.js';
  import { settings } from '$lib/stores/settings.svelte';
  import { parseNotes } from './notes';
  import { waveUpdate as up } from './update.svelte';

  const RELEASES = 'https://github.com/feauche-labs/wave-shell/releases/latest';
  const sections = $derived(parseNotes(up.notes));
  const releaseUrl = $derived(settings.update?.url ?? RELEASES);
  const mb = (bytes: number) => Math.max(1, Math.round(bytes / 1_048_576));
  const title = $derived(
    up.version && (up.phase === 'ready' || up.busy)
      ? m.wave_update_title_version({ version: up.version })
      : m.wave_update_title()
  );
</script>

<Sheet {title} onclose={() => up.close()}>
  {#if up.phase === 'checking'}
    <p class="muted">{m.wave_update_checking()}</p>
  {:else if up.phase === 'none'}
    <p>{m.wave_update_none()}</p>
    {#if up.current}<p class="muted">{m.wave_update_current({ version: up.current })}</p>{/if}
  {:else if up.phase === 'unsupported'}
    <p>{m.wave_update_unsupported()}</p>
  {:else if up.phase === 'error'}
    <p class="error">{m.wave_update_failed({ error: up.error })}</p>
  {:else}
    {#if up.current}<p class="muted">{m.wave_update_current({ version: up.current })}</p>{/if}
    {#each sections as s, i (i)}
      <section>
        {#if s.title}<h4>{s.title}</h4>{/if}
        <ul>
          {#each s.items as item, j (j)}<li>{item}</li>{/each}
        </ul>
      </section>
    {:else}
      <p class="muted">{m.wave_update_no_notes()}</p>
    {/each}
  {/if}

  {#snippet footer()}
    {#if up.phase === 'downloading'}
      <div class="progress" role="status">
        <div
          class="bar"
          role="progressbar"
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow={up.percent}
        >
          <i style:width="{up.percent}%"></i>
        </div>
        <span class="muted">
          {m.wave_update_downloading({ version: up.version })}
          {#if up.total > 0}
            · {m.wave_update_progress({
              percent: String(up.percent),
              done: String(mb(up.downloaded)),
              total: String(mb(up.total))
            })}
          {/if}. {m.wave_update_restart_note()}
        </span>
      </div>
    {:else if up.phase === 'installing'}
      <div class="progress" role="status">
        <span class="muted">{m.wave_update_installing()}</span>
      </div>
    {:else if up.phase === 'ready'}
      <button type="button" class="btn" onclick={() => up.close()}>{m.wave_update_later()}</button>
      <button type="button" class="btn btn-primary" onclick={() => void up.install()}
        >{m.wave_update_install()}</button
      >
    {:else if up.phase !== 'checking'}
      <button type="button" class="btn" onclick={() => void api.openExternal(releaseUrl)}
        >{m.wave_update_release_page()}</button
      >
      {#if up.phase === 'error'}
        <button type="button" class="btn btn-primary" onclick={() => void up.show()}
          >{m.wave_update_retry()}</button
        >
      {/if}
    {/if}
  {/snippet}
</Sheet>

<style>
  p {
    margin: 0;
    font-size: 13px;
    line-height: 1.5;
  }
  .muted {
    color: var(--muted);
  }
  .error {
    color: var(--danger, #d0245e);
  }
  h4 {
    margin: 0 0 4px;
    font-size: 13px;
    font-weight: 600;
  }
  ul {
    margin: 0;
    padding-left: 18px;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.5;
  }
  li {
    list-style: disc;
  }
  .progress {
    flex: 1;
    display: grid;
    gap: 6px;
    font-size: 12px;
  }
  .bar {
    height: 4px;
    border-radius: 2px;
    background: var(--fill-2);
    overflow: hidden;
  }
  .bar i {
    display: block;
    height: 100%;
    border-radius: 2px;
    background: var(--accent);
    transition: width 200ms var(--ease);
  }
</style>
