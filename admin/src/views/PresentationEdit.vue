<template>
  <div class='max-w-lg mx-auto'>
    <form
      v-if='presentation'
      method='post'
      @submit.prevent='onSubmit'
    >
      <HorizontalFormField
        v-model='presentation.title'
        label='Title'
      />
      <HorizontalFormField
        v-model='presentation.url'
        label='URL'
        widget-type='url'
      />
      <HorizontalFormField
        v-model='presentation.event'
        label='Event'
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
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import lightJoin from 'light-join'
import { Button as FbButton } from 'flowbite-vue'
import { D } from '@mobily/ts-belt'
import { toast } from 'vue-sonner'

import { kyClient } from '@/common'
import { API_GET_PRESENTATIONS } from '@/urls'
import { Presentation, PresentationSchema } from '@/models/minors'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'
import { handleApiError } from '@/utils/api'

interface Props {
  id?: string | null
}
const props = withDefaults(defineProps<Props>(), {
  id: null,
})

const router = useRouter()
const presentation = ref<Presentation | null>(null)
const isSubmitting = ref(false)
const validationErrors = ref<Record<string, string>>({})

async function fetchData() {
  const url = lightJoin(API_GET_PRESENTATIONS, props.id)
  const resp = await kyClient.get(url).json()
  presentation.value = PresentationSchema.parse(resp)
}

async function onSubmit() {
  if (!presentation.value) {
    return
  }
  isSubmitting.value = true
  const isCreating = !props.id
  const url = isCreating ? API_GET_PRESENTATIONS : lightJoin(API_GET_PRESENTATIONS, props.id)
  const postData = D.deleteKey(presentation.value, 'id')
  try {
    const resp = await kyClient(url, { method: isCreating ? 'post' : 'patch', json: postData }).json()
    const updatedTalk = PresentationSchema.parse(resp)
    const message = isCreating ? `Presentation "${updatedTalk.title}" is created.` : `Presentation "${updatedTalk.title}" is updated.`
    toast.success(message)
    await router.push({ name: 'presentation.list' })
  } catch (e) {
    console.log(e)
    validationErrors.value = await handleApiError(e)
  } finally {
    isSubmitting.value = false
  }
}

onMounted(async () => {
  await fetchData()
})
</script>
