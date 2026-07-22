import { createRouter, createWebHashHistory } from 'vue-router';
import Home from '../views/Home.vue';

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'Home', component: Home },
    { path: '/about', name: 'About', component: () => import('../views/About.vue') },
  ],
});
