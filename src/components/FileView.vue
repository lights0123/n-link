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

<script lang="ts">
import {Component, Prop, Vue, Watch} from 'vue-property-decorator';
import type {FileInfo} from '@/components/devices';
import FileIcon from "@/components/FileIcon.vue";

@Component({
  components: {FileIcon}
})
export default class FileView extends Vue {
  @Prop({type: Array, default: () => ([])}) private files!: FileInfo[];
  @Prop({type: Boolean, default: false}) private showHidden!: boolean;
  selected: number[] = [];

  @Watch('selected', {deep: true, immediate: true})
  onSelect(selected: number[]) {
    this.$emit('select', selected.map(file => this.filteredFiles[file]));
  }

  @Watch('files')
  onFileChange() {
    this.selected = [];
  }

  get filteredFiles() {
    if (this.showHidden) return this.files;
    return this.files.filter(file => file.isDir || file.path.endsWith('.tns'));
  }

  formatPath({path, isDir}: FileInfo) {
    const name = path.split('/').pop() as string;
    if (this.showHidden || isDir) return name;
    return name.substring(0, name.length - 4);
  }

  xorSelection(i: number) {
    if (this.selected.includes(i)) this.selected = this.selected.filter(num => num !== i);
    else this.selected.push(i);
  }

  shiftSelection(item: number) {
    const lastSelected = this.selected[this.selected.length-1];
    if(lastSelected === undefined) {
      this.selected.push(item);
      return;
    }
    const [lower, upper] = item > lastSelected ? [lastSelected, item] : [item, lastSelected];
    for(let i = lower; i <= upper; i++) {
      if (!this.selected.includes(i))this.selected.push(i);
    }
  }
}
</script>

<style scoped lang="scss">
.selected {
  background-color: #4d88e8;
  @apply rounded;
  p {
    @apply text-white;
  }
}
</style>
