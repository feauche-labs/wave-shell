/**
 * Wave: interface scale (Settings → General, showcase 16.11.2026 variant A «steps»). Scales the whole window —
 * sidebar, sheets and the terminal text — with the webview's own zoom (CSS `zoom` would put xterm's mouse
 * selection out of place); the browser demo falls back to CSS `zoom`. Kept in localStorage like the density,
 * so it is known before unlock and shared with the separate host windows (they follow via `storage`).
 */
import { isTauri } from '@tauri-apps/api/core';
import { isApple } from '$lib/util/platform';

export const SCALES = [0.9, 1, 1.1, 1.25, 1.5] as const;
const KEY = 'wave.ui.scale';

function read(): number {
  try {
    const v = Number(localStorage.getItem(KEY));
    return (SCALES as readonly number[]).includes(v) ? v : 1;
  } catch {
    return 1;
  }
}

async function apply(z: number): Promise<void> {
  if (typeof document === 'undefined') return;
  if (isTauri()) {
    try {
      const { getCurrentWebview } = await import('@tauri-apps/api/webview');
      await getCurrentWebview().setZoom(z);
    } catch {
      return;
    }
  } else {
    document.documentElement.style.zoom = z === 1 ? '' : String(z);
  }
  // Terminals re-measure their rows and columns.
  window.dispatchEvent(new Event('resize'));
}

/** The nearest step to `z`, moved by `by` steps. */
export function stepScale(z: number, by: number): number {
  let i = SCALES.findIndex((s) => s >= z);
  if (i < 0) i = SCALES.length - 1;
  return SCALES[Math.min(SCALES.length - 1, Math.max(0, i + by))]!;
}

/** ⌘⇧+ / ⌘⇧− / ⌘⇧0 (Mac only: on Windows / Linux Ctrl+Alt is AltGr, and Ctrl+Shift is the terminal's). */
export function scaleKey(e: KeyboardEvent): 1 | -1 | 0 | null {
  if (!isApple || !e.metaKey || !e.shiftKey || e.ctrlKey || e.altKey) return null;
  if (e.code === 'Equal' || e.code === 'NumpadAdd') return 1;
  if (e.code === 'Minus' || e.code === 'NumpadSubtract') return -1;
  if (e.code === 'Digit0' || e.code === 'Numpad0') return 0;
  return null;
}

class UiScale {
  value = $state(read());

  set(z: number): void {
    this.value = z;
    try {
      if (z === 1) localStorage.removeItem(KEY);
      else localStorage.setItem(KEY, String(z));
    } catch {
      /* storage unavailable: this session only */
    }
    void apply(z);
  }

  /** A key from `scaleKey`: one step up / down, or back to 100 %. */
  key(k: 1 | -1 | 0): void {
    this.set(k === 0 ? 1 : stepScale(this.value, k));
  }
}

export const uiScale = new UiScale();

if (typeof window !== 'undefined') {
  if (uiScale.value !== 1) void apply(uiScale.value);
  window.addEventListener('storage', (e) => {
    if (e.key !== KEY && e.key !== null) return;
    uiScale.value = read();
    void apply(uiScale.value);
  });
}
