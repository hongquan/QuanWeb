<template>
  <div class='max-w-lg mx-auto'>
    <form v-if='post' method='post'>
      <HorizontalFormField v-model='post.title' label='Title' />
      <HorizontalFormField v-model='post.slug' label='Slug' />
    </form>
  </div>
</template>

<script setup lang='ts'>
import { ref, onBeforeMount } from 'vue'
import { Input } from 'flowbite-vue'
import lightJoin from 'light-join'

import { kyClient } from '@/common'
import { Post, PostSchema } from '@/models/blog'
import { API_GET_POSTS } from '@/urls'
import HorizontalFormField from '@/components/HorizontalFormField.vue'

interface Props {
  postId: string
}
const props = defineProps<Props>()
const post = ref<Post | null>(null)

async function fetchData() {
  const url = lightJoin(API_GET_POSTS, props.postId)
  const resp = await kyClient.get(url).json()
  post.value = PostSchema.parse(resp)
}

onBeforeMount(fetchData)
</script>
