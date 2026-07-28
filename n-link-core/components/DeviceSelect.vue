<template>
  <div class="header border-b border-edge px-2 py-2 flex w-full bg-raised">
    <button @click="$devices.enumerate()" class="flex-shrink-0 mr-2 focus:outline-none text-fg"
            :class="$devices.enumerating && 'cursor-not-allowed opacity-25'" :disabled="$devices.enumerating">
      <icon name="refresh-cw" :size="20"/>
      <div v-if="scanHint && Object.keys($devices.devices).length === 0" class="p-4 refresh-popup">
        Click to connect a device
      </div>
    </button>
    <popover v-model:visible="active" :width="239" class="w-full">
      <template #trigger>
        <div class="relative w-full focus:outline-none">
          <div class="block w-full bg-surface border border-edge hover:border-muted px-4 py-3/2 pr-8
                      rounded leading-tight focus:outline-none h-8 truncate text-fg">
            <span v-if="selectedCalculator">
              <small class="tabular-nums">{{ selectedCalculator }}</small>
              <span v-if="calc && calc.info"> {{ calc.info.name }}</span>
              <span v-else> {{ calc!.name }}</span>
            </span>
            <span v-else class="text-muted text-sm">
              Select a device...
            </span>
          </div>
          <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-muted">
            <svg class="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
              <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z"/>
            </svg>
          </div>
        </div>
      </template>
      <ul class="-m-3">
        <li v-for="(device, id) in $devices.devices" :key="id"
            class="p-2 hover:bg-accent hover:text-accent-fg w-full cursor-pointer text-fg" @click="select(id)">
          <small class="tabular-nums">{{ id }}</small>
          <span v-if="device.info"> {{ device.info.name }}</span>
          <span v-else> {{ device.name }}</span>
        </li>
        <div v-if="!anyCalcs" class="p-2 w-full text-muted">
          No calculators found
        </div>
      </ul>
    </popover>
  </div>
</template>

<script setup lang="ts">
import {computed, inject, ref} from 'vue';
import {DEVICES_KEY, type GenericDevices} from './devices';
import Popover from './Popover.vue';
import Icon from './Icon.vue';

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
  @apply absolute bg-accent text-accent-fg;
  &:before {
    content: "";
    position: absolute;
    left: $offset;
    top: $size * -2;
    border-top: $size solid transparent;
    border-right: $size solid transparent;
    border-bottom: $size solid var(--accent);
    border-left: $size solid transparent;
  }
}
</style>
