<template>
  <client-only>
    <vue-final-modal
      v-model="isOpen"
      classes="flex justify-center h-screen overflow-auto"
      content-class="mt-10 mb-8 xl:mx-10 w-full h-min bg-white rounded p-4"
    >
      <div class="flex flex-col bg-white rounded p-4">
        <button v-if="isOpen" class="self-end" @click="isOpen = false">
          <img src="~feather-icons/dist/icons/x.svg" class="w-5" />
        </button>
        <div v-if="category === 'udev'">
          <h1 class="text-3xl">Permission Error</h1>
          <p>
            A permissions error was encountered when trying to access the
            device.
            <a
              href="https://lights0123.com/n-link/#linux"
              class="text-blue-600 underline"
            >
              Follow the Linux installation steps</a
            >
            to configure udev rules.
          </p>
        </div>
        <div v-else-if="category === 'reset'">
          <h1 class="text-3xl">Resetting Error</h1>
          <p>
            Unfortunately, you've run into a known bug affecting calculators on
            Windows and macOS. There's currently a bug in Chrome that prevents
            this from working. Feel free to grab the
            <a
              href="https://lights0123.com/n-link/"
              class="text-blue-600 underline"
            >
              desktop version</a
            >
            instead, though.
          </p>
        </div>
        <div v-else>
          <h1 class="text-3xl">Unknown Error</h1>
          <p>
            An unknown error occurred with the message:
            {{ error && error.message }}
          </p>
        </div>
      </div>
    </vue-final-modal>
  </client-only>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';

@Component
export default class ErrorMessage extends Vue {
  isOpen = false;
  category = '';
  error: DOMException | null = null;

  handleError(error: DOMException, phase: 'connection' | 'operation') {
    if (
      phase === 'connection' &&
      error.name === 'SecurityError' &&
      navigator.platform.includes('Linux')
    ) {
      this.category = 'udev';
    } else if (
      phase === 'connection' &&
      error.message === 'Unable to reset the device.' &&
      ['Win', 'Mac'].some((platform) => navigator.platform.includes(platform))
    ) {
      this.category = 'reset';
    } else {
      this.category = '';
    }
    this.isOpen = true;
    this.error = error;
  }
}
</script>
