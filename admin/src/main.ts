import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import '@fontsource/niramit/vietnamese.css'

import './style.pcss'
import App from './App.vue'
import { routes } from './routes'

const BASE_PATH = import.meta.env.BASE_URL
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

const app = createApp(App)
app.use(pinia)

const router = createRouter({
  // Our prod site will be at http://domain/p/
  history: createWebHistory(BASE_PATH),
  scrollBehavior() {
    document.getElementById('app')?.scrollIntoView()
  },
  routes,
})
app.use(router)
app.mount('#app')
