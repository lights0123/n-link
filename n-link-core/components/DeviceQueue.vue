<template>
  <div>
    <el-popover width="300" popper-class="focus:outline-none">
      <div>
        <div v-if="!queue.length">
          Nothing to do
        </div>
        <div v-for="(item, i) in queue" :key="item.id" class="flex items-center">
          <div class="min-w-0 flex-grow">
            <p class="truncate">{{ item.desc }}</p>
            <div v-if="i === 0 && device.progress">
              <small class="block tabular-nums">
                {{ (100 - device.progress.remaining / device.progress.total * 100).toFixed(1) }}%
              </small>
              <div class="mb-2 bg-gray-300 rounded-full">
                <div :style="{width: `${100-device.progress.remaining/device.progress.total * 100}%`}"
                     class="bg-teal-400 py-1 rounded-full"/>
              </div>
            </div>
          </div>
          <div class="ml-2 flex-shrink-0">
            <button class="focus:outline-none" :disabled="i===0" :class="i===0 && 'cursor-not-allowed'"
                    @click="device.queue?.splice(i, 1)">
              <img :src="xCircle" :class="i===0 && 'opacity-25'"/>
            </button>
          </div>
        </div>
      </div>
      <template #reference>
        <svg viewBox="-1 -1 2 2" class="circle focus:outline-none">
          <circle r="1" class="bg"/>
          <path class="fg" :d="pathData"/>
        </svg>
      </template>
    </el-popover>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import {ElPopover} from 'element-plus';
import 'element-plus/es/components/popover/style/css';
import type {Device} from './devices';
import xCircle from 'feather-icons/dist/icons/x-circle.svg';

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
    fill: theme('colors.gray.200');
  }

  .fg {
    fill: theme('colors.gray.800');
  }
}
</style>
