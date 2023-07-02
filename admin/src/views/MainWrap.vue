<template>
  <div class='mx-auto flex flex-col h-full px-4'>
    <HeaderBar
      :locale='selectedLocale'
      @locale-selected='preferLocale($event)'
    />
    <NavTabBar class='mb-4' />
    <main class='mx-auto grow sm:w-full'>
      <RouterView />
      <hr class='my-4'>
    </main>
    <footer class='text-sm p-4 border-t border-gray-200 dark:border-gray-600 flex justify-between'>
    </footer>
  </div>
</template>

<script setup lang='ts'>
import { onBeforeMount, ref } from 'vue'
import { getUserLocale } from 'get-user-locale'

import { Language } from '@/models'
import HeaderBar from '@/components/HeaderBar.vue'
import NavTabBar from '@/components/NavTabBar.vue'
import { preferLocale } from '@/translation'

const selectedLocale = ref(Language.EN)

onBeforeMount(() => {
  const userLocale = getUserLocale()
  if (userLocale && userLocale.startsWith(Language.VI)) {
    selectedLocale.value = Language.VI
    preferLocale(Language.VI)
  }
})
</script>
