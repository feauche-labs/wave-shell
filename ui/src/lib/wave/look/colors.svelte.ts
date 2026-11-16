/**
 * Wave: folder colours (showcase: variant 3 — a line of the folder's colour along its hosts, the name tinted).
 * Kept in settings (`wave.folder_colors`: folder id → colour id), like the snippet layout.
 */
import { api } from '$lib/api';
import { COLOR_BY_ID } from './catalog';

const KEY = 'wave.folder_colors';

class FolderColors {
  map = $state.raw<Record<string, string>>({});

  async load(): Promise<void> {
    const r = await api.settingsGet(KEY);
    if (r.status !== 'ok' || !r.data || typeof r.data !== 'object' || Array.isArray(r.data)) return;
    this.map = Object.fromEntries(
      Object.entries(r.data).filter(([, v]) => typeof v === 'string' && COLOR_BY_ID.has(v))
    ) as Record<string, string>;
  }

  /** Colour id of a folder, or null. */
  of(folderId: string | null | undefined): string | null {
    return (folderId && this.map[folderId]) || null;
  }

  /** The colour itself (hex), or null. */
  hex(folderId: string | null | undefined): string | null {
    const id = this.of(folderId);
    return id ? (COLOR_BY_ID.get(id)?.hex ?? null) : null;
  }

  set(folderId: string, color: string | null): void {
    const next = { ...this.map };
    if (color) next[folderId] = color;
    else delete next[folderId];
    this.map = next;
    void api.settingsSet(KEY, next);
  }
}

export const folderColors = new FolderColors();
