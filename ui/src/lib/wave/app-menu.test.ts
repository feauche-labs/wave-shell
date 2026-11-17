import { describe, expect, it } from 'vitest';
import { emit } from '$lib/api/mock/vault';
import { lastAppMenu } from '$lib/api/mock/services';
import type { AppMenuAction } from '$lib/api';
import { menuLabels, onMenuAction, sendMenu } from './app-menu';

describe('menu bar', () => {
  it('has a text for every item', () => {
    for (const [k, v] of Object.entries(menuLabels())) expect(v.trim(), k).not.toBe('');
  });

  it('sends the texts to the backend', () => {
    sendMenu();
    expect(lastAppMenu?.quit).toBe(menuLabels().quit);
  });

  it('acts only in the window in front', async () => {
    const got: AppMenuAction[] = [];
    const stop = await onMenuAction('main', (a) => got.push(a));
    emit('appMenuAction', { action: 'new_tab', window: 'host-1' });
    emit('appMenuAction', { action: 'settings', window: 'main' });
    stop();
    emit('appMenuAction', { action: 'close_tab', window: 'main' });
    expect(got).toEqual(['settings']);
  });
});
