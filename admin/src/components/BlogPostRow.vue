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
        {{ post.title }}
      </RouterLink>
    </th>
    <td class='px-6 py-4'>
      {{ post.slug }}
    </td>
    <td class='px-6 py-4'>{{ CategoriesDisplay }}</td>
    <td>
      <button
        class='hover:text-red-500'
        @click='deletePost'
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
import { toast } from 'vue-sonner'
import HStatus from 'http-status'
import lightJoin from 'light-join'

import { Post } from '@/models/blog'
import { API_GET_POSTS } from '@/urls'
import { kyClient } from '@/common'

interface Props {
  post: Post,
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

const CategoriesDisplay = computed(() => {
  if (!props.post.categories) {
    return ''
  }
  return props.post.categories.map(c => c.title).join(', ')
})

const editUrl = computed(() => ({
  name: 'post.edit',
  params: { postId: props.post.id },
}))

async function deletePost() {
  if (!props.post.id) {
    return
  }
  const url = lightJoin(API_GET_POSTS, props.post.id)
  let resp = await kyClient.delete(url)
  if (resp.status !== HStatus.NO_CONTENT) {
    toast.error('Failed to delete post')
    return
  }
  toast.success(`Post ${props.post.title} is deleted!`)
  emit('deleted', props.post.id)
}
</script>
