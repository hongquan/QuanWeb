<template>
  <div class='max-w-3xl mx-auto'>
    <form
      v-if='post'
      method='post'
      @submit.prevent='onSubmit'
    >
      <HorizontalFormField
        v-model='post.title'
        label='Title'
        :error-message='getValidationError("title")'
      />
      <HorizontalFormFieldWrap>
        <template #label>
          Slug
        </template>
        <template #default>
          <FbInput
            v-model='post.slug'
            size='sm'
          >
            <template
              v-if='oldSlug'
              #suffix
            >
              <FbButton
                type='button'
                pill
                outline
                size='xs'
                color='dark'
                class='absolute -bottom-0.5 right-0'
                @click='regenerateSlug'
              >
                <Icon
                  icon='mingcute:refresh-2-line'
                  class='h-3 w-auto'
                />
              </FbButton>
            </template>
          </FbInput>
        </template>
      </HorizontalFormFieldWrap>
      <DualPaneSelect
        class='mt-2'
        label='Categories'
        :all-options='allCategories'
        :selected-options='post.categories'
        @taken='onCategoryTaken($event)'
        @released='onCategoryReleased($event)'
      />
      <div class='mt-2 space-y-2'>
        <div class='flex justify-between'>
          <label class='block text-sm font-medium leading-6 dark:text-white sm:pt-2'>Body</label>
          <FbButton
            type='button'
            size='xs'
            outline
            color='dark'
            @click='getHtmlPreview'
          >
            Preview
          </FbButton>
        </div>
        <div class='border rounded font-mono py-4'>
          <div class='px-2'>
            <div
              ref='codeEditor'
              class='language-markdown relative rounded p-2 dark:text-gray-200 text-sm h-80'
            />
          </div>
        </div>
      </div>
      <HorizontalFormFieldWrap class='mt-2'>
        <template #label>
          Locale
        </template>
        <template #default='{ inputId }'>
          <FbSelect
            :id='inputId'
            v-model='postLocale'
            :options='locales'
          />
        </template>
      </HorizontalFormFieldWrap>
      <HorizontalFormFieldWrap class='mt-2'>
        <template #label>
          Author
        </template>
        <template #default='{ inputId }'>
          <FbSelect
            :id='inputId'
            v-model='postAuthorId'
            :options='allAuthorChoices'
            :option-label='(author: User) => author.email'
            :option-value='(author: User) => author.id'
          />
        </template>
      </HorizontalFormFieldWrap>
      <HorizontalFormField
        v-model='post.is_published'
        class='mt-2'
        widget-type='checkbox'
        label='Published'
      />
      <HorizontalFormField
        v-model='post.og_image'
        class='mt-2'
        widget-type='url'
        label='OpenGraph image'
        :error-message='getValidationError("og_image")'
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
    <FbModal
      v-if='previewHtml'
      :persistent='false'
      @close='previewHtml = null'
    >
      <template #body>
        <div
          class='overflow-y-auto max-h-96 text-sm html-preview'
          v-html='previewHtml'
        />
      </template>
    </FbModal>
  </div>
</template>

<script setup lang='ts'>
import { ref, onBeforeMount, onMounted, watch, onBeforeUnmount, computed } from 'vue'
import { useRouter } from 'vue-router'
import lightJoin from 'light-join'
import { slugify } from 'transliteration'
import { Button as FbButton, Select as FbSelect, Modal as FbModal, Input as FbInput } from 'flowbite-vue'
import { toast } from 'vue-sonner'
import { z } from 'zod'
import { A, F } from '@mobily/ts-belt'
import { Icon } from '@iconify/vue'
import type { KullnaEditor } from '@kullna/editor'
import { createEditor } from '@kullna/editor'
import Prism from 'prismjs'
import 'prismjs/themes/prism-dark.css'

import { kyClient } from '@/common'
import { Category, CategorySchema, Post, PostSchema } from '@/models/blog'
import { API_GET_CATEGORIES, API_GET_POSTS, API_GET_USERS, API_MARKDOWN_TO_HTML } from '@/urls'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'
import HorizontalFormFieldWrap from '@/components/forms/HorizontalFormFieldWrap.vue'
import DualPaneSelect from '@/components/forms/DualPaneSelect.vue'
import { transformPostForPosting } from '@/utils/models'
import { handleApiError } from '@/utils/api'
import { ObjectListResponseSchema } from '@/models/api'
import '../../../static/css/syntect.css'
import { User, UserSchema } from '@/models/user'

