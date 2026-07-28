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

  it('applies the secondary variant', () => {
    const wrapper = mount(AppButton, { props: { variant: 'secondary' }, slots: { default: 'x' } });
    expect(wrapper.get('button').classes()).toContain('bg-raised');
  });

  it('applies the success variant', () => {
    const wrapper = mount(AppButton, { props: { variant: 'success' }, slots: { default: 'x' } });
    expect(wrapper.get('button').classes()).toContain('bg-success');
  });

  it('applies the small size class', () => {
    const wrapper = mount(AppButton, { props: { size: 'sm' }, slots: { default: 'x' } });
    expect(wrapper.get('button').classes()).toContain('px-3');
  });

  it('does not emit click when disabled', async () => {
    const wrapper = mount(AppButton, { props: { disabled: true }, slots: { default: 'x' } });
    await wrapper.get('button').trigger('click');
    expect(wrapper.emitted('click')).toBeUndefined();
  });
});
