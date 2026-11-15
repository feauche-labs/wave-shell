/**
 * Wave: host tags. The store, the `tag` table and `host.tag_ids` are the author's (search and backups already
 * know tags); this adds the UI side — the list for suggestions, host counts, and «create on first use».
 */
import { api, type Tag } from '$lib/api';
import { tree } from '$lib/stores/tree.svelte';

const norm = (s: string) => s.trim().toLocaleLowerCase();

class WaveTags {
  list = $state.raw<Tag[]>([]);

  async load(): Promise<void> {
    const r = await api.tagList();
    if (r.status === 'ok') this.list = r.data;
  }

  byId(id: string): Tag | undefined {
    return this.list.find((t) => t.id === id);
  }

  /** Names of the given tag ids, in the same order (unknown ids are skipped). */
  names(ids: readonly string[] | undefined): string[] {
    return (ids ?? []).flatMap((id) => this.byId(id)?.name ?? []);
  }

  /** How many hosts carry each tag id. */
  get counts(): Map<string, number> {
    const c = new Map<string, number>();
    for (const h of tree.data.hosts) for (const id of h.tag_ids) c.set(id, (c.get(id) ?? 0) + 1);
    return c;
  }

  /** Tags used by at least one host, most used first. */
  get used(): Tag[] {
    const c = this.counts;
    return this.list
      .filter((t) => (c.get(t.id) ?? 0) > 0)
      .sort((a, b) => (c.get(b.id) ?? 0) - (c.get(a.id) ?? 0) || a.name.localeCompare(b.name));
  }

  /** Suggestions for what is typed: existing tags not chosen yet, prefix matches first. */
  suggest(typed: string, chosen: readonly string[]): Tag[] {
    const q = norm(typed);
    const taken = new Set(chosen.map(norm));
    const c = this.counts;
    return this.list
      .filter((t) => !taken.has(norm(t.name)) && (!q || norm(t.name).includes(q)))
      .sort(
        (a, b) =>
          Number(norm(b.name).startsWith(q)) - Number(norm(a.name).startsWith(q)) ||
          (c.get(b.id) ?? 0) - (c.get(a.id) ?? 0) ||
          a.name.localeCompare(b.name)
      );
  }

  /** Tag ids for names, creating the missing tags (case-insensitive match). `null` on an error. */
  async ensure(names: readonly string[]): Promise<string[] | null> {
    const ids: string[] = [];
    let created = false;
    for (const name of names) {
      const found = this.list.find((t) => norm(t.name) === norm(name));
      if (found) {
        ids.push(found.id);
        continue;
      }
      const r = await api.tagCreate({ name: name.trim(), color: null });
      if (r.status !== 'ok') return null;
      ids.push(r.data.id);
      created = true;
    }
    if (created) await this.load();
    return ids;
  }
}

export const waveTags = new WaveTags();
