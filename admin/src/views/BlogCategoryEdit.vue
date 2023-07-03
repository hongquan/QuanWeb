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
      />
      <HorizontalFormField
        v-model='category.slug'
        label='Slug'
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
import { D } from '@mobily/ts-belt'

import { kyClient } from '@/common'
import { Category, CategorySchema } from '@/models/blog'
import { API_GET_CATEGORIES } from '@/urls'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'

interface Props {
  categoryId?: string | null
}
const props = withDefaults(defineProps<Props>(), {
  categoryId: null,
})

const router = useRouter()
const category = ref<Category | null>(null)
const isSubmitting = ref(false)

async function fetchData() {
  if (!props.categoryId) {
    category.value = CategorySchema.parse({})
    return
  }
  const url = lightJoin(API_GET_CATEGORIES, props.categoryId)
  const raw = await kyClient.get(url).json()
  category.value = CategorySchema.parse(raw)
}

async function onSubmit() {
  if (!category.value) {
    return
  }
  isSubmitting.value = true
  const isCreating = !props.categoryId
  const url = category.value.id ? lightJoin(API_GET_CATEGORIES, category.value.id) : API_GET_CATEGORIES
  const postData = D.deleteKey(category.value, 'id')
  const resp = await kyClient(url, { json: postData, method: isCreating ? 'post' : 'patch' }).json()
  const updatedCat = CategorySchema.parse(resp)
  const message = isCreating ? `Category "${updatedCat.title}" is created!` : `Category "${updatedCat.title}" is updated!`
  toast.success(message)
  isSubmitting.value = false
  await router.push({ name: 'category.list' })
}

onBeforeMount(fetchData)
onMounted(() => {
  watch(
    () => category.value?.title,
    (title) => {
      if (title) {
        category.value!.slug = slugify(title)
      }
    },
    { flush: 'post' },
  )
})
</script>