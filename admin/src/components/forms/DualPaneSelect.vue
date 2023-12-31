<template>
  <div class='sm:grid sm:grid-cols-4 sm:items-start sm:gap-2 sm:py-2'>
    <label class='block text-sm font-medium leading-6 dark:text-white sm:pt-2'>
      {{ label }}
    </label>
    <div class='mt-2 sm:col-span-3 sm:mt-0 grid grid-cols-2 gap-4 text-sm'>
      <div class='border border-gray-300 dark:border-gray-600 p-2 pt-0 rounded min-h-20 space-y-2'>
        <label class='text-xs'>Available</label>
        <div class='overflow-y-auto max-h-20'>
          <ul v-if='availableOptions.length'>
            <li
              v-for='(opt, i) in availableOptions'
              :key='opt.id || i'
            >
              <button
                type='button'
                class='pl-2'
                :class='BUTTON_CLASS'
                @click='take(opt)'
              >
                <span>{{ opt.title }}</span>
                <Icon
                  class='h-5 w-5'
                  icon='gg:move-right'
                />
              </button>
            </li>
          </ul>
        </div>
      </div>
      <div class='border border-gray-300 dark:border-gray-600 p-2 pt-0 rounded space-y-2'>
        <label class='text-xs'>Selected</label>
        <div class='overflow'>
          <ul
            v-if='selectedOptions.length'
            class='dark:text-white'
          >
            <li
              v-for='(opt, i) in selectedOptions'
              :key='opt.id || i'
            >
              <button
                type='button'
                class='pr-2'
                :class='BUTTON_CLASS'
                @click='release(opt)'
              >
                <Icon
                  class='h-5 w-5'
                  icon='gg:move-left'
                />
                <span>{{ opt.title }}</span>
              </button>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang='ts'>
import { computed } from 'vue'
import { Icon } from '@iconify/vue'

import { SelectableEntity } from '@/types'

interface Props {
  label?: string,
  allOptions?: SelectableEntity[]
  selectedOptions?: SelectableEntity[]
}

const BUTTON_CLASS = 'w-full rounded py-0.5 hover:bg-gray-100 dark:hover:bg-gray-600 dark:hover:text-white flex flex-row justify-between items-center'

const props = withDefaults(defineProps<Props>(), {
  label: 'Select',
  allOptions: () => [],
  selectedOptions: () => [],
})
const emit = defineEmits<{
  taken: [id: string]
  released: [id: string]
}>()

const availableOptions = computed(() => {
  if (!props.selectedOptions.length) {
    return props.allOptions
  }
  const occupied = props.selectedOptions.map((c) => c.id)
  return props.allOptions.filter((c) => !occupied.includes(c.id))
})

function take(object: SelectableEntity) {
  object.id && emit('taken', object.id)
}

function release(object: SelectableEntity) {
  object.id && emit('released', object.id)
}
</script>
