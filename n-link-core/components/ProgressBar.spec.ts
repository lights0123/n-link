import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ProgressBar from './ProgressBar.vue';

const fill = (w: ReturnType<typeof mount>) => w.get('[data-testid="fill"]');

describe('ProgressBar', () => {
  it('renders fraction as a width percentage', () => {
    const wrapper = mount(ProgressBar, { props: { fraction: 0.5 } });
    expect(fill(wrapper).attributes('style')).toContain('width: 50%');
  });

  it('clamps fractions below 0 and above 1', () => {
    expect(fill(mount(ProgressBar, { props: { fraction: -0.3 } }))
      .attributes('style')).toContain('width: 0%');
    expect(fill(mount(ProgressBar, { props: { fraction: 1.8 } }))
      .attributes('style')).toContain('width: 100%');
  });

  it('applies the accent color class by default', () => {
    const wrapper = mount(ProgressBar, { props: { fraction: 0.5 } });
    expect(fill(wrapper).classes()).toContain('bg-accent');
  });

  it('applies a chosen color class', () => {
    const wrapper = mount(ProgressBar, { props: { fraction: 0.5, color: 'ok' } });
    expect(fill(wrapper).classes()).toContain('bg-ok');
  });
});
