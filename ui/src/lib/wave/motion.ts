/** Wave: shared motion — quick (200 ms) but smooth slides for panels that come in from a side. */
import { cubicOut } from 'svelte/easing';

/** A side panel in a row: its width grows from zero, so the neighbour slides aside instead of jumping. */
export function waveSide(node: HTMLElement, { duration = 200 }: { duration?: number } = {}) {
  const width = node.offsetWidth;
  return {
    duration,
    easing: cubicOut,
    css: (t: number) => `width: ${t * width}px; opacity: ${Math.min(1, t * 1.6)}; overflow: hidden;`
  };
}
