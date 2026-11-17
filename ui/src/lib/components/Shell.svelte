<script lang="ts">
  import { scaleKey, uiScale } from '$lib/wave/ui-scale.svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount, untrack, type Snippet } from 'svelte';
  import { api, type AppMenuAction } from '$lib/api';
  import { onMenuAction, PROJECT_PAGE } from '$lib/wave/app-menu';
  import { closeServer, connectAnother } from '$lib/stores/actions';
  import * as m from '$lib/paraglide/messages.js';
  import { reachability } from '$lib/fleet/reachability.svelte';
  import { sessions } from '$lib/stores/sessions.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { vault } from '$lib/stores/vault.svelte';
  import { isApple, isMac, shortcutKey } from '$lib/util/platform';
  import { edge } from '$lib/util/scroll-edge.svelte';
  import CloseServerDialog from './CloseServerDialog.svelte';
  import { isHostWindow } from './host-window';
  import HotkeysDialog from '$lib/help/HotkeysDialog.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import ConfirmDelete from './ConfirmDelete.svelte';
  import HostKeyDialog from '$lib/terminal/HostKeyDialog.svelte';
  import FolderEditor from './FolderEditor.svelte';
  import HostEditor from './HostEditor.svelte';
  import SettingsDialog from './SettingsDialog.svelte';
  import SshImportDialog from './SshImportDialog.svelte';
  // Wave: update one click away (sheet with what is new, download, install, relaunch).
  import UpdateSheet from '$lib/wave/UpdateSheet.svelte';
  import { waveUpdate } from '$lib/wave/update.svelte';
  import { waveKeys } from '$lib/wave/keys.svelte';
  import { waveTags } from '$lib/wave/tags.svelte';
  import { wavePeek } from '$lib/wave/sidebar-peek.svelte';
  import { folderColors } from '$lib/wave/look/colors.svelte';
  import Sidebar from './Sidebar.svelte';
  import TabBar from './TabBar.svelte';
  import Toasts from './Toasts.svelte';

  let { children }: { children: Snippet } = $props();

  $effect(() => () => reachability.clear());
  // Wave: look for updates through the updater itself (latest.json of our releases), not only the GitHub API.
  $effect(() => untrack(() => waveUpdate.watch(() => settings.updateCheck)));
  // Wave: key count in the sidebar from the start.
  $effect(() => untrack(() => void waveKeys.load()));
  // Wave: tag names for search and chips from the start.
  $effect(() => untrack(() => void waveTags.load()));
  // Wave: folder colours (line along the hosts of a folder in the list).
  $effect(() => untrack(() => void folderColors.load()));

  const native = isTauri();
  const mac = native && isMac;
  const windowControls = native && !isApple && !/Android/.test(navigator.userAgent);
  let narrow = $state(false);

  // Wave: the system window buttons follow the left menu (like in Arc): shown with it, hidden with it — after
  // it has slid away. Back on when the shell goes (lock screen).
  $effect(() => {
    if (!mac || isHostWindow) return;
    const show = ui.sidebarOpen || wavePeek.open;
    const t = setTimeout(
      () => void api.waveWindowButtons(show).catch(() => undefined),
      show ? 0 : 180
    );
    return () => clearTimeout(t);
  });
  onMount(() => () => {
    if (mac && !isHostWindow) void api.waveWindowButtons(true).catch(() => undefined);
  });

  // Wave: the app items of the macOS menu bar (texts: +layout) do what the same buttons and keys do.
  $effect(() => {
    if (!mac) return;
    const off = onMenuAction(getCurrentWindow().label, runMenu);
    return () => void off.then((stop) => stop());
  });
  function runMenu(action: AppMenuAction) {
    const main = !isHostWindow;
    if (action === 'check_update') void waveUpdate.show();
    else if (action === 'settings') ui.settingsOpen = true;
    else if (action === 'hotkeys') ui.hotkeysOpen = true;
    else if (action === 'project_page') void api.openExternal(PROJECT_PAGE);
    else if (action === 'scale_in') uiScale.key(1);
    else if (action === 'scale_out') uiScale.key(-1);
    else if (action === 'scale_reset') uiScale.key(0);
    else if (action === 'close_tab') {
      // ⌘W closed the window when there was no tab to close (the system menu did it) — still does.
      if (main && sessions.activeGroup) closeServer(sessions.activeGroup);
      else void getCurrentWindow().close();
    } else if (!main) return;
    else if (action === 'keys') {
      ui.page = 'keys';
      sessions.activeId = null;
    } else if (action === 'new_tab') ui.paletteOpen = true;
    else if (action === 'local_terminal') sessions.openLocal(m.local_terminal());
    else if (action === 'new_host') ui.openHostEditor(null);
    else if (action === 'toggle_sidebar') ui.sidebarOpen = !ui.sidebarOpen;
  }

  onMount(() => {
    void api.appInfo().then((r) => {
      if (r.status !== 'ok') return;
      // The local terminal starts with the app and keeps running (desktop only).
      if (!isHostWindow && ['macos', 'windows', 'linux'].includes(r.data.platform))
        sessions.ensureLocal(m.local_terminal());
    });
    const mq = matchMedia('(max-width: 720px)');
    const sync = () => {
      if (mq.matches && !narrow) ui.sidebarOpen = false;
      narrow = mq.matches;
    };
    sync();
    mq.addEventListener('change', sync);
    return () => mq.removeEventListener('change', sync);
  });

  function typing(e: KeyboardEvent): boolean {
    const t = e.target;
    return (
      t instanceof HTMLElement &&
      (t.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(t.tagName))
    );
  }

  /** The webview's own menu (Look Up, Inspect…) is only useful for text the user can act on. */
  function hideNativeMenu(e: MouseEvent) {
    const t = e.target;
    if (t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA)$/.test(t.tagName)))
      return;
    if (!window.getSelection()?.isCollapsed) return;
    e.preventDefault();
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === '?' && !e.metaKey && !e.ctrlKey && !e.altKey && !typing(e)) {
      e.preventDefault();
      ui.hotkeysOpen = !ui.hotkeysOpen;
      return;
    }
    if (e.ctrlKey && !e.metaKey && !e.altKey && e.code === 'Backquote' && sessions.localTab) {
      e.preventDefault();
      toggleLocal();
      return;
    }
    if (e.ctrlKey && e.key === 'Tab') {
      e.preventDefault();
      sessions.step(e.shiftKey ? -1 : 1);
      return;
    }
    // Wave: ⌘⇧+ / ⌘⇧− / ⌘⇧0 — interface scale (the terminal keeps ⌘+ / ⌘− for its font).
    const z = scaleKey(e);
    if (z !== null) {
      e.preventDefault();
      uiScale.key(z);
      return;
    }
    if (!(isApple ? e.metaKey : e.ctrlKey) || e.altKey) return;
    const k = shortcutKey(e);
    if (!isHostWindow && e.shiftKey && k === 't') {
      const id = sessions.active?.hostId;
      if (id) {
        e.preventDefault();
        connectAnother(id);
      }
      return;
    }
    if (e.shiftKey && k !== 'tab') return;
    if (isHostWindow && ['b', 'k', 't', 'w'].includes(k)) return;
    if (k === 'k') ui.paletteOpen = !ui.paletteOpen;
    else if (k === 'b') ui.sidebarOpen = !ui.sidebarOpen;
    else if (k === 'l') {
      if (vault.canLock) void vault.lock();
    } else if (k === ',') ui.settingsOpen = true;
    else if (k === 't') ui.paletteOpen = true;
    else if (k === 'w' && sessions.activeGroup) closeServer(sessions.activeGroup);
    else if (/^[1-9]$/.test(k)) {
      const g = sessions.visibleGroups[Number(k) - 1];
      if (g) sessions.select(g.current.id);
    } else return;
    e.preventDefault();
  }

  /** ⌃` : to the local terminal and back to where the user was. */
  let beforeLocal: string | null = null;
  function toggleLocal() {
    const local = sessions.localTab;
    if (!local) return;
    if (sessions.activeId !== local.id) {
      beforeLocal = sessions.activeId;
      sessions.select(local.id);
    } else if (beforeLocal && sessions.tabs.some((t) => t.id === beforeLocal))
      sessions.select(beforeLocal);
    else sessions.activeId = null;
  }

  // Sidebar resize: pointer drag on the 4 px handle, 200–420 px.
  // Wave: remember the menu opened / hidden with the button; a pinned menu ends the peek.
  let resizing = $state(false);
  $effect(() => {
    const open = ui.sidebarOpen;
    untrack(() => {
      if (!narrow && !isHostWindow) wavePeek.save(open);
      if (open) wavePeek.close();
    });
  });

  function startResize(e: PointerEvent) {
    e.preventDefault();
    resizing = true;
    const move = (ev: PointerEvent) => (ui.sidebarWidth = Math.min(420, Math.max(200, ev.clientX)));
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      resizing = false;
      settings.saveSidebarWidth();
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  }
  function resizeKey(e: KeyboardEvent) {
    const d = e.key === 'ArrowLeft' ? -16 : e.key === 'ArrowRight' ? 16 : 0;
    if (!d) return;
    ui.sidebarWidth = Math.min(420, Math.max(200, ui.sidebarWidth + d));
    settings.saveSidebarWidth();
  }
