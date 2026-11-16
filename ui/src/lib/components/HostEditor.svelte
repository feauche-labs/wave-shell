<script lang="ts">
  import { Eye, EyeOff, FileKey, KeyRound, LockKeyhole, Trash2 } from '@lucide/svelte';
  import { api, type IdentityDto } from '$lib/api';
  import * as m from '$lib/paraglide/messages.js';
  import { tree } from '$lib/stores/tree.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { emptyDefaults, formFrom, toPatch, toValues } from '$lib/util/defaults-form';
  import { errorText } from '$lib/util/errors';
  import { hkey, inheritedFlag } from '$lib/util/tree-model';
  import DefaultsFields from './DefaultsFields.svelte';
  import LookPicker from '$lib/wave/look/LookPicker.svelte'; // Wave: flag / icon / emoji
  import Sheet from './Sheet.svelte';
  // Wave: tags of the host and a key picked from the Keys section.
  import TagInput from '$lib/wave/TagInput.svelte';
  import { waveTags } from '$lib/wave/tags.svelte';
  import { sessions } from '$lib/stores/sessions.svelte';

  let { id, folderId }: { id: string | null; folderId: string | null } = $props();

  // The editor is re-created per target ({#key} in Shell), so initial props are intentional.
  /* svelte-ignore state_referenced_locally */
  const host = id ? tree.host(id) : undefined;
  /* svelte-ignore state_referenced_locally */
  const folder = host ? host.folder_id : folderId;

  let label = $state(host?.label ?? '');
  let hostname = $state(host?.hostname ?? '');
  let flag = $state<string | null>(host?.country_code ?? null);
  let notes = $state(host?.notes ?? '');
  let defaults = $state(host ? formFrom(host) : emptyDefaults());
  let error = $state<string | null>(null);
  let busy = $state(false);

  // Sign-in: classic login + password, or a key file. Secrets go straight to the vault.
  let auth = $state<'password' | 'key'>('password');
  let password = $state('');
  let showPw = $state(false);
  /** Saved password loaded on the first eye click; unchanged → not written to the vault again. */
  let savedPw = $state<string | null>(null);
  let passphrase = $state('');
  let identity = $state.raw<IdentityDto | null>(null);
  let keyIdentity = $state.raw<IdentityDto | null>(null);
  // Wave: keys of the Keys section to pick from; tags as names until save.
  let keys = $state.raw<IdentityDto[]>([]);
  let needPass = $state(false);
  let tagNames = $state<string[]>([]);
  void waveTags.load().then(() => {
    if (host) tagNames = waveTags.names(host.tag_ids);
  });

  void api.identityList().then((r) => {
    if (r.status === 'ok') keys = r.data.filter((i) => i.kind !== 'password');
    if (r.status !== 'ok' || !host?.identity_id) return;
    identity = r.data.find((i) => i.id === host.identity_id) ?? null;
    if (identity && identity.kind !== 'password') {
      auth = 'key';
      keyIdentity = identity;
    }
  });

  const inheritedF = $derived(inheritedFlag(tree.data, folder));
  const passwordSaved = $derived(auth === 'password' && !!identity?.has_password);

  function validate(): string | null {
    if (!hostname.trim()) return m.err_hostname_required();
    const p = defaults.port.trim();
    if (p && !(/^\d+$/.test(p) && +p >= 1 && +p <= 65535)) return m.err_port_range();
    return null;
  }

  async function togglePw() {
    if (!showPw && passwordSaved && !password && identity) {
      const r = await api.identityRevealPassword(identity.id);
      if (r.status === 'error') return ui.toast(errorText(r.error), 'error');
      savedPw = r.data;
      password = r.data ?? '';
    }
    showPw = !showPw;
  }

  async function chooseKey() {
    const r = await api.identityImportFile(passphrase || null);
    if (r.status === 'error') {
      // Wave: an encrypted key — ask for its passphrase right here, then the user picks the file again.
      if (r.error.code === 'Validation' && /passphrase/.test(r.error.detail ?? '')) {
        needPass = true;
        return ui.toast(
          /wrong/.test(r.error.detail ?? '')
            ? m.wave_keys_passphrase_wrong()
            : m.wave_keys_passphrase_need(),
          'error'
        );
      }
      return ui.toast(errorText(r.error), 'error');
    }
    passphrase = '';
    needPass = false;
    if (r.data) {
      keyIdentity = r.data;
      keys = [...keys.filter((k) => k.id !== r.data?.id), r.data];
    }
  }

  /** Wave: go to the Keys section (the editor closes; unsaved edits stay unsaved). */
  function openKeys() {
    ui.closeEditor();
    ui.page = 'keys';
    sessions.activeId = null;
  }

  /** Resolves the identity id to store on the host, writing a new password into the vault. */
  async function signIn(name: string): Promise<string | null | false> {
    const login = defaults.username.trim() || null;
    if (auth === 'key') return keyIdentity?.id ?? null;
    if (!password || password === savedPw)
      return identity?.kind === 'password' ? identity.id : null;
    const r =
      identity?.kind === 'password'
        ? await api.identityUpdate(identity.id, {
            password,
            ...(login ? { username: login } : {})
          })
        : await api.identityCreate({ label: name, username: login, kind: 'password', password });
    password = '';
    if (r.status === 'error') {
      error = errorText(r.error);
      return false;
    }
    return r.data.id;
  }

  async function save() {
    error = validate();
    if (error) return;
    busy = true;
    const name = label.trim() || hostname.trim();
    const idn = await signIn(name);
    if (idn === false) return (busy = false);
    defaults.identity = idn ?? '';
    // Wave: tag names → ids (new tags are created here).
    const tagIds = await waveTags.ensure(tagNames);
    if (tagIds === null) {
      error = m.err_generic();
      return (busy = false);
    }
    const tagsChanged = !host || [...tagIds].sort().join() !== [...host.tag_ids].sort().join();
    let ok: boolean;
    if (host) {
      ok = await tree.updateHost(host.id, {
        ...(name !== host.label ? { label: name } : {}),
        ...(hostname.trim() !== host.hostname ? { hostname: hostname.trim() } : {}),
        ...(flag !== host.country_code ? { country_code: flag } : {}),
        ...(notes !== (host.notes ?? '') ? { notes: notes || null } : {}),
        ...(tagsChanged ? { tag_ids: tagIds } : {}),
        ...toPatch(formFrom(host), defaults)
      });
    } else {
      ok =
        (await tree.createHost({
          folder_id: folder,
          label: name,
          hostname: hostname.trim(),
          country_code: flag,
          notes: notes || null,
          tag_ids: tagIds,
          ...toValues(defaults)
        })) !== null;
    }
    busy = false;
    if (ok) {
      ui.closeEditor();
      ui.toast(m.toast_saved());
    }
  }
