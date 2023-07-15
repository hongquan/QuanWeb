<template>
  <div class='sm:grid sm:grid-cols-4 sm:items-start sm:gap-2 sm:py-2'>
    <label
      :for='uid'
      class='block text-sm font-medium leading-6 dark:text-white sm:pt-2'
    ><slot name='label' /></label>
    <div class='mt-2 sm:col-span-3 sm:mt-0'>
      <slot :input-id='uid' />
    </div>
  </div>
</template>

<script setup lang='ts'>
import { nanoid } from 'nanoid'

const uid = nanoid()
</script>
