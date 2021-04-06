import Vue from 'vue';
import AsyncComputed from 'vue-async-computed';
import VueFinalModal from 'vue-final-modal';

if (process.client) Vue.use(VueFinalModal());
Vue.use(AsyncComputed);
