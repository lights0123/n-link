<template>
  <div class="h-full">
    <div class="border-b py-2 px-2 header flex">
      <ol class="flex">
        <li v-for="(part, i) in parts" :key="i" class="" :class="{active: i === parts.length - 1}">
          <span class="inline-block cursor-pointer bg-gray-200 px-4 py-1 rounded-full"
                @click="goToIndex(i)">{{ part || 'Home' }}</span>
          <img v-if="i < parts.length - 1" class="inline img-fix" :src="chevronRight"/>
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
      <div class="overflow-auto h-full px-4 pt-4 w-48 flex-shrink-0 border-l">
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
import chevronRight from 'feather-icons/dist/icons/chevron-right.svg';

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

async function loadFiles() {
  loading.value = true;
  try {
    const contents = await devices.listDir(props.dev, path.value);
    files.value = contents.map((file) => ({
      ...file,
      path: `${path.value}/${file.path}`,
    }));
  } catch (e) {
    console.error(e);
    files.value = null;
  } finally {
    loading.value = false;
  }
}

watch([() => props.dev, path, updateIndex], loadFiles, {immediate: true});

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
  @apply bg-blue-500 text-white;
}

.img-fix {
  margin-top: -1px;
}

.lds-dual-ring {
  display: inline-block;
  width: 80px;
  height: 80px;
}

.lds-dual-ring:after {
  $color: theme('colors.gray.400');
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
