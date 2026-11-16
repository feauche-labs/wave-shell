import { describe, expect, it } from 'vitest';
import {
  half,
  insert,
  layout,
  panes,
  remove,
  resize,
  sideAt,
  type LayoutNode
} from './split-layout';

const one: LayoutNode = { kind: 'pane', group: 'A' };

describe('split layout', () => {
  it('drops a pane on each side of another', () => {
    const right = insert(one, 'A', 'B', 'right');
    expect(panes(right)).toEqual(['A', 'B']);
    expect(right).toMatchObject({ kind: 'split', dir: 'row', ratio: 0.5 });
    expect(panes(insert(one, 'A', 'B', 'left'))).toEqual(['B', 'A']);
    expect(insert(one, 'A', 'B', 'top')).toMatchObject({ dir: 'col', a: { group: 'B' } });
    expect(insert(one, 'A', 'B', 'bottom')).toMatchObject({ dir: 'col', b: { group: 'B' } });
  });

  it('splits a pane again and lays out any number of panes', () => {
    // A | B, then C under B → A on the left half, B and C stacked on the right.
    const t = insert(insert(one, 'A', 'B', 'right'), 'B', 'C', 'bottom');
    const { panes: p, splitters } = layout(t);
    expect(p.get('A')).toEqual({ x: 0, y: 0, w: 0.5, h: 1 });
    expect(p.get('B')).toEqual({ x: 0.5, y: 0, w: 0.5, h: 0.5 });
    expect(p.get('C')).toEqual({ x: 0.5, y: 0.5, w: 0.5, h: 0.5 });
    expect(splitters.map((s) => [s.path, s.dir])).toEqual([
      ['', 'row'],
      ['b', 'col']
    ]);
  });

  it('removing a pane gives its space to the neighbour', () => {
    const t = insert(insert(one, 'A', 'B', 'right'), 'B', 'C', 'bottom');
    const left = remove(t, 'B')!;
    expect(panes(left)).toEqual(['A', 'C']);
    expect(layout(left).panes.get('C')).toEqual({ x: 0.5, y: 0, w: 0.5, h: 1 });
    expect(remove(remove(left, 'A')!, 'C')).toBeNull();
  });

  it('resizes a split and keeps both sides usable', () => {
    const t = insert(one, 'A', 'B', 'right');
    expect(layout(resize(t, '', 0.7)).panes.get('B')?.w).toBeCloseTo(0.3);
    expect(resize(t, '', 0.99)).toMatchObject({ ratio: 0.88 });
    expect(resize(t, '', -1)).toMatchObject({ ratio: 0.12 });
  });

  it('picks the nearest side and previews its half', () => {
    expect(sideAt(0.1, 0.5)).toBe('left');
    expect(sideAt(0.9, 0.5)).toBe('right');
    expect(sideAt(0.5, 0.05)).toBe('top');
    expect(sideAt(0.5, 0.95)).toBe('bottom');
    expect(half({ x: 0, y: 0, w: 1, h: 1 }, 'bottom')).toEqual({ x: 0, y: 0.5, w: 1, h: 0.5 });
  });
});
