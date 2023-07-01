import type { Config } from 'tailwindcss'
// import FlowbitePlugin from 'flowbite/plugin'

export default {
  content: ['./index.html', './src/**/*.vue'],
  theme: {
    extend: {},
  },
  plugins: [
    // FlowbitePlugin,
  ],
} satisfies Config
