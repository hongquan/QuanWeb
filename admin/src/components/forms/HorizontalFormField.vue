<template>
  <div class='sm:grid sm:grid-cols-4 sm:items-start sm:gap-2 sm:py-2'>
    <label
      :for='uid'
      class='block text-sm font-medium leading-6 dark:text-white sm:pt-2'
    >{{ label }}</label>
    <div class='mt-2 sm:col-span-3 sm:mt-0'>
      <Input
        :id='uid'
        v-model='value'
        size='sm'
      />
    </div>
  </div>
</template>

<script setup lang='ts'>
import { computed } from 'vue'
import { ulid } from 'ulidx'
import { Input } from 'flowbite-vue'

interface Props {
  modelValue: string
  label?: string
}
const props = withDefaults(defineProps<Props>(), {
  label: '',
})
const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const uid = ulid()

const value = computed({
  get() {
    return props.modelValue
  },
  set(v: string) {
    emit('update:modelValue', v)
  },
})
</script>
