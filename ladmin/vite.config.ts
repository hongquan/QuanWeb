import { defineConfig, PluginOption } from 'vite'
import gleam from 'vite-gleam'
// import encrecss from 'encre-css-vite'
// import UnoCSS from 'unocss/vite'

const runEncreCss = (): PluginOption => ({
  name: 'run-encre-css',
  buildStart: async () => {
    console.log(`[run-encre-css] Running 'encrecss' for build...`);
    const proc = Bun.spawn(['encrecss', 'build', '-o', 'generated-encre.css']);
    const output = await proc.exited;
    if (output !== 0) {
      console.error(`[run-encre-css] Error: Process exited with code ${output}`);
    } else {
      console.log(`[run-encre-css] Successfully generated CSS`);
    }
  },
  handleHotUpdate: async ({ file }) => {
    console.log(`[run-encre-css] File changed: ${file}. Running 'encrecss'...`);
    const proc = Bun.spawn(['encrecss', 'build', '-o', 'generated-encre.css']);
    const output = await proc.exited;
    if (output !== 0) {
      console.error(`[run-encre-css] Error: Process exited with code ${output}`);
    } else {
      console.log(`[run-encre-css] Successfully generated CSS`);
    }
  },
});

const BACKEND_ROOT_URL = 'http://localhost:3721'
const BASE_PATH = '/ladmin/'

export default defineConfig({
  plugins: [
    gleam() as PluginOption,
    // UnoCSS(),
    // encrecss({
    //   include: [
    //     // /build\/dev\/javascript\/ladmin\/.+\.mjs/,
    //     /ladmin/,
    //   ]
    // }),
    runEncreCss()
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
    },
  }
})