</script>

<svelte:window onkeydown={onKeyDown} oncontextmenu={hideNativeMenu} />

<div
  class="shell"
  class:narrow={narrow || isHostWindow}
  class:resizing
  style:--sw="{!isHostWindow && ui.sidebarOpen && !narrow ? ui.sidebarWidth : 0}px"
>
  {#if !isHostWindow}
    <!-- svelte-ignore a11y_no_static_element_interactions (Wave: hover keeps the peeked menu open) -->
    <div
      class="sidebar"
      class:open={ui.sidebarOpen}
      class:peek={wavePeek.open && !ui.sidebarOpen && !narrow}
      inert={!ui.sidebarOpen && !wavePeek.open}
      style:width="{ui.sidebarWidth}px"
      onpointerenter={() => wavePeek.stay()}
      onpointerleave={() => wavePeek.leave()}
    >
      <Sidebar {mac} />
      {#if !narrow && ui.sidebarOpen}
        <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
        <div
          class="grip"
          role="separator"
          aria-orientation="vertical"
          aria-label={m.sidebar_resize()}
          aria-valuenow={ui.sidebarWidth}
          aria-valuemin="200"
          aria-valuemax="420"
          tabindex="0"
          onpointerdown={startResize}
          onkeydown={resizeKey}
        ></div>
      {/if}
    </div>
    {#if !narrow && !ui.sidebarOpen}
      <!-- Wave: the left edge of the window — the hidden menu slides out (like Arc). -->
      <div
        class="wave-edge"
        aria-hidden="true"
        onpointerenter={(e) => wavePeek.edge(e)}
        onpointermove={(e) => wavePeek.edge(e)}
      ></div>
    {/if}
    {#if narrow && ui.sidebarOpen}
      <button class="scrim" aria-label={m.action_close()} onclick={() => (ui.sidebarOpen = false)}
      ></button>
    {/if}
  {/if}
  <div class="main">
    <!-- Wave: in the main window the system buttons hide with the menu, so the bar needs no room for them. -->
    <TabBar
      mac={mac && isHostWindow}
      {windowControls}
      sideShown={!isHostWindow && ui.sidebarOpen && !narrow}
    />
    <main class="content" class:fade={edge.scrolled && edge.fade}>{@render children()}</main>
  </div>
</div>

{#if ui.editor?.kind === 'host'}
  {#key ui.editor.id ?? 'new'}<HostEditor id={ui.editor.id} folderId={ui.editor.folderId} />{/key}
{:else if ui.editor?.kind === 'folder'}
  {#key ui.editor.id ?? 'new'}<FolderEditor id={ui.editor.id} parentId={ui.editor.parentId} />{/key}
{/if}
<CommandPalette />
<SettingsDialog />
{#if ui.sshImportOpen}<SshImportDialog />{/if}
{#if waveUpdate.open}<UpdateSheet />{/if}
<ConfirmDelete />
<HostKeyDialog />
<CloseServerDialog />
<HotkeysDialog />
<Toasts />

<style>
  .shell {
    position: relative;
    display: grid;
    grid-template-columns: var(--sw) minmax(0, 1fr);
    height: 100%;
    overflow: hidden;
  }
  .sidebar {
    position: relative;
    height: 100%;
    overflow: hidden;
    visibility: hidden;
  }
  .sidebar.open {
    visibility: visible;
  }
  /* Wave: the menu slides in and out; hidden, it waits off-screen and slides over the content (like Arc). */
  .shell:not(.narrow) {
    transition: grid-template-columns 200ms var(--ease);
  }
  .shell.resizing {
    transition: none;
  }
  .shell:not(.narrow) .sidebar {
    transition: transform 200ms var(--ease);
  }
  .shell:not(.narrow) .main {
    grid-column: 2;
  }
  .shell:not(.narrow) .sidebar:not(.open) {
    position: absolute;
    z-index: 45;
    top: 0;
    bottom: 0;
    left: 0;
    transform: translateX(calc(-100% - 16px));
    transition:
      transform 200ms var(--ease),
      visibility 0s linear 200ms;
  }
  /* Over a terminal the glass needs more body and a real shadow. */
  .shell:not(.narrow) .sidebar:not(.open) :global(.side) {
    background: color-mix(in oklab, var(--sidebar) 55%, var(--bg));
    box-shadow:
      0 0 0 0.5px var(--sidebar-edge),
      0 18px 48px -12px rgb(0 0 0 / 0.45);
  }
  .shell:not(.narrow) .sidebar.peek {
    visibility: visible;
    transform: none;
    transition: transform 200ms var(--ease);
  }
  .wave-edge {
    position: absolute;
    z-index: 44;
    top: 0;
    bottom: 0;
    left: 0;
    width: 6px;
  }
  .grip {
    position: absolute;
    top: 0;
    right: -2px;
    bottom: 0;
    width: 5px;
    cursor: col-resize;
    z-index: 5;
  }
  .grip:hover,
  .grip:focus-visible {
    background: var(--accent);
    opacity: 0.4;
    outline: none;
  }
  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    height: 100%;
  }
  .content {
    position: relative;
    flex: 1;
    overflow: hidden;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  /* Scroll edge: content dissolves under the toolbar instead of being cut off. */
  .content::before {
    content: '';
    position: absolute;
    inset: 0 0 auto;
    height: 22px;
    z-index: 3;
    background: linear-gradient(var(--bg), transparent);
    opacity: 0;
    pointer-events: none;
    transition: opacity 200ms var(--ease);
  }
  .content.fade::before {
    opacity: 1;
  }
  .narrow {
    grid-template-columns: minmax(0, 1fr);
  }
  .narrow .sidebar {
    position: fixed;
    inset: 0 auto 0 0;
    z-index: 40;
    width: min(320px, 86vw) !important;
    transform: translateX(-100%);
    transition:
      transform 150ms var(--ease),
      visibility 0s linear 150ms;
    box-shadow: var(--shadow);
  }
  .narrow .sidebar.open {
    transform: none;
    transition-delay: 0s;
  }
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 30;
    border: 0;
    background: color-mix(in oklab, var(--text) 18%, transparent);
  }
</style>
