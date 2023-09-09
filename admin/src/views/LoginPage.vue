<template>
  <div class='max-w-md mx-auto px-4'>
    <h1 class='text-2xl py-4'>
      Login
    </h1>

    <form
      class='mt-12'
      method='post'
      @submit.prevent='onSubmit'
    >
      <div class='mb-6'>
        <label
          for='email'
          class='block mb-2 text-sm font-medium text-gray-900 dark:text-white'
        >Your email</label>
        <input
          id='email'
          v-model='email'
          type='email'
          class='bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500'
          autocomplete='email'
          required
        >
      </div>
      <div class='mb-6'>
        <label
          for='password'
          class='block mb-2 text-sm font-medium text-gray-900 dark:text-white'
        >Your password</label>
        <input
          id='password'
          v-model='password'
          type='password'
          class='bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500'
          required
        >
      </div>
      <div class='flex items-start mb-6'>
        <div class='flex items-center h-5'>
          <input
            id='remember'
            v-model='remember'
            name='remember'
            type='checkbox'
            class='w-4 h-4 border border-gray-300 rounded bg-gray-50 focus:ring-3 focus:ring-blue-300 dark:bg-gray-700 dark:border-gray-600 dark:focus:ring-blue-600 dark:ring-offset-gray-800 dark:focus:ring-offset-gray-800'
          >
        </div>
        <label
          for='remember'
          class='ml-2 text-sm font-medium text-gray-900 dark:text-gray-300'
        >Remember me</label>
      </div>
      <button
        type='submit'
        class='flex items-center space-x-2 text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg w-full sm:w-auto px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800'
      >
        <Icon
          v-if='isSubmitting'
          icon='line-md:loading-twotone-loop'
          class='w-5 h-auto'
        />
        <span>Submit</span>
      </button>
    </form>
  </div>
</template>

<script setup lang='ts'>
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { toast } from 'vue-sonner'
import { HTTPError } from 'ky'
import { Icon } from '@iconify/vue'

import { kyClient } from '@/common'
import { API_LOGIN } from '@/urls'
import { useStore } from '@/stores'
import { UserSchema } from '@/models/user'
import { GeneralErrorResponseSchema } from '@/models/api'

const router = useRouter()
const route = useRoute()
const store = useStore()
const email = ref('')
const password = ref('')
const remember = ref(false)
const isSubmitting = ref(false)

async function onSubmit() {
  isSubmitting.value = true
  try {
    const resp = await kyClient.post(API_LOGIN, { json: { email: email.value, password: password.value, remember_me: remember.value } }).json()
    const user = UserSchema.parse(resp)
    store.user = user
    toast.success('Login successfully')
    console.log(user)
    const attemptUrl = route.query.attempt as string || '/'
    await router.push(attemptUrl)
  } catch (e) {
    console.info(e)
    if (e instanceof HTTPError) {
      const parsedResp = GeneralErrorResponseSchema.safeParse(await e.response.json())
      if (parsedResp.success) {
        toast.error(parsedResp.data.message)
        return
      }
    }
    toast.error('Login failed')
  } finally {
    isSubmitting.value = false
  }
}
</script>
