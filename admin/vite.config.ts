import path from 'node:path'
import { defineConfig, ViteDevServer, Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import svgLoader from 'vite-svg-loader'
import lightJoin from 'light-join'

const BACKEND_ROOT_URL = 'http://localhost:3721'
const BACKEND_PREFIXES = ['/preview/']
const BASE_PATH = '/admin/'

const backendRedirectPlugin = {
  name: 'backend-redirect',
  configureServer(server: ViteDevServer) {
    const log = server.config.logger
    // Ref: https://github.com/senchalabs/connect#use-middleware
    server.middlewares.use((req, res, next) => {
      if (!req.url || !BACKEND_PREFIXES.some(item => req.url?.startsWith(item))) {
        return next()
      }
      const newUrl = lightJoin(BACKEND_ROOT_URL, req.url)
      log.info(`To redirect to ${newUrl}`)
      // Ref: https://github.com/thenativeweb/forcedomain/blob/main/lib/forceDomain.ts
      res.writeHead(302, {
        Location: newUrl,
      })
      res.end()
    })
  },
} as Plugin

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    svgLoader({ svgo: false }),
    backendRedirectPlugin,
  ],
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
    fs: {
      allow: ['..'],
    },
    proxy: {
      '/_api': {
        target: BACKEND_ROOT_URL,
      },
    },
  },
})
