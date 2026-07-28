<template>
  <div class="flex flex-col h-full">
    <div v-if="files.length === 1">
      <div class="w-full flex flex-col items-center">
        <file-icon :path="files[0].path" :dir="files[0].isDir" :width="96"/>
        <p class="text-center w-full break-words text-fg">{{ formatPath(files[0]) }}</p>
        <p v-if="!files[0].isDir" class="mt-2 text-sm text-muted">{{ formatSize(files[0].size) }}</p>
      </div>
      <app-button v-if="!files[0].isDir" class="mt-4 w-full" @click="download">
        Download
      </app-button>
      <popover v-model:visible="renamePopup" :width="190" class="w-full block">
        <template #trigger>
          <app-button class="mt-4 w-full" variant="secondary" @click="newName = formatPath(files[0])">
            Rename
          </app-button>
        </template>
        <form @submit.prevent="rename">
          <input v-model="newName"
                 class="border border-edge rounded w-full py-2 px-3 bg-surface text-fg leading-tight focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
                 type="text" placeholder="New name">
          <app-button class="mt-4 w-full" variant="success" :disabled="!isValidName" type="submit">
            Rename
          </app-button>
        </form>
      </popover>
    </div>
    <div v-else-if="files.length && files.every(file => !file.isDir)">
      <app-button class="mt-4" @click="download">
        Download {{ files.length > 1 ? `${files.length} files` : '' }}
      </app-button>
    </div>
    <div v-if="files.length">
      <popover v-model:visible="deletePopup" :width="170" class="w-full block">
        <template #trigger>
          <app-button class="mt-4 w-full" variant="danger">
            Delete {{ files.length > 1 ? `${files.length} files` : '' }}
          </app-button>
        </template>
        <div class="text-fg">
          Delete {{ files.length }} file{{ files.length === 1 ? '' : 's' }}?
          <div class="flex w-full justify-between">
            <app-button class="mt-4" size="sm" variant="secondary" @click="deletePopup = false">
              Cancel
            </app-button>
            <app-button class="mt-4" size="sm" variant="danger" @click="deleteFiles">
              Delete
            </app-button>
          </div>
        </div>
      </popover>
    </div>
    <div class="flex-grow"/>
    <div class="pb-4">
      <popover v-model:visible="createDirPopup" :width="170" class="w-full block" placement="top-start">
        <template #trigger>
          <app-button class="mt-4 w-full" variant="secondary" size="sm" @click="newName = ''">
            Create directory
          </app-button>
        </template>
        <form @submit.prevent="$devices.createDir(dev, `${path}/${newName}`)">
          <input v-model="newName"
                 class="border border-edge rounded w-full py-2 px-3 bg-surface text-fg leading-tight focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
                 type="text" placeholder="New directory">
          <app-button class="mt-4 w-full" variant="success" :disabled="!newName.length" type="submit">
            Create
          </app-button>
        </form>
      </popover>
      <app-button class="mt-4 w-full" variant="secondary"
                  @click="nativeUpload ? upload?.click() : $devices.promptUploadFiles(dev, path)">
        Upload files
      </app-button>
      <input v-if="nativeUpload" ref="upload" type="file" class="hidden" multiple accept=".tns" @change="uploadNative" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, inject, ref } from 'vue';
import AppButton from './AppButton.vue';
import Popover from './Popover.vue';
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
