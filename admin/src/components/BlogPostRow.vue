<template>
  <tr :class='classNames'>
    <th scope='row' class='px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white'>
      {{ post.title }}
    </th>
    <td class='px-6 py-4'>
      {{ post.slug }}
    </td>
  </tr>
</template>

<script setup lang='ts'>
import { computed } from 'vue'

import { Post } from '@/models/blog'

interface Props {
  post: Post,
  isOdd?: boolean
}
const props = withDefaults(defineProps<Props>(), {
  isOdd: false
})

const classNames = computed(() => [
  props.isOdd ? 'bg-white dark:bg-gray-900' : 'bg-gray-50 dark:bg-gray-800',
  'border-b',
  'dark:border-gray-700'
])
</script>
