<template>
  <div class='max-w-lg mx-auto'>
    <form
      v-if='author'
      method='post'
      @submit.prevent='onSubmit'
    >
      <HorizontalFormField
        v-model='author.name'
        label='Name'
        :error-message='getValidationError("name")'
      />
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
import { onBeforeMount, ref } from 'vue'
import { useRouter } from 'vue-router'
import { FwbButton } from 'flowbite-vue'
import lightJoin from 'light-join'
import { D } from '@mobily/ts-belt'
import { toast } from 'vue-sonner'

import { BookAuthor, BookAuthorSchema } from '@/models/minors'
import { kyClient } from '@/common'
import { API_GET_BOOK_AUTHORS } from '@/urls'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'
import { handleApiError } from '@/utils/api'

interface Props {
  id?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  id: null,
})

const router = useRouter()
const author = ref<BookAuthor | null>(null)
const isSubmitting = ref(false)
const validationErrors = ref<Record<string, string>>({})

async function fetchData() {
  if (!props.id) {
    author.value = BookAuthorSchema.parse({})
    return
  }
  const url = lightJoin(API_GET_BOOK_AUTHORS, props.id)
  const resp = await kyClient.get(url).json()
  author.value = BookAuthorSchema.parse(resp)
}

function getValidationError(field: string) {
  return validationErrors.value[field] || ''
}

async function onSubmit() {
  if (!author.value) {
    return
  }
  isSubmitting.value = true
  const isCreating = !props.id
  const url = isCreating ? API_GET_BOOK_AUTHORS : lightJoin(API_GET_BOOK_AUTHORS, props.id)
  const postData = D.deleteKey(author.value, 'id')
  try {
    const resp = await kyClient(url, { method: isCreating ? 'post' : 'patch', json: postData }).json()
    const updatedAuthor = BookAuthorSchema.parse(resp)
    const message = isCreating ? `Author "${updatedAuthor.name}" is created.` : `Author "${updatedAuthor.name}" is updated.`
    toast.success(message)
    await router.push({ name: 'book-author.list' })
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
