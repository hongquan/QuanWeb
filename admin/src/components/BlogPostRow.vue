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
    <td class='px-6 py-4'>
      {{ CategoriesDisplay }}
    </td>
    <td>
      <div class='flex items-center space-x-4'>
        <a
          v-if='post.is_published'
          class='text-green-600 hover:text-green-400'
          :href='publicUrl'
        >
          <Icon
            icon='heroicons:globe-asia-australia-solid'
            class='w-5 h-auto'
          />
        </a>
      </div>
    </td>
    <td class='text-right pr-4'>
      <div class='flex space-x-4'>
        <a
          :href='previewUrl'
          class='hover:text-blue-600'
          title='Preview'
        ><Icon
          icon='mdi:view-in-ar'
          class='w-5 h-auto'
        /></a>
        <button
          class='hover:text-red-500'
          @click='deletePost'
        >
          <Icon
            icon='ic:outline-delete-forever'
            class='w-5 h-5'
          />
        </button>
      </div>
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
const previewUrl = computed(() => props.post.id ? `/preview/${props.post.id}` : '#')
const publicUrl = computed(() => {
  const createdAt = props.post.created_at as Date | null
  const y = createdAt ? createdAt.getFullYear() : 'y'
  const m = createdAt? createdAt.getMonth() : 'm'
  return props.post.id ? `/post/${y}/${m}/${props.post.slug}` : '#'
})

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
