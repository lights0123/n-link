<template>
  <div ref="root" class="relative inline-block">
    <div @click="toggle">
      <slot name="trigger" :visible="visible"/>
    </div>
    <div v-if="visible" data-testid="panel"
         class="absolute z-50 mt-1 bg-raised border border-edge rounded-lg shadow-lg
                p-3 select-none" :class="placementClass" :style="panelStyle">
      <slot/>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';

const props = withDefaults(defineProps<{
  width?: number | string;
  placement?: 'bottom-start' | 'bottom-end' | 'top-start';
}>(), {
  placement: 'bottom-start',
});

const visible = defineModel<boolean>('visible', { default: false });
const root = ref<HTMLElement | null>(null);

function toggle() {
  visible.value = !visible.value;
}

function onDocMousedown(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) visible.value = false;
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') visible.value = false;
}

watch(visible, (open) => {
  if (open) {
    document.addEventListener('mousedown', onDocMousedown);
    document.addEventListener('keydown', onKeydown);
  } else {
    document.removeEventListener('mousedown', onDocMousedown);
    document.removeEventListener('keydown', onKeydown);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocMousedown);
  document.removeEventListener('keydown', onKeydown);
});

const placementClass = computed(() => ({
  'bottom-start': 'top-full left-0',
  'bottom-end': 'top-full right-0',
  'top-start': 'bottom-full left-0 mb-1',
}[props.placement]));

const panelStyle = computed(() =>
  props.width === undefined
    ? undefined
    : { width: typeof props.width === 'number' ? `${props.width}px` : props.width });
</script>
