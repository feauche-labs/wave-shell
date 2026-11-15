/**
 * Wave: release notes for the update sheet. The text is the GitHub release body written by release-please:
 *
 *   ## [1.1.0](https://github.com/…/compare/v1.0.0...v1.1.0) (2026-11-16)
 *   ### Добавлено
 *   * свой значок ([26fad0e](https://github.com/…/commit/26fad0e…))
 *
 * It becomes plain sections and items (no HTML, no links), rendered as text.
 */
export interface NotesSection {
  title: string;
  items: string[];
}

/** Markdown links → their text, trailing commit references and emphasis markers removed. */
function cleanItem(line: string): string {
  return line
    .replace(/\s*\(\[[0-9a-f]{7,40}\]\([^)]*\)\)\s*$/i, '')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/\*\*|__|`/g, '')
    .trim();
}

export function parseNotes(body: string | null | undefined): NotesSection[] {
  const sections: NotesSection[] = [];
  let current: NotesSection | null = null;
  for (const raw of (body ?? '').split(/\r?\n/)) {
    const line = raw.trim();
    const heading = /^#{3,6}\s+(.+)$/.exec(line);
    if (heading) {
      current = { title: cleanItem(heading[1] ?? ''), items: [] };
      sections.push(current);
      continue;
    }
    const item = /^[*-]\s+(.+)$/.exec(line);
    if (item) {
      if (!current) {
        current = { title: '', items: [] };
        sections.push(current);
      }
      const text = cleanItem(item[1] ?? '');
      if (text) current.items.push(text);
    }
  }
  return sections.filter((s) => s.items.length > 0);
}
