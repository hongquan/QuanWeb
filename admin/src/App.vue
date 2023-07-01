<template>
  <Toaster
    rich-colors
    position='top-right'
    :toast-options='toastOptions'
  />
  <RouterView />
</template>

<script setup lang='ts'>
import { Toaster } from 'vue-sonner'
import type { ToastOptions } from 'vue-sonner/lib/types'

const toastOptions: ToastOptions = {
  className: 'fixed',
}

</script>
