import path from 'node:path'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import svgLoader from 'vite-svg-loader'

const BACKEND_ROOT_URL = 'http://localhost:3721'
const BASE_PATH = '/admin/'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue(), svgLoader({ svgo: false })],
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
    },
  },
})
