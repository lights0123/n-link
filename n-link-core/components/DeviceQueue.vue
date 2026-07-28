<template>
  <div>
    <popover :width="300" placement="bottom-end">
      <template #trigger>
        <svg viewBox="-1 -1 2 2" class="circle focus:outline-none">
          <circle r="1" class="bg"/>
          <path class="fg" :d="pathData"/>
        </svg>
      </template>
      <div v-if="!queue.length" class="text-muted">
        Nothing to do
      </div>
      <div v-for="(item, i) in queue" :key="item.id" class="flex items-center text-fg">
        <div class="min-w-0 flex-grow">
          <p class="truncate">{{ item.desc }}</p>
          <div v-if="i === 0 && device.progress">
            <small class="block tabular-nums text-muted">
              {{ (100 - device.progress.remaining / device.progress.total * 100).toFixed(1) }}%
            </small>
            <progress-bar class="mb-2"
                          :fraction="1 - device.progress.remaining / device.progress.total" color="ok"/>
          </div>
        </div>
        <div class="ml-2 flex-shrink-0">
          <button class="focus:outline-none text-fg" :disabled="i===0" :class="i===0 && 'cursor-not-allowed'"
                  @click="device.queue?.splice(i, 1)">
            <icon name="x-circle" :size="20" :class="i===0 && 'opacity-25'"/>
          </button>
        </div>
      </div>
    </popover>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import type {Device} from './devices';
import Popover from './Popover.vue';
import ProgressBar from './ProgressBar.vue';
import Icon from './Icon.vue';

function getCoordinatesForPercent(percent: number) {
  const x = Math.cos(2 * Math.PI * percent);
  const y = Math.sin(2 * Math.PI * percent);

  return [x, y];
}

function trimPath(path: string) {
  return path.split(/[\\/]/).pop() as string;
}

const props = defineProps<{
  device: Device;
}>();

const pathData = computed(() => {
  const {progress, queue} = props.device;
  const length = (queue?.length || 0);
  let percent = progress ? (1 - progress.remaining / progress.total) : 1 / (length);
  if (!Number.isFinite(percent)) percent = 0;

  const [startX, startY] = getCoordinatesForPercent(0);
  const [endX, endY] = getCoordinatesForPercent(percent);

  const largeArcFlag = percent > .5 ? 1 : 0;
  return [
    `M ${startX} ${startY}`,
    `A 1 1 0 ${largeArcFlag} 1 ${endX} ${endY}`,
    `L 0 0`,
  ].join(' ');
});

const queue = computed(() => {
  if (!props.device.queue) return [];
  return props.device.queue.map(item => {
    let desc = '';
    if (item.action === 'download') desc = `Download ${trimPath(item.path[0])}`;
    else if (item.action === 'upload') desc = `Upload ${'src' in item ? trimPath(item.src) : item.file.name}`;
    else if (item.action === 'uploadOs') desc = 'Upload OS';
    else if (item.action === 'deleteFile') desc = `Delete file ${item.path}`;
    else if (item.action === 'deleteDir') desc = `Delete directory ${item.path}`;
    else if (item.action === 'createDir') desc = `Create directory ${item.path}`;
    else if (item.action === 'copy') desc = `Copy file ${item.src} to ${item.dest}`;
    else if (item.action === 'move') desc = `Move file ${item.src} to ${item.dest}`;
    return {
      id: item.id,
      desc,
    };
  });
});
</script>

<style scoped lang="scss">
.circle {
  width: 32px;
  height: 32px;
  transform: rotate(-90deg);

  .bg {
    fill: var(--edge);
  }

  .fg {
    fill: var(--accent);
  }
}
</style>
