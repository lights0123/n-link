import Vue from 'vue'
import App from './App.vue'
import AsyncComputed from 'vue-async-computed';
import router from './router'
import 'n-link-core/assets/tailwind.css'
import './components/devices';
Vue.use(AsyncComputed);
Vue.config.productionTip = false

new Vue({
  router,
  render: h => h(App)
}).$mount('#app')
