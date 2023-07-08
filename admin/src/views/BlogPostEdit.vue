<template>
  <div class='max-w-lg mx-auto'>
    <form
      v-if='post'
      method='post'
      @submit.prevent='onSubmit'
    >
      <HorizontalFormField
        v-model='post.title'
        label='Title'
      />
      <HorizontalFormField
        v-model='post.slug'
        label='Slug'
      />
      <DualPaneSelect
        label='Categories'
        :all-options='allCategories'
        :selected-options='post.categories'
        @taken='onCategoryTaken($event)'
        @released='onCategoryReleased($event)'
      />
      <div
        ref='codeEditor'
        class='language-markdown w-full mt-4 p-4 font-mono rounded'
      />
      <div class='text-center mt-2'>
        <FbButton
          type='submit'
          :loading='isSubmitting'
        >
          Save
        </FbButton>
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
import { z } from 'zod'
import { A } from '@mobily/ts-belt'
import { CodeJar } from 'codejar'
import hljs from 'highlight.js/lib/core'
import markdown from 'highlight.js/lib/languages/markdown'
import 'highlight.js/styles/base16/zenburn.css'

import { kyClient } from '@/common'
import { Category, CategorySchema, Post, PostSchema } from '@/models/blog'
import { API_GET_CATEGORIES, API_GET_POSTS } from '@/urls'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'
import DualPaneSelect from '@/components/forms/DualPaneSelect.vue'
import { transformPostForPosting } from '@/utils/models'
import { ObjectListResponseSchema } from '@/models/api'

interface Props {
  postId?: string | null
}
const props = withDefaults(defineProps<Props>(), {
  postId: null,
})
hljs.registerLanguage('markdown', markdown)

const router = useRouter()
const post = ref<Post | null>(null)
const allCategories = ref<Category[]>([])
const isSubmitting = ref(false)
const codeEditor = ref<HTMLDivElement | null>(null)
const jar = ref<CodeJar | null>(null)

async function fetchCategories() {
  const raw = await kyClient.get(API_GET_CATEGORIES).json()
  const resp = ObjectListResponseSchema.parse(raw)
  allCategories.value = z.array(CategorySchema).parse(resp.objects)
}

async function fetchData() {
  await fetchCategories()
  if (!props.postId) {
    post.value = PostSchema.parse({})
    return
  }
  const url = lightJoin(API_GET_POSTS, props.postId)
  const resp = await kyClient.get(url).json()
  post.value = PostSchema.parse(resp)
  if (jar.value && post.value.body) {
    jar.value.updateCode(post.value.body)
  }
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
  const isCreating = !props.postId
  const url = props.postId ? lightJoin(API_GET_POSTS, props.postId) : API_GET_POSTS
  const postData = transformPostForPosting(post.value)
  isSubmitting.value = true
  try {
    const resp = await kyClient(url, {
      method: isCreating ? 'post' : 'patch',
      json: postData,
    }).json()
    const updatedPost = PostSchema.parse(resp)
    const message = isCreating ? `Post "${updatedPost.title}" is created!` : `Post "${updatedPost.title}" is updated!`
    toast.success(message)
    await router.push({ name: 'post.list' })
  } catch (e) {
    console.debug(e)
    toast.error('Failed to save post!')
  } finally {
    isSubmitting.value = false
  }
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
    { flush: 'post' },
  )
  watch(codeEditor, (el) => {
    if (el) {
      jar.value = CodeJar(el, hljs.highlightElement)
      if (post.value?.body) {
        jar.value.updateCode(post.value.body)
      }
      jar.value.onUpdate(code => {
        if (post.value) {
          post.value.body = code
        }
      })
    }
  })
})
</script>
