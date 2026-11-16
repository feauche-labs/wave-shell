import { describe, expect, it } from 'vitest';
import { SCALES, stepScale } from './ui-scale.svelte';

describe('stepScale', () => {
  it('moves one step and stops at the ends', () => {
    expect(stepScale(1, 1)).toBe(1.1);
    expect(stepScale(1.1, 1)).toBe(1.25);
    expect(stepScale(1, -1)).toBe(0.9);
    expect(stepScale(0.9, -1)).toBe(0.9);
    expect(stepScale(1.5, 1)).toBe(1.5);
  });

  it('starts from the nearest step for a value that is not one', () => {
    expect(stepScale(1.05, 0)).toBe(1.1);
    expect(stepScale(3, -1)).toBe(SCALES[SCALES.length - 2]);
  });
});
