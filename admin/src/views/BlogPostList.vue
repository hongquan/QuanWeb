<template>
  <div>
    <div class='mb-3 sm:flex justify-end'>
      <RouterLink
        to='/posts/new'
        class='block text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm p-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800'
      >
        New post
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
              Title
            </th>
            <th
              scope='col'
              class='px-6 py-3'
            >
              Slug
            </th>
            <th
              scope='col'
              class='px-6 py-3'
            >
              Categories
            </th>
            <th
              scope='col'
              class='px-6 py-3'
            />
          </tr>
        </thead>
        <tbody>
          <BlogPostRow
            v-for='(post, index) in posts'
            :key='post.id || index'
            :post='post'
            :is-odd='Boolean(index % 2)'
            @deleted='onDeleted'
          />
        </tbody>
      </table>
    </div>
    <div
      v-if='!isLoading'
      class='text-center'
    >
      <Paginator
        :total-pages='totalPages'
        :current-page='currentPage'
        class='mt-6'
      />
    </div>
  </div>
</template>

<script setup lang='ts'>
import { computed, onBeforeMount, ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import LoadingIndicator from 'svg-loaders/svg-smil-loaders/circles.svg?component'

import { kyClient } from '@/common'
import { API_GET_POSTS } from '@/urls'
import { ObjectListResponseSchema } from '@/models/api'
import { PostSchema, Post } from '@/models/blog'
import BlogPostRow from '@/components/BlogPostRow.vue'
import Paginator from '@/components/Paginator.vue'

const route = useRoute()
const posts = ref<Post[]>([])
const totalPages = ref(1)
const isLoading = ref(true)

const currentPage = computed(() => Number(route.query.page) || 1)

async function fetchData() {
  const searchParams = {
    page: currentPage.value,
  }
  const resp = await kyClient.get(API_GET_POSTS, { searchParams }).json()
  const data = ObjectListResponseSchema.parse(resp)
  posts.value = PostSchema.array().parse(data.objects)
  totalPages.value = data.total_pages
  isLoading.value = false
}

function onDeleted(id: string) {
  posts.value = posts.value.filter(item => item.id !== id)
}

onBeforeMount(fetchData)

onMounted(() => {
  watch(
    () => route.query,
    fetchData,
    { flush: 'post' },
  )
})
</script>
