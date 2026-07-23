import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import { defineComponent, h } from 'vue';

describe('vitest setup', () => {
  it('mounts a component', () => {
    const C = defineComponent({ render: () => h('div', 'hello') });
    const wrapper = mount(C);
    expect(wrapper.text()).toBe('hello');
  });
});
