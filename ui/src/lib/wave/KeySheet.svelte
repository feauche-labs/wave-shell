<script lang="ts">
  /**
   * Wave: one key on the right (Termius «Edit key» / «New key»). New: paste the private key or pick a file; an
   * encrypted key shows the passphrase field on top. Edit: name, public key, install on a connected server,
   * hosts that use it, delete. The private key never comes back to the UI (vault), as in the author's design.
   */
  import { Copy, FileKey, Trash2, Upload } from '@lucide/svelte';
  import { api, type IdentityDto } from '$lib/api';
  import Sheet from '$lib/components/Sheet.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { sessions } from '$lib/stores/sessions.svelte';
  import { tree } from '$lib/stores/tree.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { errorText } from '$lib/util/errors';
  import { passphraseProblem, waveKeys as keys, type KeyPanel } from './keys.svelte';

  let { panel }: { panel: NonNullable<KeyPanel> } = $props();

  /* svelte-ignore state_referenced_locally */
  const key: IdentityDto | undefined = panel.kind === 'edit' ? keys.byId(panel.id) : undefined;

  let label = $state(key?.label ?? '');
  let pem = $state('');
  let passphrase = $state('');
  let needPass = $state(false);
  let algo = $state<'ed25519' | 'rsa4096'>('ed25519');
  let error = $state<string | null>(null);
  let busy = $state(false);
  let confirmDelete = $state(false);

  const title = $derived(
    panel.kind === 'edit'
      ? m.wave_keys_edit_title()
      : panel.kind === 'generate'
        ? m.wave_keys_generate()
        : m.wave_keys_new_title()
  );
  const usedBy = $derived(key ? tree.data.hosts.filter((h) => h.identity_id === key.id) : []);
  /** Servers with an open terminal: the public key can be installed there. */
  const connected = $derived(
    sessions.groups
      .map((g) => ({ label: g.label, sid: sessions.groupSession(g) }))
      .filter((s): s is { label: string; sid: string } => !!s.sid)
  );

  function close() {
    keys.panel = null;
  }

  /** Vault errors about the passphrase show the field on top instead of a raw message. */
  function fail(detail: string | undefined, fallback: string) {
    const p = passphraseProblem(detail);
    if (p) {
      needPass = true;
      error = p === 'wrong' ? m.wave_keys_passphrase_wrong() : m.wave_keys_passphrase_need();
    } else if (detail && /invalid key/.test(detail)) {
      error = m.wave_keys_invalid();
    } else {
      error = fallback;
    }
  }

  async function saveNew() {
    if (!pem.trim()) return void (error = m.wave_keys_invalid());
    busy = true;
    error = null;
    const r = await api.identityCreate({
      label: label.trim() || 'SSH key',
      kind: passphrase ? 'key_password' : 'key',
      private_key_pem: pem.trim() + '\n',
      passphrase: passphrase || null
    });
    busy = false;
    if (r.status === 'error')
      return fail(r.error.code === 'Validation' ? r.error.detail : undefined, errorText(r.error));
    pem = '';
    await keys.load();
    keys.panel = { kind: 'edit', id: r.data.id };
    ui.toast(m.wave_keys_added());
  }

  async function fromFile() {
    busy = true;
    error = null;
    const r = await api.identityImportFile(passphrase || null);
    busy = false;
    if (r.status === 'error')
      return fail(r.error.code === 'Validation' ? r.error.detail : undefined, errorText(r.error));
    if (!r.data) return; // dialog cancelled
    let made = r.data;
    if (label.trim() && label.trim() !== made.label) {
      const u = await api.identityUpdate(made.id, { label: label.trim() });
      if (u.status === 'ok') made = u.data;
    }
    await keys.load();
    keys.panel = { kind: 'edit', id: made.id };
    ui.toast(m.wave_keys_added());
  }

  async function generate() {
    busy = true;
    error = null;
    const r = await api.identityGenerate(label.trim() || 'Wave Shell', algo);
    busy = false;
    if (r.status === 'error') return void (error = errorText(r.error));
    await keys.load();
    keys.panel = { kind: 'edit', id: r.data.id };
    ui.toast(m.keygen_done());
  }

  async function rename() {
    if (!key || !label.trim() || label.trim() === key.label) return close();
    busy = true;
    const r = await api.identityUpdate(key.id, { label: label.trim() });
    busy = false;
    if (r.status === 'error') return void (error = errorText(r.error));
    await keys.load();
    ui.toast(m.toast_saved());
    close();
  }

  async function remove() {
    if (!key) return;
    if (!confirmDelete) return void (confirmDelete = true);
    const r = await api.identityDelete(key.id);
    if (r.status === 'error') return void (error = errorText(r.error));
    await keys.load();
    ui.toast(m.toast_deleted());
    close();
  }

  async function install(host: string, sid: string) {
    if (!key) return;
    busy = true;
    const r = await api.identityInstall(sid, key.id);
    busy = false;
    if (r.status === 'error') return ui.toast(errorText(r.error), 'error');
    ui.toast(r.data ? m.wave_keys_installed({ host }) : m.wave_keys_already({ host }));
  }

  function copy(text: string) {
    void navigator.clipboard.writeText(text).then(() => ui.toast(m.toast_copied()));
  }
</script>

