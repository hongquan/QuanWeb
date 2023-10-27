<template>
  <div class='max-w-lg mx-auto'>
    <form
      v-if='book'
      method='post'
      @submit.prevent='onSubmit'
    >
      <HorizontalFormField
        v-model='book.title'
        label='Name'
        required
        :error-message='getValidationError("title")'
      />
      <HorizontalFormField
        v-model='book.download_url'
        label='Download URL'
        widget-type='url'
        required
      />
      <HorizontalFormFieldWrap>
        <template #label>
          Author
        </template>
        <template #default>
          <FwbSelect
            v-model='selectedAuthorId'
            :options='authorOptions'
          />
        </template>
      </HorizontalFormFieldWrap>
      <div class='text-center mt-2'>
        <FwbButton
          type='submit'
          :loading='isSubmitting'
        >
          Save
        </FwbButton>
      </div>
    </form>
  </div>
</template>

<script setup lang='ts'>
import { computed, onBeforeMount, ref } from 'vue'
import { useRouter } from 'vue-router'
import { FwbButton, FwbSelect } from 'flowbite-vue'
import lightJoin from 'light-join'
import { A } from '@mobily/ts-belt'
import { toast } from 'vue-sonner'

import { Book, BookAuthor, BookAuthorSchema, BookSchema } from '@/models/minors'
import { kyClient } from '@/common'
import { API_GET_BOOKS, API_GET_BOOK_AUTHORS } from '@/urls'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'
import HorizontalFormFieldWrap from '@/components/forms/HorizontalFormFieldWrap.vue'
import { handleApiError } from '@/utils/api'
import { ObjectListResponseSchema } from '@/models/api'
import { transformBookForPosting } from '@/utils/models'

interface Props {
  id?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  id: null,
})

const router = useRouter()
const book = ref<Book | null>(null)
const authors = ref<BookAuthor[]>([])
const isSubmitting = ref(false)
const validationErrors = ref<Record<string, string>>({})

const authorOptions = computed(() => (
  authors.value.map((author) => ({
    name: author.name,
    value: author.id || '',
  }))
))

const selectedAuthorId = computed({
  get() {
    if (!book.value || !book.value.author) {
      return ''
    }
    return book.value.author.id || ''
  },
  set(value: string) {
    if (!book.value) {
      return
    }
    const selectedAuthor = A.getBy(authors.value, a => a.id === value)
    book.value.author = selectedAuthor || null
  },
})

async function fetchAuthors() {
  try {
    const resp = await kyClient.get(API_GET_BOOK_AUTHORS).json()
    const data = ObjectListResponseSchema.parse(resp)
    authors.value = BookAuthorSchema.array().parse(data.objects)
  } catch (e) {
    console.debug(e)
    toast.error('Failed to fetch authors.')
  }
}

async function fetchData() {
  await fetchAuthors()
  if (!props.id) {
    book.value = BookSchema.parse({})
    return
  }
  const url = lightJoin(API_GET_BOOKS, props.id)
  const resp = await kyClient.get(url).json()
  book.value = BookSchema.parse(resp)
}

function getValidationError(field: string) {
  return validationErrors.value[field] || ''
}

async function onSubmit() {
  if (!book.value) {
    return
  }
  isSubmitting.value = true
  const isCreating = !props.id
  const url = isCreating ? API_GET_BOOKS : lightJoin(API_GET_BOOKS, props.id)
  const postData = transformBookForPosting(book.value)
  try {
    const resp = await kyClient(url, { method: isCreating ? 'post' : 'patch', json: postData }).json()
    const updatedBook = BookSchema.parse(resp)
    const message = isCreating ? `Book "${updatedBook.title}" is created.` : `Book "${updatedBook.title}" is updated.`
    toast.success(message)
    await router.push({ name: 'book.list' })
  } catch (e) {
    console.debug(e)
    validationErrors.value = await handleApiError(e)
  } finally {
    isSubmitting.value = false
  }
}

onBeforeMount(async () => {
  await fetchData()
})
</script>
