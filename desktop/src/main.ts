import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import devices from './components/devices';
import { DEVICES_KEY } from 'n-link-core/components/devices';
import 'n-link-core/assets/tailwind.css';
import '@fontsource/inter/400.css';
import '@fontsource/inter/500.css';
import '@fontsource/inter/600.css';
import '@fontsource/inter/700.css';

const app = createApp(App);
app.use(router);
// Two registrations, both required:
//   globalProperties -> templates across n-link-core reference $devices
//   provide          -> <script setup> blocks have no `this`, so they inject
app.config.globalProperties.$devices = devices;
app.provide(DEVICES_KEY, devices);
app.mount('#app');
