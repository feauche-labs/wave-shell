/**
 * Wave: a manual reconnect the backend refuses («session is still connected» in the short window
 * before the old task lets go, or the session is gone) left the tab on «Connecting…» for good —
 * the result was ignored and nothing was scheduled.
 */
import { describe, expect, it, vi } from 'vitest';

const calls = vi.hoisted(() => ({
  onEvent: null as null | ((e: unknown) => void),
  writes: [] as Uint8Array[]
}));

vi.mock('$lib/api', () => {
  const ok = <T>(data: T) => Promise.resolve({ status: 'ok' as const, data });
  return {
    api: new Proxy(
      {
        sessionOpen: (
          _h: unknown,
          _c: number,
          _r: number,
          _d: unknown,
          ev: { onmessage: (e: unknown) => void }
        ) => {
          calls.onEvent = (e) => ev.onmessage(e);
          return ok('s-1');
        },
        shellWrite: (_id: string, data: Uint8Array) => {
          calls.writes.push(data);
          return ok(null);
        },
        sessionReconnect: () =>
          Promise.resolve({
            status: 'error' as const,
            error: { code: 'Validation', detail: 'session is still connected' }
          })
      } as Record<string, unknown>,
      { get: (t, k: string) => t[k] ?? (() => ok(null)) }
    )
  };
});

// Wave: imported once, outside the tests — loading the module on a slow Windows runner took longer than a
// test's 5 s limit when it was done inside the first one.
import { Connection } from './connection.svelte';

describe('manual reconnect', () => {
  it('goes back to «disconnected» when the backend refuses', async () => {
    const c = new Connection(
      { id: 't-1', hostId: 'h-1', label: 'srv' } as never,
      () => {},
      () => {}
    );
    await c.open(80, 24);
    calls.onEvent?.({ type: 'Disconnected', reason: 'shell_exited', can_reconnect: true });
    expect(c.phase).toBe('disconnected');
    await c.reconnect();
    expect(c.phase).toBe('disconnected');
    expect(c.canReconnect).toBe(true);
    c.close();
  });

  // Wave: keystrokes went out on the next animation frame (0–16 ms later, more when WebKit is late).
  it('sends a keystroke at once, merging what one event produced', async () => {
    const c = new Connection(
      { id: 't-2', hostId: 'h-1', label: 'srv' } as never,
      () => {},
      () => {}
    );
    await c.open(80, 24);
    calls.onEvent?.({ type: 'Connected', server_banner: null });
    calls.writes.length = 0;
    c.write(new Uint8Array([0x61]));
    c.write(new Uint8Array([0x62]));
    await Promise.resolve();
    expect(calls.writes.map((w) => [...w])).toEqual([[0x61, 0x62]]);
    c.close();
  });
});
