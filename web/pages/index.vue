<template>
  <div class="home h-full overflow-hidden">
    <div class="flex flex-row h-full">
      <div class="flex flex-col flex-shrink-0 border-r w-64">
        <device-select
          :selected.sync="selectedCalculator"
          :class="webUSB || 'opacity-50 pointer-events-none'"
        />
        <div class="overflow-auto flex flex-col h-full px-4 py-4">
          <div v-if="needsDrivers">
            <h1 class="text-3xl">Drivers required</h1>
            <p>The WinUSB driver is required to use this device.</p>
            <p class="text-center mt-2">
              <a href="#" class="text-blue-600" @click.prevent="installDrivers">
                See installation instructions
              </a>
            </p>
          </div>
          <div
            v-else-if="calculator && !calculator.info"
            class="flex items-center justify-center flex-grow"
          >
            <div class="lds-dual-ring" />
          </div>
          <div v-else-if="calculator && calculator.info">
            <calc-info
              :info="calculator.info"
              :dev="selectedCalculator"
              native-upload
            />
            <label class="inline-flex items-center cursor-pointer mr-2 mt-4">
              <input
                v-model="showHidden"
                type="checkbox"
                class="form-checkbox h-5 w-5 text-blue-600 cursor-pointer"
              />
              <span class="mx-2 text-gray-700 select-none"
                >Include hidden files</span
              >
            </label>
          </div>
          <div v-if="calculator">
            <button class="mt-4 button gray-button" @click="copyLogs">
              Copy logs
            </button>
          </div>
          <div v-if="!(calculator && !calculator.info)" class="flex-grow" />
          <div class="mt-4 select-text">
            <p>
              <a
                href="https://lights0123.com/n-link/"
                target="_blank"
                class="text-blue-600 underline"
              >
                N-Link</a
              >
              ©
              <a
                href="https://lights0123.com/"
                target="_blank"
                class="text-blue-600 underline"
              >
                Ben Schattinger</a
              >
            </p>
            <p>Licensed under the GPL v3.0</p>
          </div>
        </div>
      </div>
      <div class="w-full">
        <div v-if="webUSB" class="h-full">
          <file-browser
            v-if="calculator && calculator.info"
            :dev="selectedCalculator"
            :show-hidden="showHidden"
            native-upload
          />
        </div>
        <div
          v-else
          class="flex flex-col items-center justify-center h-full select-text"
        >
          <p class="text-3xl">Your browser doesn't support WebUSB</p>
          <a
            href="https://lights0123.com/n-link/"
            class="text-xl text-blue-600 underline"
          >
            Check out the desktop version instead
          </a>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component, Vue, Watch } from 'vue-property-decorator';
import CalcInfo from 'n-link-core/components/CalcInfo.vue';
import FileBrowser from 'n-link-core/components/FileBrowser.vue';
import DeviceSelect from 'n-link-core/components/DeviceSelect.vue';
import '@/components/devices';

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
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
  webUSB = true;

  mounted() {
    this.webUSB = !!(navigator as any).usb;
  }

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
    } else if (
      !Object.keys(this.$devices.devices).includes(this.selectedCalculator)
    ) {
      this.selectedCalculator = null;
      // go back and choose the first if available
      this.onDeviceChange();
    }
  }

  @Watch('selectedCalculator')
  async onSelectCalculator(dev: string | null) {
    if (
      dev &&
      !this.$devices.devices[dev].info &&
      !this.$devices.devices[dev].needsDrivers
    ) {
      try {
        await this.$devices.open(dev);
      } catch (e) {
        console.error(e);
        this.selectedCalculator = null;
      }
    }
  }

  get calculator() {
    return (
      this.selectedCalculator && this.$devices.devices[this.selectedCalculator]
    );
  }

  get needsDrivers() {
    return (
      this.selectedCalculator &&
      this.$devices.devices[this.selectedCalculator]?.needsDrivers
    );
  }

  installDrivers() {
    open('https://lights0123.com/n-link/#windows');
  }

  copyLogs() {
    navigator.clipboard.writeText((this.calculator as any)?.log).catch(alert);
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
  content: ' ';
  display: block;
  width: 64px;
  height: 64px;
  margin: 8px;
  border-radius: 50%;
  border: 6px solid $color;
  border-color: $color transparent $color transparent;
  animation: lds-dual-ring 1.2s linear infinite;
}

@keyframes lds-dual-ring {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

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
</style>
