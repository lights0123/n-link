<template>
  <div class="flex flex-wrap min-h-full items-start content-start" @mousedown.exact="selected = []">
    <div class="mb-2 mx-1 w-24 flex flex-col items-center cursor-default" :class="{ selected: selected.includes(i) }"
         v-for="(file, i) in filteredFiles" :key="file.path" @mousedown.ctrl.stop="xorSelection(i)" @mousedown.shift.stop="shiftSelection(i)"
         @mousedown.exact.stop="selected = [i]"
         @dblclick="file.isDir && $emit('nav', file.path)">
      <file-icon :path="file.path" :dir="file.isDir"/>
      <p class="mt-1 text-sm w-full text-center select-none break-words">{{ formatPath(file) }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed, ref, watch} from 'vue';
import type {FileInfo} from './devices';
import FileIcon from './FileIcon.vue';

const props = withDefaults(defineProps<{
  files: FileInfo[];
  showHidden?: boolean;
}>(), {
  files: () => [],
  showHidden: false,
});

const emit = defineEmits<{
  nav: [path: string];
  select: [files: FileInfo[]];
}>();

const selected = ref<number[]>([]);

const filteredFiles = computed(() => {
  if (props.showHidden) return props.files;
  return props.files.filter(file => file.isDir || file.path.endsWith('.tns'));
});

watch(selected, (sel) => {
  emit('select', sel.map(file => filteredFiles.value[file]));
}, {deep: true, immediate: true});

watch(() => props.files, () => {
  selected.value = [];
});

function formatPath({path, isDir}: FileInfo) {
  const name = path.split('/').pop() as string;
  if (props.showHidden || isDir) return name;
  return name.substring(0, name.length - 4);
}

function xorSelection(i: number) {
  if (selected.value.includes(i)) selected.value = selected.value.filter(num => num !== i);
  else selected.value.push(i);
}

function shiftSelection(item: number) {
  const lastSelected = selected.value[selected.value.length - 1];
  if (lastSelected === undefined) {
    selected.value.push(item);
    return;
  }
  const [lower, upper] = item > lastSelected ? [lastSelected, item] : [item, lastSelected];
  for (let i = lower; i <= upper; i++) {
    if (!selected.value.includes(i)) selected.value.push(i);
  }
}
</script>

<style scoped lang="scss">
.selected {
  @apply rounded bg-selected;
  p {
    @apply text-fg;
  }
}
</style>