interface Props {
  postId?: string | null
}
const props = withDefaults(defineProps<Props>(), {
  postId: null,
})

Prism.manual = true
Prism.languages.console = Prism.languages['shell-session']

const router = useRouter()
const locales = [{ name: 'English', value: 'en' }, { name: 'Tiếng Việt', value: 'vi' }]
const post = ref<Post | null>(null)
const oldSlug = ref<string | null>(null)
const allCategories = ref<Category[]>([])
const allAuthors = ref<User[]>([])
const isSubmitting = ref(false)
const codeEditor = ref<HTMLDivElement | null>(null)
const kullnaEditor = ref<KullnaEditor | null>(null)
const previewHtml = ref<string | null>(null)
const validationErrors = ref<Record<string, string>>({})

const postLocale = computed({
  get() {
    return post.value?.locale || undefined
  },
  set(val) {
    if (post.value) {
      post.value.locale = (val || null)
    }
  },
})

const allAuthorChoices = computed(() => allAuthors.value.map(a => ({ name: a.email, value: a.id })))

const postAuthorId = computed({
  get() {
    return post.value?.author?.id || undefined
  },
  set(val) {
    if (post.value) {
      post.value.author = allAuthors.value.find(a => a.id === val) || null
    }
  },
})

async function fetchCategories() {
  const raw = await kyClient.get(API_GET_CATEGORIES).json()
  const resp = ObjectListResponseSchema.parse(raw)
  allCategories.value = z.array(CategorySchema).parse(resp.objects)
  let nextUrl = resp.links.next
  while (nextUrl) {
    const raw = await kyClient.get(nextUrl).json()
    const resp = ObjectListResponseSchema.parse(raw)
    allCategories.value = allCategories.value.concat(z.array(CategorySchema).parse(resp.objects))
    nextUrl = resp.links.next
  }
}

async function fetchAuthors() {
  const raw = await kyClient.get(API_GET_USERS).json()
  allAuthors.value = z.array(UserSchema).parse(raw)
}

async function fetchData() {
  await fetchCategories()
  await fetchAuthors()
  if (!props.postId) {
    post.value = PostSchema.parse({ created_at: new Date().toISOString() })
    return
  }
  const url = lightJoin(API_GET_POSTS, props.postId)
  const resp = await kyClient.get(url).json()
  post.value = PostSchema.parse(resp)
  oldSlug.value = post.value.slug
  if (kullnaEditor.value && post.value.body) {
    kullnaEditor.value.code = post.value.body
  }
}

function regenerateSlug() {
  if (post.value) {
    post.value.slug = slugify(post.value.title)
  }
}

function getValidationError(field: string) {
  return validationErrors.value[field] || ''
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
    validationErrors.value = await handleApiError(e)
  } finally {
    isSubmitting.value = false
  }
}

const mergeCodeUpdate = F.debounce((code: string) => {
  if (post.value) {
    post.value.body = code
  }
}, 500)

async function getHtmlPreview() {
  if (!post.value) {
    return
  }
  const resp = await kyClient.post(API_MARKDOWN_TO_HTML, { body: post.value.body }).text()
  previewHtml.value = resp
}

onBeforeMount(fetchData)
onMounted(() => {
  watch(
    () => post.value?.title,
    (title) => {
      // Don't regenerate slug, to avoid breaking URL
      if (post.value && title && !oldSlug.value) {
        post.value.slug = slugify(title)
      }
    },
    { flush: 'post' },
  )
  watch(codeEditor, (el) => {
    if (el) {
      kullnaEditor.value = createEditor(el, { highlightElement: Prism.highlightElement })
      kullnaEditor.value.wrapsText = true
      if (post.value?.body) {
        kullnaEditor.value.code = post.value.body
      }
      kullnaEditor.value.onUpdate(mergeCodeUpdate)
    }
  })
})
onBeforeUnmount(() => {
  if (kullnaEditor.value) {
    kullnaEditor.value.destroy()
  }
})
</script>
