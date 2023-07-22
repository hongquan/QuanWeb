<template>
  <div>
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

onBeforeMount(async () => {
  await fethData()
})
</script>
