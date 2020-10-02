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
          popper-class="focus:outline-none" @show="newName = formatPath(files[0])" v-model="renamePopup">
        <form @submit.prevent="rename">
          <input v-model="newName"
                 class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                 type="text" placeholder="New name">
          <button class="mt-4 button success w-full" :class="{disabled: !isValidName}" type="submit"
                  :disabled="!isValidName">
            Rename
          </button>
        </form>
        <button slot="reference" class="mt-4 button w-full">
          Rename
        </button>
      </el-popover>
    </div>
    <div v-else-if="files.length && files.every(file => !file.isDir)">
      <button class="mt-4 button" @click="download">
        Download {{ files.length > 1 ? `${files.length} files` : '' }}
      </button>
    </div>
    <div v-if="files.length">
      <el-popover width="170" popper-class="focus:outline-none" v-model="deletePopup">
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
        <button slot="reference" class="mt-4 button danger w-full">
          Delete {{ files.length > 1 ? `${files.length} files` : '' }}
        </button>
      </el-popover>
    </div>
    <div class="flex-grow"/>
    <div class="pb-4">
      <el-popover width="170" popper-class="focus:outline-none" v-model="createDirPopup" @show="newName = ''">
        <button slot="reference" class="mt-4 button w-full text-sm">
          Create directory
        </button>
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
      <button class="mt-4 button w-full" @click="$devices.promptUploadFiles(dev, path)">
        Upload files
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import {Component, Prop, Vue} from 'vue-property-decorator';
import ElPopover from 'element-ui/packages/popover/src/main.vue';
import 'element-ui/lib/theme-chalk/popover.css';
import fileSize from "filesize";
import type {FileInfo} from './devices';
import FileIcon from "./FileIcon.vue";

@Component({
  components: {FileIcon, ElPopover}
})
export default class FileData extends Vue {
  @Prop({type: Array, default: () => ([])}) private files!: FileInfo[];
  @Prop({type: String, required: true}) private path!: string;
  @Prop({type: Boolean, default: false}) private showHidden!: boolean;
  @Prop({type: String, required: true}) private dev!: string;
  newName = '';
  renamePopup = false;
  deletePopup = false;
  createDirPopup = false;

  formatSize(size: number) {
    return fileSize(size, {round: 1});
  }

  formatPath({path, isDir}: FileInfo) {
    const name = path.split('/').pop() as string;
    if (this.showHidden || isDir) return name;
    return name.substring(0, name.length - 4);
  }

  download() {
    this.$devices.downloadFiles(this.dev, this.files.map(file => [file.path, file.size]));
  }

  deleteFiles() {
    this.deletePopup = false;
    this.$devices.delete(this.dev, this.files);
  }

  get isValidName() {
    if (this.showHidden && !this.files[0]?.isDir && !this.newName.endsWith('.tns')) return false;
    return this.newName.length && !this.newName.includes('/');
  }

  rename() {
    if (this.isValidName) {
      this.renamePopup = false;
      const path = this.files[0].path;
      const newPath = path.split('/');
      newPath.pop();
      newPath.push(this.newName + (!this.showHidden && !this.files[0].isDir ? '.tns' : ''));
      this.$devices.move(this.dev, path, newPath.join('/'));
    }
  }
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
