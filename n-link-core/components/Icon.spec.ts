import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import Icon from './Icon.vue';

describe('Icon', () => {
  it('renders an inline svg that inherits currentColor', () => {
    const wrapper = mount(Icon, { props: { name: 'refresh-cw' } });
    expect(wrapper.html()).toContain('<svg');
    expect(wrapper.html()).toContain('currentColor');
  });

  it('applies the size to width and height', () => {
    const wrapper = mount(Icon, { props: { name: 'chevron-right', size: 32 } });
    expect(wrapper.html()).toContain('width="32"');
    expect(wrapper.html()).toContain('height="32"');
  });

  it('throws for an unknown icon name', () => {
    expect(() => mount(Icon, { props: { name: 'not-a-real-icon' } }))
      .toThrow(/unknown feather icon/);
  });
});
