<template>
  <HorizontalFormFieldWrap>
    <template #label>
      {{ label }}
    </template>
    <template #default='{ inputId }'>
      <input
        v-if='widgetType === "checkbox"'
        :id='inputId'
        v-model='value'
        type='checkbox'
        :required='required'
      >
      <FwbInput
        v-else
        :id='inputId'
        v-model='htmlAttrValue'
        :type='widgetType'
        size='sm'
        :required='required'
      />
    </template>
    <template
      v-if='errorMessage'
      #validationMessage
    >
      <span class='text-red-600 dark:text-red-500 text-sm'>{{ errorMessage }}</span>
    </template>
  </HorizontalFormFieldWrap>
</template>

<script setup lang='ts'>
import { computed } from 'vue'
import { FwbInput } from 'flowbite-vue'
import { G } from '@mobily/ts-belt'

import HorizontalFormFieldWrap from '@/components/forms/HorizontalFormFieldWrap.vue'

export interface SelectOption {
  name: string
  value: string
}

interface Props {
  modelValue: string | boolean | null
  widgetType?: 'number' | 'hidden' | 'color' | 'text' | 'search' | 'image' | 'button' | 'checkbox' | 'date' | 'datetime-local' | 'email' | 'file' | 'month' | 'password' | 'radio' | 'range' | 'reset' | 'submit' | 'tel' | 'time' | 'url' | 'week'
  label?: string
  required?: boolean
  choices?: SelectOption[],
  errorMessage?: string,
}
const props = withDefaults(defineProps<Props>(), {
  label: '',
  widgetType: 'text',
  required: false,
  choices: () => [],
  errorMessage: '',
})
const emit = defineEmits<{
  'update:modelValue': [value: string | boolean | null]
}>()

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
