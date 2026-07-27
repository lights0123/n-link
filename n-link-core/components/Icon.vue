<template>
  <span class="inline-flex items-center" v-html="svg"/>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import feather from 'feather-icons';

const props = withDefaults(defineProps<{
  name: string;
  size?: number;
}>(), {
  size: 20,
});

const svg = computed(() => {
  const icon = feather.icons[props.name as keyof typeof feather.icons];
  if (!icon) throw new Error(`unknown feather icon: ${props.name}`);
  return icon.toSvg({ width: props.size, height: props.size });
});
</script>
