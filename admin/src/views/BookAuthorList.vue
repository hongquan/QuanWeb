<template>
  <div>
    <div class='mb-3 sm:flex justify-end'>
      <RouterLink
        to='/books/authors/new'
        class='block text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm p-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800'
      >
        New author
      </RouterLink>
    </div>
    <LoadingIndicator
      v-if='isLoading'
      class='mt-32 w-16 h-auto mx-auto text-blue-500 fill-current'
    />

    <div
      v-else
      class='relative overflow-x-auto shadow-md sm:rounded-lg'
    >
      <table class='w-full text-sm text-left text-gray-600 dark:text-gray-400'>
        <thead class='text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
          <tr>
            <th
              scope='col'
              class='px-6 py-3'
            >
              Name
            </th>
            <th />
          </tr>
        </thead>
        <tbody>
          <BookAuthorRow
            v-for='(item, index) in authors'
            :key='item.id || index'
            :is-odd='Boolean(index % 2)'
            :author='item'
            @deleted='onDeleted'
          />
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang='ts'>
import { onBeforeMount, ref } from 'vue'
import LoadingIndicator from 'svg-loaders/svg-smil-loaders/circles.svg?component'

import { BookAuthor, BookAuthorSchema } from '@/models/minors'
import { kyClient } from '@/common'
import { API_GET_BOOK_AUTHORS } from '@/urls'
import { ObjectListResponseSchema } from '@/models/api'
import BookAuthorRow from '@/components/BookAuthorRow.vue'

const authors = ref<BookAuthor[]>([])
const totalPages = ref(1)
const isLoading = ref(true)

async function fethData() {
  isLoading.value = true
  const resp = await kyClient.get(API_GET_BOOK_AUTHORS).json()
  const data = ObjectListResponseSchema.parse(resp)
  authors.value = BookAuthorSchema.array().parse(data.objects)
  totalPages.value = data.total_pages
  isLoading.value = false
}

function onDeleted(id: string) {
  authors.value = authors.value.filter((item) => item.id !== id)
}

onBeforeMount(async () => {
  await fethData()
})
</script>