</script>

<Sheet title={host ? m.editor_host_edit() : m.editor_host_new()} onclose={() => ui.closeEditor()}>
  <form id="host-form" class="form" onsubmit={(e) => (e.preventDefault(), void save())}>
    <section>
      <div class="pair">
        <div class="fld">
          <label for="h-host">{m.field_address()}</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input
            id="h-host"
            class="input mono"
            bind:value={hostname}
            autocomplete="off"
            spellcheck="false"
            placeholder="203.0.113.7"
            autofocus={!host}
          />
        </div>
        <div class="fld port">
          <label for="h-port">{m.field_port()}</label>
          <input
            id="h-port"
            class="input mono"
            inputmode="numeric"
            bind:value={defaults.port}
            placeholder="22"
            autocomplete="off"
          />
        </div>
      </div>
      <div class="fld">
        <label for="h-label">{m.field_label()}</label>
        <input
          id="h-label"
          class="input"
          bind:value={label}
          autocomplete="off"
          placeholder={hostname.trim() || m.host_name_hint()}
        />
      </div>
      <div class="fld">
        <label for="h-tags">{m.wave_tags()}</label>
        <TagInput id="h-tags" bind:names={tagNames} />
      </div>
      <div class="fld">
        <span class="lbl">{m.wave_look_field()}</span>
        <LookPicker value={flag} inherited={inheritedF} onchange={(c) => (flag = c)} />
      </div>
    </section>

    <section>
      <div class="head">
        <h3>{m.section_signin()}</h3>
        <div class="seg" role="radiogroup" aria-label={m.section_signin()}>
          <button
            type="button"
            role="radio"
            aria-checked={auth === 'password'}
            onclick={() => (auth = 'password')}><LockKeyhole size={13} />{m.auth_password()}</button
          >
          <button
            type="button"
            role="radio"
            aria-checked={auth === 'key'}
            onclick={() => (auth = 'key')}><KeyRound size={13} />{m.auth_key()}</button
          >
        </div>
      </div>
      <div class="fld">
        <label for="h-login">{m.field_login()}</label>
        <input
          id="h-login"
          class="input mono"
          bind:value={defaults.username}
          autocomplete="off"
          spellcheck="false"
          placeholder="root"
        />
      </div>
      {#if auth === 'password'}
        <div class="fld">
          <label for="h-pw">{m.field_password()}</label>
          <div class="pw">
            <input
              id="h-pw"
              class="input"
              type={showPw ? 'text' : 'password'}
              bind:value={password}
              autocomplete="new-password"
              placeholder={passwordSaved ? '••••••••' : ''}
            />
            <button
              type="button"
              class="icon-btn"
              aria-label={m.field_password()}
              aria-pressed={showPw}
              onclick={togglePw}
              >{#if showPw}<EyeOff size={15} />{:else}<Eye size={15} />{/if}</button
            >
          </div>
          {#if passwordSaved && savedPw === null}<p class="note">{m.password_saved()}</p>{/if}
        </div>
      {:else}
        <!-- Wave: the key is picked from the Keys section (added there or from a file right here). -->
        <div class="fld">
          <label for="h-key">{m.wave_host_key()}</label>
          {#if keys.length}
            <select
              id="h-key"
              class="input"
              value={keyIdentity?.id ?? ''}
              onchange={(e) =>
                (keyIdentity = keys.find((k) => k.id === e.currentTarget.value) ?? null)}
            >
              <option value="" disabled>{m.wave_host_key_pick()}</option>
              {#each keys as k (k.id)}<option value={k.id}
                  >{k.label}{k.key_algo ? ` · ${k.key_algo}` : ''}</option
                >{/each}
            </select>
            {#if keyIdentity?.key_fingerprint}<p class="note mono fp">
                {keyIdentity.key_fingerprint}
              </p>{/if}
          {:else}
            <p class="note">{m.wave_host_key_none()}</p>
          {/if}
        </div>
        {#if needPass}
          <div class="fld">
            <label for="h-pp">{m.key_passphrase()}</label>
            <input
              id="h-pp"
              class="input"
              type="password"
              bind:value={passphrase}
              autocomplete="off"
            />
          </div>
        {/if}
        <div class="key-acts">
          <button type="button" class="btn btn-sm" onclick={chooseKey}
            ><FileKey size={14} />{m.wave_host_key_file()}</button
          >
          <button type="button" class="btn btn-sm" onclick={openKeys}
            ><KeyRound size={14} />{m.wave_host_key_open()}</button
          >
        </div>
      {/if}
    </section>

    <section>
      <div class="fld">
        <label for="h-notes">{m.section_notes()}</label>
        <textarea
          id="h-notes"
          class="input"
          rows="3"
          placeholder={m.notes_placeholder()}
          bind:value={notes}></textarea>
      </div>
    </section>

    <details>
      <summary>{m.section_advanced()}</summary>
      <div class="adv">
        <DefaultsFields
          bind:form={defaults}
          inheritFrom={folder}
          selfHostId={host?.id ?? null}
          advancedOnly
        />
      </div>
    </details>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </form>
  {#snippet footer()}
    {#if host}
      <button
        type="button"
        class="btn btn-ghost btn-danger btn-delete"
        disabled={busy}
        onclick={() => ui.deleteFromEditor(hkey(host.id))}
        ><Trash2 size={14} />{m.action_delete()}</button
      >
    {/if}
    <button type="button" class="btn btn-ghost" onclick={() => ui.closeEditor()}
      >{m.action_cancel()}</button
    >
    <button type="submit" form="host-form" class="btn btn-primary" disabled={busy}
      >{m.action_save()}</button
    >
  {/snippet}
</Sheet>

<style>
  .pair {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 88px;
    gap: 10px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .pw {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  /* Wave: actions under the key picker. */
  .key-acts {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .fp {
    color: var(--muted);
    font-size: 11px;
    overflow-wrap: anywhere;
  }
  .btn {
    justify-self: start;
  }
  details summary {
    cursor: pointer;
    color: var(--muted);
    font-size: 12px;
    font-weight: 600;
    list-style: none;
  }
  details summary::-webkit-details-marker {
    display: none;
  }
  details summary::after {
    content: '';
    display: inline-block;
    width: 6px;
    height: 6px;
    margin-left: 8px;
    border-right: 1.5px solid currentColor;
    border-bottom: 1.5px solid currentColor;
    transform: translateY(-2px) rotate(45deg);
    transition: transform 150ms var(--ease);
  }
  details[open] summary::after {
    transform: translateY(1px) rotate(-135deg);
  }
  .adv {
    display: grid;
    gap: 12px;
    padding-top: 12px;
  }
</style>
