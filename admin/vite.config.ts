import path from 'node:path'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

const BACKEND_ROOT_URL = 'http://localhost:3721'
const BASE_PATH = '/admin/'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, '/src'),
    },
  },
  // Our prod site will be at http://domain/admin/
  base: BASE_PATH,
  optimizeDeps: {
    exclude: ['vue-demi'],
  },
  server: {
    proxy: {
      '/_api': {
        target: BACKEND_ROOT_URL,
      },
    }
  }
})
