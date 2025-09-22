import { defineConfig, PluginOption } from 'vite'
import gleam from 'vite-gleam'
import encrecss from 'encre-css-vite'

const BACKEND_ROOT_URL = 'http://localhost:3721'
const BASE_PATH = '/ladmin/'

export default defineConfig({
  plugins: [
    gleam() as PluginOption,
    encrecss({
      include: [
        /\.(html|gleam)($|\?)/,
        /ladmin/,
        'src/**/*.{ts,js}',
      ]
    })],
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
