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
    <td />
  </tr>
</template>

<script setup lang='ts'>
import { Presentation } from '@/models/minors'
import { computed } from 'vue';

interface Props {
  presentation: Presentation,
  isOdd?: boolean
}
const props = withDefaults(defineProps<Props>(), {
  isOdd: false,
})

const classNames = computed(() => [
  props.isOdd ? 'bg-white dark:bg-gray-900' : 'bg-gray-50 dark:bg-gray-800',
  'border-b',
  'dark:border-gray-700',
])

const editUrl = computed(() => ({
  name: 'presentation.edit',
  params: { id: props.presentation.id },
}))
</script>
