import { defineConfig, PluginOption } from 'vite'
import gleam from 'vite-gleam'
import UnoCSS from 'unocss/vite'

const BACKEND_ROOT_URL = 'http://localhost:3721'
const BASE_PATH = '/ladmin/'

export default defineConfig({
  plugins: [gleam() as PluginOption, UnoCSS()],
  // Our prod site will be at http://domain/ladmin/
  base: BASE_PATH,
  server: {
    fs: {
      allow: ['..'],
    },
    proxy: {
      '/_api': {
        target: BACKEND_ROOT_URL,
      },
    },
  }
})