<Sheet {title} onclose={close}>
  {#if needPass}
    <div class="fld pass">
      <label for="k-pass">{m.wave_keys_passphrase()}</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        id="k-pass"
        class="input"
        type="password"
        bind:value={passphrase}
        autocomplete="off"
        autofocus
      />
    </div>
  {/if}

  <div class="fld">
    <label for="k-label">{m.wave_keys_label()}</label>
    <input
      id="k-label"
      class="input"
      bind:value={label}
      placeholder={m.wave_keys_label_ph()}
      autocomplete="off"
    />
  </div>

  {#if panel.kind === 'new'}
    <div class="fld">
      <label for="k-pem">{m.wave_keys_private()}</label>
      <textarea
        id="k-pem"
        class="input mono"
        rows="8"
        spellcheck="false"
        bind:value={pem}
        placeholder={m.wave_keys_private_ph()}></textarea>
    </div>
    <div class="or"><span>{m.wave_keys_or_file()}</span></div>
    <button type="button" class="btn file" disabled={busy} onclick={fromFile}
      ><Upload size={14} />{m.wave_keys_import_file()}</button
    >
  {:else if panel.kind === 'generate'}
    <div class="fld">
      <span class="lbl">{m.wave_keys_algo()}</span>
      <div class="seg" role="radiogroup" aria-label={m.wave_keys_algo()}>
        <button
          type="button"
          role="radio"
          aria-checked={algo === 'ed25519'}
          onclick={() => (algo = 'ed25519')}>Ed25519</button
        >
        <button
          type="button"
          role="radio"
          aria-checked={algo === 'rsa4096'}
          onclick={() => (algo = 'rsa4096')}>RSA 4096</button
        >
      </div>
      <p class="note">
        {m.wave_keys_generate_note()}
        {algo === 'rsa4096' ? m.keygen_rsa_note() : ''}
      </p>
    </div>
  {:else if key}
    <p class="note meta mono">
      {key.key_algo ?? ''}{key.key_fingerprint ? ` · ${key.key_fingerprint}` : ''}
    </p>
    <p class="note">{m.wave_keys_private_hidden()}</p>
    {#if key.public_key}
      <div class="fld">
        <div class="lblrow">
          <span class="lbl">{m.wave_keys_public()}</span>
          <button type="button" class="btn btn-sm" onclick={() => copy(key.public_key ?? '')}
            ><Copy size={13} />{m.wave_keys_copy_public()}</button
          >
        </div>
        <textarea class="input mono pub" rows="4" readonly value={key.public_key}></textarea>
      </div>
    {/if}
    <div class="fld">
      <span class="lbl">{m.wave_keys_install()}</span>
      {#if connected.length}
        <div class="row">
          {#each connected as s (s.sid)}
            <button
              type="button"
              class="btn btn-sm"
              disabled={busy}
              onclick={() => install(s.label, s.sid)}
              ><FileKey size={13} />{m.wave_keys_install_to({ host: s.label })}</button
            >
          {/each}
        </div>
        <p class="note">{m.wave_keys_install_hint()}</p>
      {:else}
        <p class="note">{m.wave_keys_install_none()}</p>
      {/if}
    </div>
    <div class="fld">
      <span class="lbl">{m.wave_keys_used_by()}</span>
      {#if usedBy.length}
        <div class="row">
          {#each usedBy as h (h.id)}<span class="pill">{h.label || h.hostname}</span>{/each}
        </div>
      {:else}
        <p class="note">{m.wave_keys_unused()}</p>
      {/if}
    </div>
  {/if}

  {#if error}<p class="err" role="alert">{error}</p>{/if}

  {#snippet footer()}
    {#if key}
      <button type="button" class="btn btn-delete" onclick={remove}
        ><Trash2 size={14} />{confirmDelete
          ? m.wave_keys_delete_confirm()
          : m.wave_keys_delete()}</button
      >
    {/if}
    <button type="button" class="btn" onclick={close}>{m.action_cancel()}</button>
    {#if panel.kind === 'new'}
      <button type="button" class="btn btn-primary" disabled={busy || !pem.trim()} onclick={saveNew}
        >{m.wave_keys_save()}</button
      >
    {:else if panel.kind === 'generate'}
      <button type="button" class="btn btn-primary" disabled={busy} onclick={generate}
        >{m.action_create()}</button
      >
    {:else}
      <button type="button" class="btn btn-primary" disabled={busy} onclick={rename}
        >{m.wave_keys_save()}</button
      >
    {/if}
  {/snippet}
</Sheet>

<style>
  .fld {
    display: grid;
    gap: 6px;
  }
  .fld label,
  .lbl {
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .pass {
    padding: 10px 12px;
    border-radius: var(--r-control);
    background: var(--fill-2);
  }
  textarea {
    height: auto;
    padding: 8px 10px;
    resize: vertical;
    font-size: 12px;
    line-height: 1.45;
  }
  .pub {
    word-break: break-all;
  }
  .lblrow {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .note {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.45;
  }
  .meta {
    overflow-wrap: anywhere;
  }
  .or {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
    font-size: 12px;
  }
  .or::before,
  .or::after {
    content: '';
    flex: 1;
    height: 0.5px;
    background: var(--sep);
  }
  .file {
    justify-self: start;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .pill {
    padding: 3px 9px;
    border-radius: 99px;
    background: var(--fill-2);
    font-size: 12px;
  }
  .seg {
    display: inline-flex;
    justify-self: start;
    gap: 2px;
    padding: 2px;
    border-radius: var(--r-control);
    background: var(--fill-2);
  }
  .seg button {
    padding: 4px 12px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .seg button[aria-checked='true'] {
    background: var(--surface);
    box-shadow: var(--lift);
  }
  .err {
    margin: 0;
    color: var(--danger);
    font-size: 12px;
  }
</style>
