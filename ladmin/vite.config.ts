import { defineConfig, PluginOption } from 'vite'
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
        // /build\/dev\/javascript\/ladmin\/.+\.mjs/,
        /ladmin/,
      ]
    })
  ],
  // Our prod site will be at http://domain/ladmin/
  base: BASE_PATH,
  server: {
    port: 5111,
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
