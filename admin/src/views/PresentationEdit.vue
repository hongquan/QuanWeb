<template>
  <div class='max-w-lg mx-auto'>
    <form
      v-if='presentation'
      method='post'
      @submit='onSubmit'
    >
      <HorizontalFormField
        v-model='presentation.title'
        label='Title'
      />
      <HorizontalFormField
        v-model='presentation.url'
        label='URL'
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
import lightJoin from 'light-join'
import { Button as FbButton } from 'flowbite-vue'

import { kyClient } from '@/common'
import { API_GET_PRESENTATIONS } from '@/urls'
import { Presentation, PresentationSchema } from '@/models/minors'
import HorizontalFormField from '@/components/forms/HorizontalFormField.vue'

interface Props {
  id?: string | null
}
const props = withDefaults(defineProps<Props>(), {
  id: null,
})

const presentation = ref<Presentation | null>(null)
const isSubmitting = ref(false)

async function fetchData() {
  const url = lightJoin(API_GET_PRESENTATIONS, props.id)
  const resp = await kyClient.get(url).json()
  presentation.value = PresentationSchema.parse(resp)
}

async function onSubmit() {

}

onMounted(async () => {
  await fetchData()
})
</script>
