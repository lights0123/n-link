<template>
  <div class="w-full bg-edge rounded-full overflow-hidden">
    <div data-testid="fill" class="py-1 rounded-full" :class="fillClass"
         :style="{ width: `${percent}%` }"/>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  fraction: number;
  color?: 'accent' | 'ok' | 'warn';
}>(), {
  color: 'accent',
});

const percent = computed(() => Math.min(100, Math.max(0, props.fraction * 100)));
const fillClass = computed(() => ({
  accent: 'bg-accent',
  ok: 'bg-ok',
  warn: 'bg-warn',
}[props.color]));
</script>
