/**
 * Wave: update one click away. The sidebar banner (author's GitHub check, `settings.update`) opens the update
 * sheet; the sheet asks the Tauri updater for `latest.json` of our releases (signed packages only), shows
 * what is new, downloads with progress, installs and relaunches. Outside the real app (browser demo) and on
 * any failure the sheet offers the release page instead.
 */
import { isTauri } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { api } from '$lib/api';

export type UpdatePhase =
  'checking' | 'ready' | 'downloading' | 'installing' | 'none' | 'unsupported' | 'error';

/** Real Tauri app (not the browser demo with the mock API). */
const canSelfUpdate = (): boolean => import.meta.env.VITE_API === 'real' && isTauri();

class WaveUpdate {
  /** The update sheet is shown. */
  open = $state(false);
  phase = $state<UpdatePhase>('checking');
  /** Version that will be installed. */
  version = $state('');
  /** Version running now. */
  current = $state('');
  /** Release notes (markdown body of the release). */
  notes = $state('');
  downloaded = $state(0);
  /** Size of the package in bytes, 0 when the server does not say. */
  total = $state(0);
  error = $state('');
  /** Newer version found by the background check (`null`: nothing new, not checked, or the demo). */
  available = $state<string | null>(null);
  #update: Update | null = null;
  #lastCheck = 0;

  /**
   * Background check through the updater itself: reads `latest.json` from our releases — no GitHub API, so no
   * 60-requests-per-hour limit per address (shared by everyone behind one VPN exit). Quiet on any error.
   */
  async detect(): Promise<void> {
    if (!canSelfUpdate() || this.busy) return;
    this.#lastCheck = Date.now();
    try {
      const found = await check();
      this.#update = found;
      this.available = found?.version ?? null;
      if (found) {
        this.version = found.version;
        this.current = found.currentVersion;
        this.notes = found.body ?? '';
      }
    } catch {
      /* offline, no release yet — the author's GitHub check (settings.update) still works as a fallback */
    }
  }

  /** Checks now, every 6 h and on window focus at most once an hour — like the author's check. */
  watch(enabled: () => boolean): () => void {
    const run = () => {
      if (enabled()) void this.detect();
    };
    run();
    const timer = setInterval(run, 6 * 3600_000);
    const onFocus = () => {
      if (Date.now() - this.#lastCheck > 3600_000) run();
    };
    window.addEventListener('focus', onFocus);
    return () => {
      clearInterval(timer);
      window.removeEventListener('focus', onFocus);
    };
  }

  /** True while a download or the install runs (the sheet may be hidden meanwhile). */
  get busy(): boolean {
    return this.phase === 'downloading' || this.phase === 'installing';
  }

  /** Download progress, 0–100. */
  get percent(): number {
    return this.total > 0 ? Math.min(100, Math.round((this.downloaded / this.total) * 100)) : 0;
  }

  /** Opens the sheet and asks the updater what would be installed. */
  async show(): Promise<void> {
    this.open = true;
    if (this.busy) return;
    this.error = '';
    if (!this.current) {
      const info = await api.appInfo();
      if (info.status === 'ok') this.current = info.data.version;
    }
    if (!canSelfUpdate()) {
      this.phase = 'unsupported';
      return;
    }
    this.phase = 'checking';
    try {
      const found = await check();
      this.#update = found;
      if (!found) {
        this.phase = 'none';
        return;
      }
      this.version = found.version;
      this.current = found.currentVersion;
      this.notes = found.body ?? '';
      this.phase = 'ready';
    } catch (e) {
      this.fail(e);
    }
  }

  /** Downloads, installs and relaunches. On Windows the installer closes the app itself. */
  async install(): Promise<void> {
    const found = this.#update;
    if (!found || this.phase !== 'ready') return;
    this.phase = 'downloading';
    this.downloaded = 0;
    this.total = 0;
    try {
      await found.downloadAndInstall((ev) => {
        if (ev.event === 'Started') this.total = ev.data.contentLength ?? 0;
        else if (ev.event === 'Progress') this.downloaded += ev.data.chunkLength;
        else this.phase = 'installing';
      });
      this.phase = 'installing';
      await relaunch();
    } catch (e) {
      this.fail(e);
    }
  }

  close(): void {
    this.open = false;
  }

  private fail(e: unknown): void {
    this.phase = 'error';
    this.error = e instanceof Error ? e.message : String(e);
  }
}

export const waveUpdate = new WaveUpdate();
