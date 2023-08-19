<template>
  <tr :class='classNames'>
    <th
      scope='row'
      class='px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white'
    >
      <RouterLink
        :to='editUrl'
        class='hover:underline'
      >
        {{ presentation.title }}
      </RouterLink>
    </th>
    <td class='px-6 py-4'>
      {{ presentation.event }}
    </td>
    <td>
      <button
        class='hover:text-red-500'
        @click='deletePresentation'
      >
        <Icon
          icon='ic:outline-delete-forever'
          class='w-5 h-5'
        />
      </button>
    </td>
  </tr>
</template>

<script setup lang='ts'>
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import lightJoin from 'light-join'
import HStatus from 'http-status'
import { toast } from 'vue-sonner'

import { Presentation } from '@/models/minors'
import { API_GET_PRESENTATIONS } from '@/urls'
import { kyClient } from '@/common'

interface Props {
  presentation: Presentation
  isOdd?: boolean
}
const props = withDefaults(defineProps<Props>(), {
  isOdd: false,
})
const emit = defineEmits<{
  deleted: [id: string],
}>()

const classNames = computed(() => [
  props.isOdd ? 'bg-white dark:bg-gray-900' : 'bg-gray-50 dark:bg-gray-800',
  'border-b',
  'dark:border-gray-700',
])

const editUrl = computed(() => ({
  name: 'presentation.edit',
  params: { id: props.presentation.id },
}))

async function deletePresentation() {
  if (!props.presentation.id) {
    return
  }
  const sure = confirm(`Are you sure you want to delete "${props.presentation.title}" presentation?`)
  if (!sure) {
    return
  }
  const title = props.presentation.title || 'Untitled'
  const url = lightJoin(API_GET_PRESENTATIONS, props.presentation.id)
  const resp = await kyClient.delete(url)
  if (resp.status !== HStatus.NO_CONTENT) {
    toast.error('Failed to delete the presentation')
    return
  }
  toast.success(`Presentation "${title}" is deleted.`)
  emit('deleted', props.presentation.id)
}
</script>
