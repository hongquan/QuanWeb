import { defineConfig, presetWind4 } from 'unocss'

export default defineConfig({
  content: {
    pipeline: {
      include: [
        /\.(html|gleam)($|\?)/,
        /ladmin/,
        'src/**/*.{ts,js}',
      ]
    }
  },
  presets: [presetWind4({ dark: 'media' })]
})
