<template>
  <RouterLink
    v-slot='{ isActive, href, navigate }'
    :to='toPage'
    custom
  >
    <li class='mr-2'>
      <a
        :href='href'
        class='inline-block p-4 rounded-t-lg border-b-2'
        :class='isActive ? ACTIVE_CLASS : INACTIVE_CLASS'
        :aria-current='getAriaCurrent(isActive)'
        @click='navigate'
      >
        <slot />
      </a>
    </li>
  </RouterLink>
</template>

<script setup lang='ts'>
import { computed } from 'vue'
interface Props {
  routeName: string,
}
const ACTIVE_CLASS = 'active text-blue-600 border-blue-600 dark:text-blue-500 dark:border-blue-500'
const INACTIVE_CLASS = 'border-transparent hover:text-gray-600 hover:border-gray-300 dark:hover:text-gray-300'
const props = defineProps<Props>()
const toPage = computed(() => ({ name: props.routeName }))

function getAriaCurrent(isActive: boolean) {
  return isActive ? 'page' : undefined
}
</script>
