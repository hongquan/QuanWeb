<template>
  <header
    class='bg-white border-b border-gray-200 dark:border-gray-600 dark:bg-gray-800 mb-4 sm:flex item-center justify-between'>
    <h1 class='text-2xl text-slate-900 dark:text-white p-4'>
      Admin
    </h1>
    <div class='py-4 text-right'>
      <select
        class='bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded focus:ring-blue-500 focus:border-blue-500 px-2 py-1 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500'
        @change='onLocaleSwitch'>
        <option v-for='(label, code) of LANGUAGES' :key='code' :value='code' :selected='code === locale'>
          {{ label }}
        </option>
      </select>
    </div>
  </header>
</template>

<script setup lang='ts'>
import { Language } from '@/models'

interface Props {
  locale?: Language
}

interface Emits {
  (e: 'localeSelected', payload: Language): void,
}

withDefaults(defineProps<Props>(), {
  locale: Language.EN,
})

const emit = defineEmits<Emits>()

const LANGUAGES = {
  [Language.EN]: 'English',
  [Language.VI]: 'Tiếng Việt',
}

function onLocaleSwitch(ev: Event) {
  const value = (ev.target as HTMLSelectElement).value as Language
  emit('localeSelected', value)
}
</script>
