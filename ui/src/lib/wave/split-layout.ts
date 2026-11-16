/**
 * Wave: split-screen layout (like Termius workspaces) — a binary tree of panes. Each split divides its box
 * left/right (`row`) or top/bottom (`col`) by `ratio`; any pane can be split again, so the number of panes is
 * unlimited. Pure functions only (tested in split-layout.test.ts).
 */

/** A pane shows the terminal of one server tab group (`sessions` group key). */
export type LayoutNode =
  | { kind: 'pane'; group: string }
  | { kind: 'split'; dir: 'row' | 'col'; ratio: number; a: LayoutNode; b: LayoutNode };

export type Side = 'left' | 'right' | 'top' | 'bottom';

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** A splitter: its path from the root ('a'/'b' steps), direction and the box of the split it divides. */
export interface Splitter {
  path: string;
  dir: 'row' | 'col';
  box: Box;
  ratio: number;
}

export const MIN_RATIO = 0.12;
const clampRatio = (r: number) => Math.min(1 - MIN_RATIO, Math.max(MIN_RATIO, r));

export function panes(node: LayoutNode): string[] {
  return node.kind === 'pane' ? [node.group] : [...panes(node.a), ...panes(node.b)];
}

/** Puts `group` next to the pane `target` on `side` (the target keeps the other half). */
export function insert(node: LayoutNode, target: string, group: string, side: Side): LayoutNode {
  if (node.kind === 'pane') {
    if (node.group !== target) return node;
    const added: LayoutNode = { kind: 'pane', group };
    const dir = side === 'left' || side === 'right' ? 'row' : 'col';
    const first = side === 'left' || side === 'top';
    return { kind: 'split', dir, ratio: 0.5, a: first ? added : node, b: first ? node : added };
  }
  return {
    ...node,
    a: insert(node.a, target, group, side),
    b: insert(node.b, target, group, side)
  };
}

/** Removes a pane; its sibling takes the freed space. `null` when nothing is left. */
export function remove(node: LayoutNode, group: string): LayoutNode | null {
  if (node.kind === 'pane') return node.group === group ? null : node;
  const a = remove(node.a, group);
  const b = remove(node.b, group);
  if (!a) return b;
  if (!b) return a;
  return { ...node, a, b };
}

/** Sets the ratio of the split at `path`. */
export function resize(node: LayoutNode, path: string, ratio: number): LayoutNode {
  if (node.kind === 'pane') return node;
  if (!path) return { ...node, ratio: clampRatio(ratio) };
  const [step, rest] = [path[0], path.slice(1)];
  return step === 'a'
    ? { ...node, a: resize(node.a, rest, ratio) }
    : { ...node, b: resize(node.b, rest, ratio) };
}

/** Boxes of every pane and every splitter inside `box` (fractions of the stage, 0–1). */
export function layout(
  node: LayoutNode,
  box: Box = { x: 0, y: 0, w: 1, h: 1 },
  path = ''
): { panes: Map<string, Box>; splitters: Splitter[] } {
  if (node.kind === 'pane') return { panes: new Map([[node.group, box]]), splitters: [] };
  const [ba, bb]: [Box, Box] =
    node.dir === 'row'
      ? [
          { ...box, w: box.w * node.ratio },
          { ...box, x: box.x + box.w * node.ratio, w: box.w * (1 - node.ratio) }
        ]
      : [
          { ...box, h: box.h * node.ratio },
          { ...box, y: box.y + box.h * node.ratio, h: box.h * (1 - node.ratio) }
        ];
  const la = layout(node.a, ba, path + 'a');
  const lb = layout(node.b, bb, path + 'b');
  return {
    panes: new Map([...la.panes, ...lb.panes]),
    splitters: [{ path, dir: node.dir, box, ratio: node.ratio }, ...la.splitters, ...lb.splitters]
  };
}

/** Which side of a pane the pointer (`px`, `py` inside the pane box, 0–1) is closest to. */
export function sideAt(px: number, py: number): Side {
  const d = { left: px, right: 1 - px, top: py, bottom: 1 - py };
  return (Object.keys(d) as Side[]).reduce((best, s) => (d[s] < d[best] ? s : best), 'right');
}

/** The half of `box` a pane dropped on `side` will take — the drop preview. */
export function half(box: Box, side: Side): Box {
  if (side === 'left') return { ...box, w: box.w / 2 };
  if (side === 'right') return { ...box, x: box.x + box.w / 2, w: box.w / 2 };
  if (side === 'top') return { ...box, h: box.h / 2 };
  return { ...box, y: box.y + box.h / 2, h: box.h / 2 };
}
