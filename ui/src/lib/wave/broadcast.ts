/**
 * Wave: «Ввод во все окна» (broadcast input). Every terminal registers how to write to its session; keystrokes
 * typed in a pane of a workspace with broadcast on are also written to the other panes of that workspace.
 */
import { panes } from './split-layout';
import { waveSplit } from './split.svelte';

const writers = new Map<string, (bytes: Uint8Array) => void>();

/** Registers the writer of terminal tab `tabId`; returns the unregister function. */
export function registerWriter(tabId: string, write: (bytes: Uint8Array) => void): () => void {
  writers.set(tabId, write);
  return () => {
    if (writers.get(tabId) === write) writers.delete(tabId);
  };
}

/** Sends `bytes` typed in the pane of server `key` to the other panes, when its workspace broadcasts. */
export function fanOut(key: string, bytes: Uint8Array): void {
  const s = waveSplit.spaceOf(key);
  if (!s?.broadcast) return;
  for (const other of panes(s.root)) {
    if (other === key) continue;
    const t = waveSplit.terminalOf(other);
    if (t) writers.get(t.id)?.(bytes);
  }
}
