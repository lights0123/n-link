import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';
import { fileURLToPath, URL } from 'node:url';

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      'n-link-core': fileURLToPath(new URL('../n-link-core', import.meta.url)),
    },
  },
  test: {
    environment: 'happy-dom',
    include: ['../n-link-core/components/**/*.spec.ts', 'src/**/*.spec.ts'],
  },
});
