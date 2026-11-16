<script lang="ts">
  import { canOpenHostWindow, detachGroup, openHostWindow } from './host-window';
  import { ContextMenu } from 'bits-ui';
  import { ChevronRight } from '@lucide/svelte';
  import Flag from '$lib/flags/Flag.svelte';
  import { flagName, type FlagLocale } from '$lib/flags/flags';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { COLORS, lookName } from '$lib/wave/look/catalog';
  import { folderColors } from '$lib/wave/look/colors.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { connectAnother, connectFolder, connectHost, hostsInFolder } from '$lib/stores/actions';
  import { sessions } from '$lib/stores/sessions.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { tree } from '$lib/stores/tree.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { folderSubtree, siblings, type NodeKey, type Row } from '$lib/util/tree-model';

  /** Context row (null = empty area) and the keys the action applies to. */
  let { row, keys }: { row: Row | null; keys: NodeKey[] } = $props();

  const multi = $derived(keys.length > 1);
  const only = $derived(keys.length === 1 ? row : null);

  const targets = $derived.by(() => {
    const banned = new Set<string>();
    for (const k of keys)
      if (k.startsWith('f:')) folderSubtree(tree.data, k.slice(2)).forEach((id) => banned.add(id));
    return tree.rows
      .filter((r) => r.kind === 'folder' && !banned.has(r.id))
      .map((r) => ({ id: r.id, label: r.label, depth: r.depth }));
  });

  function moveTo(parent: string | null) {
    const skip = new Set(keys);
    const sibs = siblings(tree.data, parent, skip);
    void tree.move(keys, {
      parent,
      after: sibs[sibs.length - 1] ?? null,
      rowIndex: -1,
      zone: 'inside'
    });
  }
  function flag(code: string | null) {
    if (code) settings.pushRecentFlag(code);
    for (const k of keys) void tree.setFlag(k, code);
  }
  // Wave: icons / emoji and folder colours.
  const locale = getLocale() as FlagLocale;
  function paint(color: string | null) {
    for (const k of keys) if (k.startsWith('f:')) folderColors.set(k.slice(2), color);
  }
  function edit() {
    if (!row) return;
    if (row.kind === 'host') ui.openHostEditor(row.id);
    else ui.openFolderEditor(row.id);
  }
</script>

{#snippet item(label: string, onSelect: () => void, danger = false)}
  <ContextMenu.Item class="menu-item {danger ? 'danger' : ''}" {onSelect}>{label}</ContextMenu.Item>
{/snippet}

{#if !row}
  {@render item(m.action_new_host(), () => ui.openHostEditor(null))}
  {@render item(m.action_new_folder(), () => ui.openFolderEditor(null))}
  {@render item(m.imp_action(), () => (ui.sshImportOpen = true))}
{:else}
  {#if only?.kind === 'host'}
    {@render item(m.menu_connect(), () => connectHost(only.id))}
    {#if canOpenHostWindow}
      {@render item(m.menu_open_host_window(), () => {
        const current =
          sessions.groups.find(
            (g) => g.current.hostId === only.id && g.key === sessions.activeGroup
          ) ?? sessions.groups.find((g) => g.current.hostId === only.id);
        if (current) void detachGroup(current.key);
        else void openHostWindow(only.id);
      })}
    {/if}
    {@render item(m.menu_connect_another(), () => connectAnother(only.id))}
    {@render item(m.menu_sftp(), () => connectHost(only.id, 'sftp'))}
    {@render item(m.menu_status(), () => connectHost(only.id, 'status'))}
    <div class="menu-sep"></div>
    {@render item(m.menu_duplicate(), () => void tree.duplicate(only.id))}
    {@render item(m.menu_edit(), edit)}
  {:else if only?.kind === 'folder'}
    {@const n = hostsInFolder(only.id).length}
    {#if n}
      {@render item(m.menu_connect_all({ count: n }), () => connectFolder(only.id))}
      <div class="menu-sep"></div>
    {/if}
    {@render item(m.menu_new_host_here(), () => ui.openHostEditor(only.id, only.id))}
    {@render item(m.menu_new_subfolder(), () => ui.openFolderEditor(null, only.id))}
    <div class="menu-sep"></div>
    {@render item(m.menu_edit_defaults(), edit)}
  {/if}
  {#if only}
    {@render item(m.menu_rename(), () => (ui.renameKey = only.key))}
  {/if}

  <ContextMenu.Sub>
    <ContextMenu.SubTrigger class="menu-item">
      <span class="grow">{m.wave_look_field()}</span><ChevronRight size={14} />
    </ContextMenu.SubTrigger>
    <!-- Wave: in a portal — inside the menu (overflow + backdrop-filter) the submenu was clipped away. -->
    <ContextMenu.Portal>
      <ContextMenu.SubContent class="menu" sideOffset={14}>
        {@render item(m.wave_look_none(), () => flag(null))}
        {#each settings.recentFlags as code (code)}
          <ContextMenu.Item class="menu-item" onSelect={() => flag(code)}>
            <Flag {code} size={14} /><span
              >{lookName(code.includes(':') ? code : 'flag:' + code, locale, flagName)}</span
            >
          </ContextMenu.Item>
        {/each}
        {#if only}{@render item(m.wave_look_more(), edit)}{/if}
      </ContextMenu.SubContent>
    </ContextMenu.Portal>
  </ContextMenu.Sub>

  {#if keys.every((k) => k.startsWith('f:'))}
    <!-- Wave: folder colour -->
    <ContextMenu.Sub>
      <ContextMenu.SubTrigger class="menu-item">
        <span class="grow">{m.wave_color_field()}</span><ChevronRight size={14} />
      </ContextMenu.SubTrigger>
      <ContextMenu.Portal>
        <ContextMenu.SubContent class="menu" sideOffset={14}>
          {@render item(m.wave_color_none(), () => paint(null))}
          {#each COLORS as c (c.id)}
            <ContextMenu.Item class="menu-item" onSelect={() => paint(c.id)}>
              <span class="wave-dot" style:background={c.hex}></span><span
                >{locale === 'ru' ? c.ru : c.en}</span
              >
            </ContextMenu.Item>
          {/each}
        </ContextMenu.SubContent>
      </ContextMenu.Portal>
    </ContextMenu.Sub>
  {/if}

  <ContextMenu.Sub>
    <ContextMenu.SubTrigger class="menu-item">
      <span class="grow">{m.menu_move_to()}</span><ChevronRight size={14} />
    </ContextMenu.SubTrigger>
    <!-- Wave: in a portal — inside the menu (overflow + backdrop-filter) the submenu was clipped away. -->
    <ContextMenu.Portal>
      <ContextMenu.SubContent class="menu" sideOffset={14}>
        {@render item(m.menu_top_level(), () => moveTo(null))}
        {#each targets as t (t.id)}
          <ContextMenu.Item
            class="menu-item"
            style="padding-left: {8 + t.depth * 12}px"
            onSelect={() => moveTo(t.id)}>{t.label}</ContextMenu.Item
          >
        {/each}
      </ContextMenu.SubContent>
    </ContextMenu.Portal>
  </ContextMenu.Sub>

  <div class="menu-sep"></div>
  {@render item(
    multi ? m.menu_delete_n({ count: keys.length }) : m.menu_delete(),
    () => (ui.deleteKeys = keys),
    true
  )}
{/if}

<style>
  :global(.grow) {
    flex: 1;
  }
  .wave-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    opacity: 0.85;
  }
</style>
