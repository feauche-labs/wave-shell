/**
 * Wave: the left menu like in Arc. Hidden with the button — it slides out over the content when the pointer
 * reaches the left edge of the window and hides again when the pointer leaves it. Opened with the button — it
 * stays open, and that is remembered between launches.
 */
import { ui } from '$lib/stores/ui.svelte';

const KEY = 'wave.ui.sidebarOpen';
/** The pointer may leave the menu for a moment (a tooltip, a slip) without it closing. */
const LEAVE_MS = 280;

class WavePeek {
  /** The hidden menu is shown over the content right now. */
  open = $state(false);
  #timer: ReturnType<typeof setTimeout> | undefined;

  /** The saved choice (true when nothing is saved). */
  load(): boolean {
    try {
      return localStorage.getItem(KEY) !== 'false';
    } catch {
      return true;
    }
  }

  save(open: boolean): void {
    try {
      localStorage.setItem(KEY, String(open));
    } catch {
      /* storage unavailable */
    }
  }

  /** The pointer reached the left edge (no button held — not a drag or a text selection). */
  edge(e: PointerEvent): void {
    if (ui.sidebarOpen || e.buttons !== 0) return;
    this.stay();
    this.open = true;
  }

  stay(): void {
    clearTimeout(this.#timer);
  }

  leave(): void {
    if (!this.open) return;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => (this.open = false), LEAVE_MS);
  }

  close(): void {
    clearTimeout(this.#timer);
    this.open = false;
  }
}

export const wavePeek = new WavePeek();
