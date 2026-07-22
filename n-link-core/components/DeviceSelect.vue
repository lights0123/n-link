<template>
  <div class="header border-b px-2 py-2 flex w-full">
    <button @click="$devices.enumerate()" class="flex-shrink-0 mr-2 focus:outline-none"
            :class="$devices.enumerating && 'cursor-not-allowed opacity-25'" :disabled="$devices.enumerating">
      <img :src="refreshCw" class="w-5"/>
      <div v-if="scanHint && Object.keys($devices.devices).length === 0" class="p-4 refresh-popup">
        Click to connect a device
      </div>
    </button>
    <el-popover width="239" :show-arrow="false" popper-class="focus:outline-none dev-select-pop" v-model:visible="active"
                class="w-full overflow-hidden">
      <template #reference>
        <div class="relative w-full focus:outline-none">
          <div
              class="block w-full bg-white border border-gray-400 hover:border-gray-500 px-4 py-3/2 pr-8 rounded shadow leading-tight focus:outline-none focus:shadow-outline h-8 truncate">
            <span v-if="selectedCalculator">
              <small class="tabular-nums">{{ selectedCalculator }}</small>
              <span v-if="calc && calc.info"> {{ calc.info.name }}</span>
              <span v-else> {{ calc!.name }}</span>
            </span>
            <span v-else class="text-gray-700 text-sm">
              Select a device...
            </span>
          </div>
          <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700">
            <svg class="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
              <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z"/>
            </svg>
          </div>
        </div>
      </template>
      <ul>
        <li v-for="(device, id) in $devices.devices" :key="id"
            class="p-2 hover:bg-blue-500 hover:text-white w-full cursor-pointer" @click="select(id)">
          <small class="tabular-nums">{{ id }}</small>
          <span v-if="device.info"> {{ device.info.name }}</span>
          <span v-else> {{ device.name }}</span>
        </li>
        <div v-if="!anyCalcs" class="p-2 w-full">
          No calculators found
        </div>
      </ul>
    </el-popover>
  </div>
</template>

<script setup lang="ts">
import {computed, inject, ref} from 'vue';
import {ElPopover} from 'element-plus';
import 'element-plus/es/components/popover/style/css';
import {DEVICES_KEY, type GenericDevices} from './devices';
import refreshCw from 'feather-icons/dist/icons/refresh-cw.svg';

withDefaults(defineProps<{
  scanHint?: boolean;
}>(), {
  scanHint: false,
});

const selectedCalculator = defineModel<string | null>('selected');

const devices = inject(DEVICES_KEY) as GenericDevices;

const active = ref(false);

function select(dev: string) {
  selectedCalculator.value = dev;
  active.value = false;
}

const calc = computed(() => {
  const id = selectedCalculator.value;
  return id ? devices.devices[id] : undefined;
});

const anyCalcs = computed(() => !!Object.keys(devices.devices).length);
</script>

<style scoped lang="scss">
.header {
  height: 50px;
}

.refresh-popup {
  $offset: 5px;
  $size: 9px;
  margin-left: -4.5px;
  margin-top: 10px;
  @apply absolute bg-blue-600 text-white;
  &:before {
    content: "";
    position: absolute;
    left: $offset;
    top: $size * -2;
    border-top: $size solid transparent;
    border-right: $size solid transparent;
    border-bottom: $size solid theme('colors.blue.600');

    border-left: $size solid transparent;
  }
}
</style>

<style lang="scss">
.dev-select-pop {
  margin-top: 0 !important;
  @apply p-0 overflow-hidden;
}
</style>
