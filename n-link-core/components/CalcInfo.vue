<template>
  <div>
    <h2 class="text-center text-xl select-text">{{ info.name }}</h2>
    <h3 class="text-center select-text">OS {{ formatVersion(info.version) }}</h3>
    <h3 class="text-center text-xs select-text">{{ info.id }}</h3>
    <br>
    <div :title="`${formatSize(info.free_storage)} free`">
      <small class="block select-text">
        Storage: {{ formatSize(info.total_storage - info.free_storage) }} / {{ formatSize(info.total_storage) }} used
      </small>
      <div class="mb-2 bg-gray-300 rounded-full">
        <div :style="{width: `${100 - (info.free_storage / info.total_storage) * 100}%`}"
             class="bg-blue-500 py-1 rounded-full"/>
      </div>
    </div>
    <div :title="`${formatSize(info.free_ram)} free`">
      <small class="block select-text">
        RAM: {{ formatSize(info.total_ram - info.free_ram) }} / {{ formatSize(info.total_ram) }} used
      </small>
      <div class="mb-2 bg-gray-300 rounded-full">
        <div :style="{width: `${100 - (info.free_ram / info.total_ram) * 100}%`}"
             class="bg-teal-400 py-1 rounded-full"/>
      </div>
    </div>
    <small class="block select-text">Boot1: {{ formatVersion(info.boot1_version) }}</small>
    <small class="block select-text">Boot2: {{ formatVersion(info.boot2_version) }}</small>
    <button class="mt-4 button" @click="refresh" :disabled="refreshing">
      <div class="flex">
        <div v-if="refreshing" class="lds-dual-ring" />
        Refresh
      </div>
    </button>
    <button class="mt-4 button gray-button" @click="$devices.uploadOs(dev, info.os_extension.split('.').pop())">
      Upload OS
    </button>
  </div>
</template>

<script lang="ts">
import {Component, Prop, Vue} from 'vue-property-decorator';
import type {Info, Version} from './devices';
import fileSize from "filesize";

@Component
export default class FileView extends Vue {
  @Prop({type: Object, required: true}) private info!: Info;
  @Prop({type: String, required: true}) private dev!: string;
  refreshing = false;

  formatSize(size: number) {
    return fileSize(size, {round: 1});
  }

  formatVersion(version: Version) {
    return `${version.major}.${version.minor}.${version.patch}.${version.build}`;
  }

  async refresh() {
    this.refreshing = true;
    try {
      await this.$devices.update(this.dev);
    } catch(e) {
      /* */
    }
    this.refreshing = false;
  }
}
</script>

<style scoped lang="scss">
.button {
  @apply bg-blue-500 text-white rounded px-6 py-2.5 font-bold;
  &:disabled {
    cursor: not-allowed;
    opacity: 0.75;
  }
  &:focus {
    outline: none;
  }
}
.gray-button {
  @apply bg-gray-400 text-gray-800;
}

.lds-dual-ring {
  $scale-factor: 0.2;
  margin-right: 64px * $scale-factor + 8px;
  margin-bottom: 64px * $scale-factor;
  width: 0;
  height: 64px * $scale-factor * 0.8;
  transform: scale($scale-factor);
}
.lds-dual-ring:after {
  content: " ";
  display: block;
  width: 64px;
  height: 64px;
  margin: 8px;
  border-radius: 50%;
  border: 6px solid white;
  border-color: white transparent white transparent;
  animation: lds-dual-ring 1.2s linear infinite;
}

</style>
