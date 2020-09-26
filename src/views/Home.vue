<template>
  <div class="home h-full overflow-hidden">
    <div class="flex flex-row h-full">
      <div class="flex-shrink-0 border-r w-64">
        <device-select :selected.sync="selectedCalculator"/>
        <div class="overflow-auto h-full px-4 py-4">
          <div v-if="calculator && calculator.info">
            <calc-info :info="calculator.info" :dev="selectedCalculator"/>
            <label class="inline-flex items-center cursor-pointer mr-2 mt-4">
              <input type="checkbox" class="form-checkbox h-5 w-5 text-blue-600 cursor-pointer" v-model="showHidden">
              <span class="mx-2 text-gray-700 select-none">Include hidden files</span>
            </label>
          </div>
        </div>
      </div>
      <div class="w-full">
        <div class="h-full">
          <file-browser v-if="calculator && calculator.info" :dev="selectedCalculator" :show-hidden="showHidden"/>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {Component, Vue, Watch} from 'vue-property-decorator';
import CalcInfo from '@/components/CalcInfo.vue';
import FileBrowser from '@/components/FileBrowser.vue';
import DeviceSelect from "@/components/DeviceSelect.vue";

@Component({
  components: {
    DeviceSelect,
    FileBrowser,
    CalcInfo,
  },
})
export default class Home extends Vue {
  selectedCalculator: string | null = null;
  showHidden = false;

  @Watch('$devices.devices')
  onDeviceChange() {
    if (!this.selectedCalculator) {
      const first = Object.keys(this.$devices.devices)[0];
      if (first) this.selectedCalculator = first;
    } else if (!Object.keys(this.$devices.devices).includes(this.selectedCalculator)) {
      this.selectedCalculator = null;
      // go back and choose the first if available
      this.onDeviceChange();
    }
  }

  @Watch('selectedCalculator')
  onSelectCalculator(dev: string | null) {
    if (dev && !this.$devices.devices[dev].info) {
      console.log('open', dev);
      this.$devices.open(dev);
    }
  }

  get calculator() {
    return this.selectedCalculator && this.$devices.devices[this.selectedCalculator];
  }
}
</script>
