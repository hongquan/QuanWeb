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
        {{ book.title }}
      </RouterLink>
    </th>
    <td class='px-6 py-4'>
      {{ book.author?.name || '--' }}
    </td>
    <td class='px-6 py-4'>
      <a
        v-if='book.download_url'
        class='truncate hover:underline'
        :href='book.download_url'
      >{{ book.download_url }}</a>
    </td>
    <td>
      <button
        class='hover:text-red-500'
        @click='deleteBook'
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
import lightJoin from 'light-join'
import httpStatus from 'http-status'
import { Icon } from '@iconify/vue'
import { toast } from 'vue-sonner'

import { Book } from '@/models/minors'
import { API_GET_BOOK_AUTHORS } from '@/urls'
import { kyClient } from '@/common'

interface Props {
  book: Book
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
  name: 'book.edit',
  params: {
    id: props.book.id,
  },
}))

async function deleteBook() {
  if (!props.book.id) {
    return
  }
  const url = lightJoin(API_GET_BOOK_AUTHORS, props.book.id)
  const resp = await kyClient.delete(url)
  if (resp.status !== httpStatus.NO_CONTENT) {
    toast.error('Failed to delete the presentation')
    return
  }
  toast.success(`Book "${props.book.title}" is deleted.`)
  emit('deleted', props.book.id)
}

</script>
