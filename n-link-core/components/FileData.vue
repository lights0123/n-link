<template>
  <div class="flex flex-col h-full">
    <div v-if="files.length === 1">
      <div class="w-full flex flex-col items-center">
        <file-icon :path="files[0].path" :dir="files[0].isDir" :width="96"/>
        <p class="text-center w-full break-words">{{ formatPath(files[0]) }}</p>
        <p v-if="!files[0].isDir" class="mt-2 text-sm">{{ formatSize(files[0].size) }}</p>
      </div>
      <button v-if="!files[0].isDir" class="mt-4 button w-full" @click="download">
        Download
      </button>
      <el-popover
          width="190"
          popper-class="focus:outline-none" @show="newName = formatPath(files[0])" v-model:visible="renamePopup">
        <form @submit.prevent="rename">
          <input v-model="newName"
                 class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                 type="text" placeholder="New name">
          <button class="mt-4 button success w-full" :class="{disabled: !isValidName}" type="submit"
                  :disabled="!isValidName">
            Rename
          </button>
        </form>
        <template #reference>
          <button class="mt-4 button w-full">
            Rename
          </button>
        </template>
      </el-popover>
    </div>
    <div v-else-if="files.length && files.every(file => !file.isDir)">
      <button class="mt-4 button" @click="download">
        Download {{ files.length > 1 ? `${files.length} files` : '' }}
      </button>
    </div>
    <div v-if="files.length">
      <el-popover width="170" popper-class="focus:outline-none" v-model:visible="deletePopup">
        <div>
          Delete {{ files.length }} file{{ files.length === 1 ? '' : 's' }}?
          <div class="flex w-full justify-between">
            <button class="mt-4 button small" @click="deletePopup = false">
              Cancel
            </button>
            <button class="mt-4 button small danger" @click="deleteFiles">
              Delete
            </button>
          </div>
        </div>
        <template #reference>
          <button class="mt-4 button danger w-full">
            Delete {{ files.length > 1 ? `${files.length} files` : '' }}
          </button>
        </template>
      </el-popover>
    </div>
    <div class="flex-grow"/>
    <div class="pb-4">
      <el-popover width="170" popper-class="focus:outline-none" v-model:visible="createDirPopup" @show="newName = ''">
        <template #reference>
          <button class="mt-4 button w-full text-sm">
            Create directory
          </button>
        </template>
        <form @submit.prevent="$devices.createDir(dev, `${path}/${newName}`)">
          <input v-model="newName"
                 class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                 type="text" placeholder="New directory">
          <button class="mt-4 button success w-full" :class="{disabled: !newName.length}" type="submit"
                  :disabled="!newName.length">
            Create
          </button>
        </form>
      </el-popover>
      <button class="mt-4 button w-full" @click="nativeUpload ? upload?.click() : $devices.promptUploadFiles(dev, path)">
        Upload files
      </button>
      <input v-if="nativeUpload" ref="upload" type="file" class="hidden" multiple accept=".tns" @change="uploadNative" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, inject, ref } from 'vue';
import { ElPopover } from 'element-plus';
import 'element-plus/es/components/popover/style/css';
import { DEVICES_KEY, type GenericDevices, type FileInfo } from './devices';
import { filesize } from 'filesize';
import FileIcon from './FileIcon.vue';

const devices = inject(DEVICES_KEY) as GenericDevices;

const props = withDefaults(defineProps<{
  files: FileInfo[];
  dev: string;
  path: string;
  showHidden?: boolean;
  nativeUpload?: boolean;
}>(), {
  showHidden: false,
  nativeUpload: false,
});

const newName = ref('');
const renamePopup = ref(false);
const deletePopup = ref(false);
const createDirPopup = ref(false);
const upload = ref<HTMLInputElement | null>(null);

function formatSize(size: number) {
  return filesize(size, {round: 1});
}

function formatPath({path, isDir}: FileInfo) {
  const name = path.split('/').pop() as string;
  if (props.showHidden || isDir) return name;
  return name.substring(0, name.length - 4);
}

function download() {
  devices.downloadFiles(props.dev, props.files.map(file => [file.path, file.size]));
}

function deleteFiles() {
  deletePopup.value = false;
  devices.delete(props.dev, props.files);
}

const isValidName = computed(() => {
  if (props.showHidden && !props.files[0]?.isDir && !newName.value.endsWith('.tns')) return false;
  return newName.value.length > 0 && !newName.value.includes('/');
});

function rename() {
  if (isValidName.value) {
    renamePopup.value = false;
    const path = props.files[0].path;
    const newPath = path.split('/');
    newPath.pop();
    newPath.push(newName.value + (!props.showHidden && !props.files[0].isDir ? '.tns' : ''));
    devices.move(props.dev, path, newPath.join('/'));
  }
}

function uploadNative(e: Event) {
  const files = (e.target as HTMLInputElement).files;
  devices.uploadFiles(props.dev, props.path, [...(files || [])]);
}
</script>

<style scoped lang="scss">
.button {
  @apply bg-gray-400 text-gray-800 rounded px-6 py-2.5 font-bold;
  &:focus {
    outline: none;
  }

  &.danger {
    @apply bg-red-600 text-white;
  }

  &.success {
    @apply bg-green-500 text-white;
  }

  &.disabled {
    @apply opacity-75 cursor-not-allowed;
  }

  &.small {
    @apply px-3 py-2;
  }
}

</style>
