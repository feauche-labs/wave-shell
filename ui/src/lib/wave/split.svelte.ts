/**
 * Wave: split screen (like Termius workspaces). Drag a server tab out of the tab bar over the terminal: a ghost
 * follows the pointer and the half of the pane it will take is highlighted; drop — the screen splits. The tabs
 * of a workspace merge into one («Рабочая область»): the first server's tab stays in the bar (the anchor), the
 * others hide. The pane you click is the active tab, so the status bar, the side panel and Status/Terminal/Files
 * follow it. Terminals are never re-created: each sits in a PaneFrame that only changes its position.
 */
import { groupKey, sessions, type Tab } from '$lib/stores/sessions.svelte';
import {
  insert,
  layout,
  panes,
  remove,
  resize,
  sideAt,
  type Box,
  type LayoutNode,
  type Side,
  type Splitter
} from './split-layout';

export interface Workspace {
  id: number;
  /** The group whose tab stands for the workspace in the tab bar. */
  anchor: string;
  root: LayoutNode;
  /** A pane shown on the whole stage for a while. */
  max: string | null;
}

export interface DragState {
  key: string;
  label: string;
  flag: string | null;
  x: number;
  y: number;
  over: { group: string; side: Side } | null;
}

const FULL: Box = { x: 0, y: 0, w: 1, h: 1 };

class WaveSplit {
  spaces = $state.raw<Workspace[]>([]);
  drag = $state.raw<DragState | null>(null);
  /** The terminal stage (SplitOverlay) — pane boxes are fractions of it. */
  stage: HTMLElement | null = null;
  #next = 1;

  spaceOf(key: string | null | undefined): Workspace | undefined {
    return key ? this.spaces.find((s) => panes(s.root).includes(key)) : undefined;
  }

  /** A workspace member other than the anchor: not shown in the tab bar. */
  hidden(key: string): boolean {
    const s = this.spaceOf(key);
    return !!s && s.anchor !== key;
  }

  isAnchor(key: string): boolean {
    return this.spaceOf(key)?.anchor === key;
  }

  count(key: string): number {
    const s = this.spaceOf(key);
    return s ? panes(s.root).length : 0;
  }

  /** `b` is in the same workspace as `a` (tab bar: the workspace tab is active while any pane is). */
  together(a: string, b: string | null): boolean {
    const s = this.spaceOf(a);
    return !!s && !!b && panes(s.root).includes(b);
  }

  /** The workspace on screen: the one holding the active tab's server. */
  get current(): Workspace | undefined {
    return this.spaceOf(sessions.activeGroup);
  }

  terminalOf(key: string): Tab | undefined {
    return sessions.groups.find((g) => g.key === key)?.tabs.find((t) => t.kind === 'terminal');
  }

  /** Where a terminal tab sits in the visible workspace (fractions of the stage), or null. */
  boxOf(tab: Tab): Box | null {
    const s = this.current;
    if (!s || tab.local) return null;
    const key = groupKey(tab);
    if (this.terminalOf(key)?.id !== tab.id) return null;
    if (s.max) return s.max === key ? FULL : null;
    return layout(s.root).panes.get(key) ?? null;
  }

  get splitters(): Splitter[] {
    const s = this.current;
    return s && !s.max ? layout(s.root).splitters : [];
  }

  /** Box of a pane for the drop preview; the single active terminal counts as one full pane. */
  targetBox(group: string): Box | null {
    const s = this.current;
    if (s && !s.max) return layout(s.root).panes.get(group) ?? null;
    return group === sessions.activeGroup ? FULL : null;
  }

  focus(key: string): void {
    const t = this.terminalOf(key);
    if (t && sessions.activeId !== t.id) sessions.select(t.id);
  }

