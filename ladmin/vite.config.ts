import { defineConfig, type PluginOption } from 'vite'
import gleam from 'vite-gleam'
import encrecss from 'encre-css-vite'
// import UnoCSS from 'unocss/vite'

const BACKEND_ROOT_URL = 'http://localhost:3721'
const BASE_PATH = '/ladmin/'

export default defineConfig({
  plugins: [
    gleam() as PluginOption,
    // UnoCSS(),
    encrecss({
      include: [
        /\.(ts|html|gleam)($|\?)/,
      ]
    }),
  ],
  // Our prod site will be at http://domain/ladmin/
  base: BASE_PATH,
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules')) {
            return 'vendor'
          }
          if (id.includes('/gleam')) {
            return 'gleam'
          }
          return null
        }
      }
    }
  },
  server: {
    port: 5111,
    fs: {
      allow: ['..'],
    },
    proxy: {
      '/_api': {
        target: BACKEND_ROOT_URL,
      },
      '/static/': {
        target: BACKEND_ROOT_URL,
      },
    },
  }
})
