<template>
  <div class="h-full">
    <div class="border-b py-2 px-2 header flex">
      <ol class="flex">
        <li v-for="(part, i) in parts" :key="i" class="" :class="{active: i === parts.length - 1}">
          <span class="inline-block cursor-pointer bg-gray-200 px-4 py-1 rounded-full"
                @click="goToIndex(i)">{{ part || 'Home' }}</span>
          <img v-if="i < parts.length - 1" class="inline img-fix" src="~feather-icons/dist/icons/chevron-right.svg"/>
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
        <file-data :files="selected" :show-hidden="showHidden" :dev="dev" :path="path"/>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {Component, Prop, Vue, Watch} from 'vue-property-decorator';
import FileView from './FileView.vue';
import type {FileInfo} from "./devices";
import FileData from "./FileData.vue";
import DeviceQueue from "./DeviceQueue.vue";

@Component({
  asyncComputed: {
    files: {
      async get(this: FileBrowser) {
        this.updateIndex;
        return (await this.$devices.listDir(this.dev, this.path)).map(file => ({
          ...file,
          path: `${this.path}/${file.path}`
        }));
      },
      default: null,
    }
  },
  components: {DeviceQueue, FileData, FileView}
})
export default class FileBrowser extends Vue {
  @Prop({type: String, required: true}) private dev!: string;
  @Prop({type: Boolean, default: false}) private showHidden!: boolean;
  path = '';
  updateIndex = 0;
  selected: FileInfo[] = [];
  files!: FileInfo[] | null;

  @Watch('path')
  onPathChange() {
    this.selected = [];
  }

  get parts() {
    return this.path.split('/');
  }

  get loading() {
    return this.$asyncComputed.files.updating;
  }

  goToIndex(i: number) {
    this.path = this.parts.slice(0, i + 1).join('/');
    this.updateIndex++;
  }
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
