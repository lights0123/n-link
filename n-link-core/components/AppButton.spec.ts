import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import AppButton from './AppButton.vue';

describe('AppButton', () => {
  it('renders slot content', () => {
    const wrapper = mount(AppButton, { slots: { default: 'Download' } });
    expect(wrapper.text()).toBe('Download');
  });

  it('applies the primary accent style by default', () => {
    const wrapper = mount(AppButton, { slots: { default: 'Go' } });
    expect(wrapper.get('button').classes()).toContain('bg-accent');
  });

  it('applies the danger variant', () => {
    const wrapper = mount(AppButton, {
      props: { variant: 'danger' }, slots: { default: 'Delete' },
    });
    expect(wrapper.get('button').classes()).toContain('bg-danger');
  });

  it('emits click when enabled', async () => {
    const wrapper = mount(AppButton, { slots: { default: 'Go' } });
    await wrapper.get('button').trigger('click');
    expect(wrapper.emitted('click')).toHaveLength(1);
  });

  it('reflects the disabled prop', () => {
    const wrapper = mount(AppButton, {
      props: { disabled: true }, slots: { default: 'Go' },
    });
    expect(wrapper.get('button').attributes('disabled')).toBeDefined();
  });
});
