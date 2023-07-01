import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import '@fontsource/niramit/vietnamese.css'

import './style.pcss'
import App from './App.vue'
import { routes } from './routes'

const BASE_PATH = import.meta.env.BASE_URL
const app = createApp(App)

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
