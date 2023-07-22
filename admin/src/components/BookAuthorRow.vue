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
        {{ author.name }}
      </RouterLink>
    </th>
    <td>
      <button
        class='hover:text-red-500'
        @click='deleteAuthor'
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

import { BookAuthor } from '@/models/minors'
import { API_GET_BOOK_AUTHORS } from '@/urls'
import { kyClient } from '@/common'

interface Props {
  author: BookAuthor
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
  name: 'book-author.edit',
  params: {
    id: props.author.id,
  },
}))

async function deleteAuthor() {
  if (!props.author.id) {
    return
  }
  const url = lightJoin(API_GET_BOOK_AUTHORS, props.author.id)
  const resp = await kyClient.delete(url)
  if (resp.status !== HStatus.NO_CONTENT) {
    toast.error('Failed to delete the presentation')
    return
  }
  toast.success(`Author "${props.author.name}" is deleted.`)
  emit('deleted', props.author.id)
}
</script>
