<template>
  <div class="home h-full overflow-hidden">
    <div class="flex flex-row h-full">
      <div class="flex-shrink-0 border-r w-64">
        <device-select :selected.sync="selectedCalculator"/>
        <div class="overflow-auto h-full px-4 py-4">
          <div v-if="needsDrivers">
            <h1 class="text-3xl">Drivers required</h1>
            <p>The WinUSB driver is required to use this device.</p>
            <p class="text-center mt-2">
              <a href="#" @click.prevent="installDrivers" class="text-blue-600">See installation instructions</a>
            </p>
          </div>
          <div v-else-if="calculator && !calculator.info" class="flex items-center justify-center h-full">
            <div class="lds-dual-ring"/>
          </div>
          <div v-else-if="calculator && calculator.info">
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
import {open} from 'tauri/api/window';
import CalcInfo from 'n-link-core/components/CalcInfo.vue';
import FileBrowser from 'n-link-core/components/FileBrowser.vue';
import DeviceSelect from "n-link-core/components/DeviceSelect.vue";

function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

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

  @Watch('$devices.hasEnumerated')
  onEnumerated() {
    const first = Object.keys(this.$devices.devices)[0];
    if (first) this.selectedCalculator = first;
  }

  @Watch('$devices.devices')
  async onDeviceChange() {
    if (!this.selectedCalculator) {
      await sleep(1000);
      if (this.selectedCalculator) return;
      const first = Object.keys(this.$devices.devices)[0];
      if (first) this.selectedCalculator = first;
    } else if (!Object.keys(this.$devices.devices).includes(this.selectedCalculator)) {
      this.selectedCalculator = null;
      // go back and choose the first if available
      this.onDeviceChange();
    }
  }

  @Watch('selectedCalculator')
  async onSelectCalculator(dev: string | null) {
    if (dev && !this.$devices.devices[dev].info && !this.$devices.devices[dev].needsDrivers) {
      try {
        await this.$devices.open(dev);
      } catch (e) {
        console.error(e);
        this.selectedCalculator = null;
      }
    }
  }

  get calculator() {
    return this.selectedCalculator && this.$devices.devices[this.selectedCalculator];
  }

  get needsDrivers() {
    return this.selectedCalculator && this.$devices.devices[this.selectedCalculator]?.needsDrivers;
  }

  installDrivers() {
    open('https://lights0123.com/n-link/#windows');
  }
}
</script>

<style lang="scss" scoped>
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
