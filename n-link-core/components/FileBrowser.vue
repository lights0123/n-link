<template>
  <div class="h-full">
    <div class="border-b border-edge py-2 px-2 header flex">
      <ol class="flex">
        <li v-for="(part, i) in parts" :key="i" class="" :class="{active: i === parts.length - 1}">
          <span class="inline-block cursor-pointer bg-raised text-fg px-4 py-1 rounded-full"
                @click="goToIndex(i)">{{ part || 'Home' }}</span>
          <icon v-if="i < parts.length - 1" name="chevron-right" :size="16" class="inline text-muted"/>
        </li>
      </ol>
      <div class="flex-grow"/>
      <device-queue :device="$devices.devices[dev]"/>
    </div>
    <div class="h-full flex w-full body">
      <div class="overflow-auto h-full py-4 flex-grow">
        <file-view v-if="!loading && files" :files="files" :show-hidden="showHidden" @nav="path = $event"
                   @select="selected = $event"/>
        <div v-else-if="loading" class="flex items-center justify-center h-full">
          <div class="lds-dual-ring"/>
        </div>
      </div>
      <div class="overflow-auto h-full px-4 pt-4 w-48 flex-shrink-0 border-l border-edge">
        <file-data :files="selected" :show-hidden="showHidden" :dev="dev" :path="path" :native-upload="nativeUpload"/>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed, inject, ref, watch} from 'vue';
import FileView from './FileView.vue';
import {DEVICES_KEY, type GenericDevices, type FileInfo} from './devices';
import FileData from './FileData.vue';
import DeviceQueue from './DeviceQueue.vue';
import Icon from './Icon.vue';

const devices = inject(DEVICES_KEY) as GenericDevices;

const props = withDefaults(defineProps<{
  dev: string;
  showHidden?: boolean;
  nativeUpload?: boolean;
}>(), {
  showHidden: false,
  nativeUpload: false,
});

const path = ref('');
const updateIndex = ref(0);
const selected = ref<FileInfo[]>([]);

const files = ref<FileInfo[] | null>(null);
const loading = ref(false);

// `silent` keeps the current listing on screen while re-fetching, so the
// refresh after a queued operation doesn't flash the loading spinner.
async function loadFiles(silent = false) {
  if (!silent) loading.value = true;
  try {
    const contents = await devices.listDir(props.dev, path.value);
    files.value = contents.map((file) => ({
      ...file,
      path: `${path.value}/${file.path}`,
    }));
  } catch (e) {
    console.error(e);
    if (!silent) files.value = null;
  } finally {
    loading.value = false;
  }
}

watch([() => props.dev, path, updateIndex], () => loadFiles(), {immediate: true});

// File operations (upload, delete, rename, create directory) are pushed onto
// the device's command queue and run asynchronously, so the listing on screen
// is stale by the time they finish. Re-list once the queue drains.
const busy = computed(() => {
  const device = devices.devices[props.dev];
  if (!device) return false;
  return !!device.running || !!device.queue?.length;
});

watch(busy, (isBusy, wasBusy) => {
  // FileView resets its own selection when the new listing arrives.
  if (wasBusy && !isBusy) loadFiles(true);
});

watch(path, () => {
  selected.value = [];
});

const parts = computed(() => path.value.split('/'));

function goToIndex(i: number) {
  path.value = parts.value.slice(0, i + 1).join('/');
  updateIndex.value++;
}
</script>

<style scoped lang="scss">
.header {
  height: 50px;
}

.body {
  padding-bottom: 50px;
}

.active span {
  @apply bg-accent text-accent-fg;
}

.lds-dual-ring {
  display: inline-block;
  width: 80px;
  height: 80px;
}

.lds-dual-ring:after {
  $color: var(--muted);
  content: " ";
  display: block;
  width: 64px;
  height: 64px;
  margin: 8px;
  border-radius: 50%;
  border: 6px solid $color;
  border-color: $color transparent $color transparent;
  animation: lds-dual-ring 1.2s linear infinite;
}

</style>
