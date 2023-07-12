<template>
  <div class='sm:grid sm:grid-cols-4 sm:items-start sm:gap-2 sm:py-2'>
    <label
      :for='uid'
      class='block text-sm font-medium leading-6 dark:text-white sm:pt-2'
    >{{ label }}</label>
    <div class='mt-2 sm:col-span-3 sm:mt-0'>
      <FbSelect
        v-if='(G.isString(value) || G.isNull(value)) && choices.length'
        :id='uid'
        v-model='htmlAttrValue'
        :options='choices'
        :required='required'
      />
      <FbInput
        v-else-if='G.isString(value) || G.isNull(value)'
        :id='uid'
        v-model='htmlAttrValue'
        size='sm'
        :required='required'
      />
      <input
        v-else-if='G.isBoolean(value)'
        :id='uid'
        v-model='value'
        class='mt-3'
        type='checkbox'
        :required='required'
      >
    </div>
  </div>
</template>

<script setup lang='ts'>
import { computed } from 'vue'
import { nanoid } from 'nanoid'
import { Input as FbInput } from 'flowbite-vue'
import { Select as FbSelect } from 'flowbite-vue'
import { G } from '@mobily/ts-belt'

export interface SelectOption {
  name: string
  value: string
}

interface Props {
  modelValue: string | boolean | null
  label?: string
  required?: boolean
  choices?: SelectOption[]
}
const props = withDefaults(defineProps<Props>(), {
  label: '',
  required: false,
  choices: () => [],
})
const emit = defineEmits<{
  'update:modelValue': [value: string | boolean | null]
}>()

const uid = nanoid()

const value = computed({
  get() {
    return props.modelValue
  },
  set(v: string | boolean | null) {
    emit('update:modelValue', v)
  },
})

const htmlAttrValue = computed({
  get() {
    if (G.isBoolean(props.modelValue)) {
      return props.modelValue ? 'true' : 'false'
    }
    return props.modelValue || undefined
  },
  set(v: string | undefined) {
    emit('update:modelValue', v || null)
  },
})
</script>
