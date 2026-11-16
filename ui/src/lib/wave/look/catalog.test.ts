import { describe, expect, it } from 'vitest';
import { flagName } from '$lib/flags/flags';
import { COLORS, ICONS, lookName, parseLook, searchEmoji, searchIcons } from './catalog';

describe('looks', () => {
  it('reads flags, icons and emoji from the one field', () => {
    expect(parseLook('de')).toEqual({ kind: 'flag', value: 'de' });
    expect(parseLook('icon:server')).toEqual({ kind: 'icon', value: 'server' });
    expect(parseLook('emoji:🏠')).toEqual({ kind: 'emoji', value: '🏠' });
    expect(parseLook('weird:x')).toBeNull();
    expect(parseLook(null)).toBeNull();
  });

  it('names a look for labels', () => {
    expect(lookName('flag:de', 'ru', flagName)).toBe(flagName('de', 'ru'));
    expect(lookName('icon:database', 'ru', flagName)).toBe('База');
    expect(lookName('icon:hard-drive', 'en', flagName)).toBe('Hard drive');
    expect(lookName('emoji:🚀', 'ru', flagName)).toBe('🚀');
  });

  it('finds icons and emoji in Russian and English', () => {
    expect(searchIcons('сервер')).toContain('icon:server');
    expect(searchIcons('database')).toContain('icon:database');
    expect(searchEmoji('дом')).toContain('emoji:🏠');
    expect(searchIcons('')).toHaveLength(ICONS.length);
  });

  it('every icon id is valid for the store (lowercase, digits, dashes)', () => {
    for (const i of ICONS) expect(i.id).toMatch(/^[a-z0-9-]{1,40}$/);
    expect(new Set(ICONS.map((i) => i.id)).size).toBe(ICONS.length);
    expect(COLORS).toHaveLength(10);
  });
});
