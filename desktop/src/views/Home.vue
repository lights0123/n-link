<template>
  <div class="home h-full overflow-hidden">
    <div class="flex flex-row h-full">
      <div class="flex-shrink-0 border-r border-edge w-64">
        <device-select v-model:selected="selectedCalculator"/>
        <div class="overflow-auto h-full px-4 py-4">
          <div v-if="needsDrivers && !isLinux">
            <h1 class="text-3xl">Drivers required</h1>
            <p>The WinUSB driver is required to use this device.</p>
            <p class="text-center mt-2">
              <a href="#" @click.prevent="installDrivers()" class="text-accent">See installation instructions</a>
            </p>
          </div>
          <div v-else-if="needsDrivers && isLinux">
            <h1 class="text-3xl">udev rules required</h1>
            <p>udev rules are required to access this device.</p>
            <p class="text-center mt-2">
              <a href="#" @click.prevent="installDrivers('linux')" class="text-accent">See installation
                instructions</a>
            </p>
          </div>
          <div v-else-if="calculator && !calculator.info" class="flex items-center justify-center h-full">
            <div class="lds-dual-ring"/>
          </div>
          <div v-else-if="calculator && calculator.info">
            <calc-info :info="calculator.info" :dev="selectedCalculator!"/>
            <label class="inline-flex items-center cursor-pointer mr-2 mt-4">
              <input type="checkbox" class="h-5 w-5 text-accent cursor-pointer" v-model="showHidden">
              <span class="mx-2 text-muted select-none">Include hidden files</span>
            </label>
          </div>
        </div>
      </div>
      <div class="w-full">
        <div class="h-full">
          <file-browser v-if="calculator && calculator.info" :dev="selectedCalculator!" :show-hidden="showHidden"/>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-shell';
import CalcInfo from 'n-link-core/components/CalcInfo.vue';
import FileBrowser from 'n-link-core/components/FileBrowser.vue';
import DeviceSelect from 'n-link-core/components/DeviceSelect.vue';
import devices from '../components/devices';

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const selectedCalculator = ref<string | null>(null);
const showHidden = ref(false);

function onEnumerated() {
  const first = Object.keys(devices.devices)[0];
  if (first) selectedCalculator.value = first;
}

async function onDeviceChange() {
  if (!selectedCalculator.value) {
    await sleep(1000);
    if (selectedCalculator.value) return;
    const first = Object.keys(devices.devices)[0];
    if (first) selectedCalculator.value = first;
  } else if (!Object.keys(devices.devices).includes(selectedCalculator.value)) {
    selectedCalculator.value = null;
    // go back and choose the first if available
    onDeviceChange();
  }
}

async function onSelectCalculator(dev: string | null) {
  if (dev && !devices.devices[dev].info && !devices.devices[dev].needsDrivers) {
    try {
      await devices.open(dev);
    } catch (e) {
      console.error(e);
      selectedCalculator.value = null;
    }
  }
}

watch(() => devices.hasEnumerated, onEnumerated);
watch(() => devices.devices, onDeviceChange, { deep: true });
watch(selectedCalculator, onSelectCalculator);

const calculator = computed(() => selectedCalculator.value && devices.devices[selectedCalculator.value]);

const needsDrivers = computed(() => selectedCalculator.value && devices.devices[selectedCalculator.value]?.needsDrivers);

const isLinux = computed(() => navigator.platform.includes('Linux'));

function installDrivers(platform = 'windows') {
  open(`https://lights0123.com/n-link/#${platform}`);
}
</script>

<style lang="scss" scoped>
.lds-dual-ring {
  display: inline-block;
  width: 80px;
  height: 80px;
}

.lds-dual-ring:after {
  $color: var(--muted);
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
