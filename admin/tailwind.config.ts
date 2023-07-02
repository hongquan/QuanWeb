import type { Config } from 'tailwindcss'
import defaultTheme from 'tailwindcss/defaultTheme'
import FlowbitePlugin from 'flowbite/plugin'

defaultTheme.fontFamily;

export default {
  content: [
    './index.html',
    './src/**/*.vue',
    'node_modules/flowbite-vue/**/*.{js,jsx,ts,tsx}',
    'node_modules/flowbite/**/*.{js,jsx,ts,tsx}',
  ],
  darkMode: 'media',
  theme: {
    extend: {
      fontFamily: {
        sans: ['Niramit', ...defaultTheme.fontFamily.sans],
      },
      minWidth: {
        8: '2rem',
        ...defaultTheme.minWidth,
      }
    },
  },
  plugins: [
    FlowbitePlugin,
  ],
} satisfies Config
