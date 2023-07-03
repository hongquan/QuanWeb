<template>
  <div class='max-w-lg mx-auto'>
    <form v-if='post' method='post' @submit.prevent='onSubmit'>
      <HorizontalFormField v-model='post.title' label='Title' />
      <HorizontalFormField v-model='post.slug' label='Slug' />
      <DualPaneSelect label='Categories' :all-options='allCategories' :selected-options='post.categories'
        @taken='onCategoryTaken($event)' @released='onCategoryReleased($event)' />
      <div class='text-center mt-2'>
        <FbButton type='submit' size='sm' :loading='isSubmitting'>Submit</FbButton>
      </div>
    </form>
  </div>
</template>

<script setup lang='ts'>
import { ref, onBeforeMount, onMounted, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import lightJoin from 'light-join'
import { slugify } from 'transliteration'
import { Button as FbButton } from 'flowbite-vue'
import { toast } from 'vue-sonner'
import { z } from 'zod'
import { A } from '@mobily/ts-belt'

import { kyClient } from '@/common'
import { Category, CategorySchema, Post, PostSchema } from '@/models/blog'
import { API_GET_CATEGORIES, API_GET_POSTS } from '@/urls'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'
import DualPaneSelect from '@/components/forms/DualPaneSelect.vue'
import { transformPostForPosting } from '@/utils/models'
import { ObjectListResponseSchema } from '@/models/api'

interface Props {
  postId: string
}
const props = defineProps<Props>()
const router = useRouter()
const post = ref<Post | null>(null)
const allCategories = ref<Category[]>([])
const isSubmitting = ref(false)

async function fetchCategories() {
  const raw = await kyClient.get(API_GET_CATEGORIES).json()
  const resp = ObjectListResponseSchema.parse(raw)
  allCategories.value = z.array(CategorySchema).parse(resp.objects)
}

async function fetchData() {
  const url = lightJoin(API_GET_POSTS, props.postId)
  const resp = await kyClient.get(url).json()
  post.value = PostSchema.parse(resp)
  await fetchCategories()
}

function onCategoryTaken(id: string) {
  let cat = A.getBy(allCategories.value, c => c.id === id)
  if (cat && post.value) {
    post.value.categories.push(cat)
  }

}

function onCategoryReleased(id: string) {
  if (!post.value) {
    return
  }
  post.value.categories = post.value.categories.filter(c => c.id !== id)
}

async function onSubmit() {
  if (!post.value) {
    return
  }
  const url = lightJoin(API_GET_POSTS, props.postId)
  const postData = transformPostForPosting(post.value)
  const resp = await kyClient.patch(url, {
    json: postData,
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
