import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import Popover from './Popover.vue';

function make() {
  return mount(Popover, {
    attachTo: document.body,
    slots: {
      trigger: '<button class="trg">open</button>',
      default: '<div class="content">panel body</div>',
    },
  });
}

describe('Popover', () => {
  it('hides the panel initially', () => {
    expect(make().find('[data-testid="panel"]').exists()).toBe(false);
  });

  it('opens when the trigger is clicked', async () => {
    const wrapper = make();
    await wrapper.get('.trg').trigger('click');
    expect(wrapper.find('[data-testid="panel"]').exists()).toBe(true);
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([true]);
  });

  it('closes on Escape', async () => {
    const wrapper = make();
    await wrapper.get('.trg').trigger('click');
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="panel"]').exists()).toBe(false);
  });

  it('closes on an outside mousedown', async () => {
    const wrapper = make();
    await wrapper.get('.trg').trigger('click');
    document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="panel"]').exists()).toBe(false);
  });

  it('applies a width style to the panel', async () => {
    const wrapper = mount(Popover, {
      attachTo: document.body,
      props: { width: 200 },
      slots: { trigger: '<button class="trg">o</button>', default: '<div>b</div>' },
    });
    await wrapper.get('.trg').trigger('click');
    expect(wrapper.get('[data-testid="panel"]').attributes('style')).toContain('width: 200px');
  });

  it('applies bottom-end placement classes', async () => {
    const wrapper = mount(Popover, {
      attachTo: document.body,
      props: { placement: 'bottom-end' },
      slots: { trigger: '<button class="trg">o</button>', default: '<div>b</div>' },
    });
    await wrapper.get('.trg').trigger('click');
    expect(wrapper.get('[data-testid="panel"]').classes()).toEqual(
      expect.arrayContaining(['top-full', 'right-0']));
  });

  it('applies top-start placement classes', async () => {
    const wrapper = mount(Popover, {
      attachTo: document.body,
      props: { placement: 'top-start' },
      slots: { trigger: '<button class="trg">o</button>', default: '<div>b</div>' },
    });
    await wrapper.get('.trg').trigger('click');
    expect(wrapper.get('[data-testid="panel"]').classes()).toEqual(
      expect.arrayContaining(['bottom-full', 'left-0', 'mb-1']));
  });

  it('applies a string width verbatim', async () => {
    const wrapper = mount(Popover, {
      attachTo: document.body,
      props: { width: '12rem' },
      slots: { trigger: '<button class="trg">o</button>', default: '<div>b</div>' },
    });
    await wrapper.get('.trg').trigger('click');
    expect(wrapper.get('[data-testid="panel"]').attributes('style')).toContain('width: 12rem');
  });
});