  #update(id: number, fn: (s: Workspace) => Workspace | null): void {
    this.spaces = this.spaces.flatMap((s) => {
      if (s.id !== id) return [s];
      const next = fn(s);
      return next && panes(next.root).length > 1 ? [next] : [];
    });
  }

  /** Puts server `key` next to pane `target` on `side`. False when it cannot go there. */
  place(key: string, target: string, side: Side): boolean {
    if (key === target || !this.terminalOf(key) || !this.terminalOf(target)) return false;
    // A workspace tab (several panes) is not dropped into another one.
    if (this.count(key) > 1 || this.together(target, key)) return false;
    let s = this.spaceOf(target);
    if (!s) {
      s = { id: this.#next++, anchor: target, root: { kind: 'pane', group: target }, max: null };
      this.spaces = [...this.spaces, s];
    }
    this.#update(s.id, (x) => ({ ...x, root: insert(x.root, target, key, side), max: null }));
    this.focus(key);
    return true;
  }

  /** Tab bar callback on drop outside the bar: split if the pointer was over a pane. */
  drop = (key: string): boolean => {
    const over = this.drag?.over;
    this.drag = null;
    return !!over && this.place(key, over.group, over.side);
  };

  /** «Return to a tab»: the server leaves the workspace and gets its own tab back. */
  detach(key: string): void {
    const s = this.spaceOf(key);
    if (!s) return;
    const rest = remove(s.root, key);
    this.#update(s.id, (x) =>
      rest
        ? {
            ...x,
            root: rest,
            max: x.max === key ? null : x.max,
            anchor: x.anchor === key ? (panes(rest)[0] ?? x.anchor) : x.anchor
          }
        : null
    );
    const left = rest ? panes(rest)[0] : undefined;
    if (left && sessions.activeGroup === key) this.focus(left);
  }

  /** Disconnects a server shown in a pane. */
  close(key: string): void {
    this.detach(key);
    sessions.closeGroup(key);
  }

  toggleMax(key: string): void {
    const s = this.spaceOf(key);
    if (s) this.#update(s.id, (x) => ({ ...x, max: x.max === key ? null : key }));
    this.focus(key);
  }

  /** Splits the workspace of `key` back into separate tabs (nothing is disconnected). */
  dissolve(key: string): void {
    const s = this.spaceOf(key);
    if (s) this.spaces = this.spaces.filter((x) => x.id !== s.id);
  }

  setRatio(path: string, ratio: number): void {
    const s = this.current;
    if (s) this.#update(s.id, (x) => ({ ...x, root: resize(x.root, path, ratio) }));
  }

  /** Drops panes whose server has no terminal any more (closed tab, disconnected). */
  prune(): void {
    for (const s of this.spaces)
      for (const key of panes(s.root)) if (!this.terminalOf(key)) this.detach(key);
  }

  /** Follows a tab being dragged from the tab bar (TabDrag keeps reordering inside the bar). */
  track(
    e: PointerEvent,
    key: string,
    label: string,
    flag: string | null,
    bar: () => DOMRect
  ): void {
    const id = e.pointerId;
    const move = (ev: PointerEvent) => {
      if (ev.pointerId !== id) return;
      const r = bar();
      const inBar =
        ev.clientX >= r.left &&
        ev.clientX <= r.right &&
        ev.clientY >= r.top &&
        ev.clientY <= r.bottom;
      this.drag = inBar
        ? null
        : {
            key,
            label,
            flag,
            x: ev.clientX,
            y: ev.clientY,
            over: this.#hit(ev.clientX, ev.clientY, key)
          };
    };
    const end = (ev: PointerEvent) => {
      if (ev.pointerId !== id) return;
      window.removeEventListener('pointermove', move, true);
      window.removeEventListener('pointerup', end, true);
      window.removeEventListener('pointercancel', end, true);
      // TabDrag reads `drag` in its own pointerup (drop); clear after it.
      setTimeout(() => (this.drag = null), 0);
    };
    window.addEventListener('pointermove', move, true);
    window.addEventListener('pointerup', end, true);
    window.addEventListener('pointercancel', end, true);
  }

  #hit(x: number, y: number, dragging: string): DragState['over'] {
    const r = this.stage?.getBoundingClientRect();
    if (!r || x < r.left || x > r.right || y < r.top || y > r.bottom) return null;
    const fx = (x - r.left) / r.width;
    const fy = (y - r.top) / r.height;
    const s = this.current;
    const boxes: [string, Box][] =
      s && !s.max
        ? [...layout(s.root).panes]
        : sessions.active?.kind === 'terminal' && sessions.activeGroup
          ? [[sessions.activeGroup, FULL]]
          : [];
    for (const [group, b] of boxes) {
      if (group === dragging) continue;
      if (fx >= b.x && fx <= b.x + b.w && fy >= b.y && fy <= b.y + b.h)
        return { group, side: sideAt((fx - b.x) / b.w, (fy - b.y) / b.h) };
    }
    return null;
  }
}

export const waveSplit = new WaveSplit();
