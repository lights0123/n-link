<template>
  <main style="font-family: monospace; padding: 1rem">
    <button @click="run">Run backend check</button>
    <pre>{{ log }}</pre>
  </main>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const log = ref('');
const add = (line: string) => (log.value += line + '\n');

listen('addDevice', (e) => add('event addDevice: ' + JSON.stringify(e.payload)));
listen('removeDevice', (e) => add('event removeDevice: ' + JSON.stringify(e.payload)));
listen('progress', (e) => add('event progress: ' + JSON.stringify(e.payload)));

// libnspire transfer commands can hang indefinitely against the CX II's
// known firmware Busy issue. Give those calls a client-side timeout so a
// hang on one call never prevents the rest of the harness from reporting.
const TRANSFER_TIMEOUT_MS = 10_000;

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('TIMED_OUT')), ms);
    promise.then(
      (value) => {
        clearTimeout(timer);
        resolve(value);
      },
      (err) => {
        clearTimeout(timer);
        reject(err);
      },
    );
  });
}

// Runs one invoke call, reporting its outcome without ever throwing —
// a failure or timeout here must not stop the remaining steps from running.
async function step<T>(label: string, call: () => Promise<T>, timeoutMs?: number) {
  try {
    const result = timeoutMs ? await withTimeout(call(), timeoutMs) : await call();
    add(`${label} -> ` + JSON.stringify(result, null, 2));
    return { ok: true as const, result };
  } catch (e) {
    if (e instanceof Error && e.message === 'TIMED_OUT') {
      add(`${label} -> TIMED OUT (expected — known CX II firmware issue)`);
    } else {
      add(`${label} ERROR: ` + String(e));
    }
    return { ok: false as const, result: undefined };
  }
}

async function run() {
  log.value = '';

  // enumerate only reads USB descriptors, no device transfer — it never
  // hangs, so it gets no timeout and always reports before anything else runs.
  const enumerateResult = await step('enumerate', () => invoke<any[]>('enumerate'));
  if (!enumerateResult.ok) return;

  const devices = enumerateResult.result;
  if (!devices.length) {
    add('NO DEVICES FOUND');
    return;
  }

  const { busNumber, address } = devices[0];
  add(`opening ${busNumber}:${address}`);

  // The remaining commands perform real libnspire transfers and are the
  // ones known to hang or return Busy on this machine's CX II firmware.
  await step('open_device', () => invoke('open_device', { busNumber, address }), TRANSFER_TIMEOUT_MS);
  await step('list_dir /', () => invoke('list_dir', { busNumber, address, path: '/' }), TRANSFER_TIMEOUT_MS);
  await step('close_device', () => invoke('close_device', { busNumber, address }), TRANSFER_TIMEOUT_MS);
}
</script>
