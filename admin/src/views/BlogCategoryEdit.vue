<template>
  <div class='max-w-lg mx-auto'>
    <form
      v-if='category'
      method='post'
      @submit.prevent='onSubmit'
    >
      <HorizontalFormField
        v-model='category.title'
        label='Title'
        required
        :error-message='getValidationError("title")'
      />
      <HorizontalFormFieldWrap>
        <template #label>
          Slug
        </template>
        <template #default>
          <FbInput
            v-model='category.slug'
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
import { Button as FbButton, Input as FbInput } from 'flowbite-vue'
import { toast } from 'vue-sonner'
import { D } from '@mobily/ts-belt'
import { Icon } from '@iconify/vue'

import { kyClient } from '@/common'
import { Category, CategorySchema } from '@/models/blog'
import { API_GET_CATEGORIES } from '@/urls'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'
import HorizontalFormFieldWrap from '@/components/forms/HorizontalFormFieldWrap.vue'
import { handleApiError } from '@/utils/api'

interface Props {
  categoryId?: string | null
}
const props = withDefaults(defineProps<Props>(), {
  categoryId: null,
})

const router = useRouter()
const category = ref<Category | null>(null)
const oldSlug = ref<string | null>(null)
const isSubmitting = ref(false)
const validationErrors = ref<Record<string, string>>({})

async function fetchData() {
  if (!props.categoryId) {
    category.value = CategorySchema.parse({})
    return
  }
  const url = lightJoin(API_GET_CATEGORIES, props.categoryId)
  const raw = await kyClient.get(url).json()
  category.value = CategorySchema.parse(raw)
  oldSlug.value = category.value.slug
}

function regenerateSlug() {
  if (category.value) {
    category.value.slug = slugify(category.value.title)
  }
}

async function onSubmit() {
  if (!category.value) {
    return
  }
  const isCreating = !props.categoryId
  const url = category.value.id ? lightJoin(API_GET_CATEGORIES, category.value.id) : API_GET_CATEGORIES
  const postData = D.deleteKey(category.value, 'id')
  isSubmitting.value = true
  try {
    const resp = await kyClient(url, { json: postData, method: isCreating ? 'post' : 'patch' }).json()
    const updatedCat = CategorySchema.parse(resp)
    const message = isCreating ? `Category "${updatedCat.title}" is created!` : `Category "${updatedCat.title}" is updated!`
    toast.success(message)
    await router.push({ name: 'category.list' })
  } catch (e) {
    await handleApiError(e)
  } finally {
    isSubmitting.value = false
  }
}

function getValidationError(field: string) {
  return validationErrors.value[field] || ''
}

onBeforeMount(fetchData)
onMounted(() => {
  watch(
    () => category.value?.title,
    (title) => {
      if (category.value && title && !oldSlug.value) {
        category.value.slug = slugify(title)
      }
    },
    { flush: 'post' },
  )
})
</script>
