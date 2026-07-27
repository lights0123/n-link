<template>
  <button class="rounded font-bold focus:outline-none focus-visible:ring-2
                 focus-visible:ring-accent transition-colors disabled:opacity-75
                 disabled:cursor-not-allowed"
          :class="[variantClass, sizeClass]" :disabled="disabled">
    <slot/>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  variant?: 'primary' | 'secondary' | 'danger' | 'success';
  size?: 'md' | 'sm';
  disabled?: boolean;
}>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
});

const variantClass = computed(() => ({
  primary: 'bg-accent text-accent-fg',
  secondary: 'bg-raised text-fg border border-edge',
  danger: 'bg-danger text-white',
  success: 'bg-success text-white',
}[props.variant]));

const sizeClass = computed(() => (props.size === 'sm' ? 'px-3 py-2 text-sm' : 'px-6 py-2.5'));
</script>
