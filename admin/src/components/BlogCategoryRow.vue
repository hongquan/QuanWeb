<template>
  <tr :class='classNames'>
    <th
      scope='row'
      class='px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white'
    >
      <RouterLink
        :to='editUrl'
        class='hover:underline'
      >
        {{ category.title }}
      </RouterLink>
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
import HStatus from 'http-status'
import { toast } from 'vue-sonner'

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

const editUrl = computed(() => ({
  name: 'category.edit',
  params: { categoryId: props.category.id },
}))

async function deleteCategory() {
  if (!props.category.id) {
    return
  }
  const url = lightJoin(API_GET_CATEGORIES, props.category.id)
  const resp = await kyClient.delete(url)
  if (resp.status !== HStatus.NO_CONTENT) {
    toast.error('Failed to delete category')
    return
  }
  toast.success(`Category ${props.category.title} is deleted!`)
  emit('deleted', props.category.id)
}
</script>
