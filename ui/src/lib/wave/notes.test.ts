import { describe, expect, it } from 'vitest';
import { parseNotes } from './notes';

const BODY = `## [1.1.0](https://github.com/feauche-labs/wave-shell/compare/v1.0.0...v1.1.0) (2026-11-16)


### Добавлено

* свой значок Wave Shell — стрелка и бирюзовая волна ([26fad0e](https://github.com/feauche-labs/wave-shell/commit/26fad0e1234567))
* **обновление** одной кнопкой ([abc1234](https://github.com/feauche-labs/wave-shell/commit/abc1234))


### Исправлено

* кнопки в пустом списке хостов ([7f82e0f](https://github.com/feauche-labs/wave-shell/commit/7f82e0f))
`;

describe('parseNotes', () => {
  it('turns a release-please body into sections without links', () => {
    expect(parseNotes(BODY)).toEqual([
      {
        title: 'Добавлено',
        items: ['свой значок Wave Shell — стрелка и бирюзовая волна', 'обновление одной кнопкой']
      },
      { title: 'Исправлено', items: ['кнопки в пустом списке хостов'] }
    ]);
  });

  it('keeps items that come before any heading and link text inside items', () => {
    expect(parseNotes('- see [the docs](https://x.y) now')).toEqual([
      { title: '', items: ['see the docs now'] }
    ]);
  });

  it('is empty for no notes', () => {
    expect(parseNotes(undefined)).toEqual([]);
    expect(parseNotes('## 1.0.0\n\n### Добавлено\n')).toEqual([]);
  });
});
