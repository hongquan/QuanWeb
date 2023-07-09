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
      width: {
        120: '30rem',
        160: '40rem',
        200: '50rem',
        ...defaultTheme.width,
      },
      minWidth: {
        8: '2rem',
        10: '2.5rem',
        20: '5rem',
        40: '10rem',
        60: '15rem',
        72: '18rem',
        80: '20rem',
        100: '25rem',
        120: '30rem',
        160: '40rem',
        ...defaultTheme.minWidth,
      },
      maxWidth: {
        4: '1rem',
        6: '1.5rem',
        8: '2rem',
        20: '5rem',
        120: '30rem',
        ...defaultTheme.maxWidth,
      },
      height: {
        120: '30rem',
        160: '40rem',
        ...defaultTheme.height,
      },
      minHeight: {
        20: '5rem',
        ...defaultTheme.minHeight,
      },
      screens: {
        'xxs': '358px',
        ...defaultTheme.screens,
      },
    },
  },
  plugins: [
    FlowbitePlugin,
  ],
} satisfies Config
