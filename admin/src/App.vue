<template>
  <Toaster
    rich-colors
    position='top-right'
    close-button
    :toast-options='toastOptions'
  />
  <RouterView />
</template>

<script setup lang='ts'>
import { Toaster } from 'vue-sonner'

const toastOptions = {
  class: 'fixed',
}

</script>
