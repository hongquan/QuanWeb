<template>
  <tr :class='classNames'>
    <th
      scope='row'
      class='px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white'
    >
      {{ category.title }}
    </th>
    <td class='px-6 py-4'>
      {{ category.slug }}
    </td>
    <td>
      <button
        class='hover:text-red-500'
        @click='deleteCategory'
      >
        <Icon
          icon='ic:outline-delete-forever'
          class='w-5 h-5'
        />
      </button>
    </td>
  </tr>
</template>

<script setup lang='ts'>
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import lightJoin from 'light-join'

import { Category } from '@/models/blog'
import { kyClient } from '@/common'
import { API_GET_CATEGORIES } from '@/urls'

interface Props {
  category: Category,
  isOdd?: boolean
}
const props = withDefaults(defineProps<Props>(), {
  isOdd: false,
})
const emit = defineEmits<{
  deleted: [id: string],
}>()

const classNames = computed(() => [
  props.isOdd ? 'bg-white dark:bg-gray-900' : 'bg-gray-50 dark:bg-gray-800',
  'border-b',
  'dark:border-gray-700',
])

async function deleteCategory() {
  const url = lightJoin(API_GET_CATEGORIES, props.category.id)
  await kyClient.delete(url)
  emit('deleted', props.category.id)
}
</script>
