/**
 * Wave: a manual reconnect the backend refuses («session is still connected» in the short window
 * before the old task lets go, or the session is gone) left the tab on «Connecting…» for good —
 * the result was ignored and nothing was scheduled.
 */
import { describe, expect, it, vi } from 'vitest';

const calls = vi.hoisted(() => ({ onEvent: null as null | ((e: unknown) => void) }));

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

describe('manual reconnect', () => {
  it('goes back to «disconnected» when the backend refuses', async () => {
    const { Connection } = await import('./connection.svelte');
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
});
