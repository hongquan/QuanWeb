<template>
  <div class='max-w-md mx-auto'>
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
        class='text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm w-full sm:w-auto px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800'
      >
        Submit
      </button>
    </form>
  </div>
</template>

<script setup lang='ts'>
import { ref } from 'vue'

import { kyClient } from '@/common'
import { API_LOGIN } from '@/urls'
import { useStore } from '@/stores'
import { UserSchema } from '@/models/user'

const store = useStore()
const email = ref('')
const password = ref('')

async function onSubmit() {
  const resp = await kyClient.post(API_LOGIN, { json: { email: email.value, password: password.value } }).json()
  const user = UserSchema.parse(resp)
  store.user = user
  console.log(user)
}
</script>
