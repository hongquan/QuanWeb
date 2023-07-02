<template>
  <div class='max-w-lg mx-auto'>
    <form v-if='post' method='post' @submit.prevent='onSubmit'>
      <HorizontalFormField v-model='post.title' label='Title' />
      <HorizontalFormField v-model='post.slug' label='Slug' />
      <div class='text-center mt-2'>
        <FbButton type='submit' size='sm' :loading='isSubmitting'>Submit</FbButton>
      </div>
    </form>
  </div>
</template>

<script setup lang='ts'>
import { ref, onBeforeMount, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import lightJoin from 'light-join'
import { slugify } from 'transliteration'
import { Button as FbButton } from 'flowbite-vue'
import { toast } from 'vue-sonner'

import { kyClient } from '@/common'
import { Post, PostSchema } from '@/models/blog'
import { API_GET_POSTS } from '@/urls'
import HorizontalFormField from '@/components/HorizontalFormField.vue'

interface Props {
  postId: string
}
const props = defineProps<Props>()
const router = useRouter()
const post = ref<Post | null>(null)
const isSubmitting = ref(false)

async function fetchData() {
  const url = lightJoin(API_GET_POSTS, props.postId)
  const resp = await kyClient.get(url).json()
  post.value = PostSchema.parse(resp)
}

async function onSubmit() {
  const url = lightJoin(API_GET_POSTS, props.postId)
  const resp = await kyClient.patch(url, {
    json: post.value,
  }).json()
  isSubmitting.value = false
  const updatedPost = PostSchema.parse(resp)
  toast.success(`Post "${updatedPost.title}" is updated!`)
  router.push({ name: 'post.list' })
}

onBeforeMount(fetchData)
onMounted(() => {
  watch(
    () => post.value?.title,
    (title) => {
      if (title) {
        post.value!.slug = slugify(title)
      }
    },
    { flush: 'post' }
  )
})
</script>
