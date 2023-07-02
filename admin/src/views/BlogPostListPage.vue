<template>
  <div>
    <div class='relative overflow-x-auto shadow-md sm:rounded-lg'>
      <table class='w-full text-sm text-left text-gray-500 dark:text-gray-400'>
        <thead class='text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400'>
          <tr>
            <th scope='col' class='px-6 py-3'>
              Title
            </th>
            <th scope='col' class='px-6 py-3'>
              Slug
            </th>
          </tr>
        </thead>
        <tbody>
          <BlogPostRow v-for='(post, index) in posts' :key='post.id' :post='post' :isOdd='Boolean(index % 2)' />
        </tbody>
      </table>
    </div>
    <div class='text-center'>
      <Paginator :total-pages='totalPages' :current-page='currentPage' class='mt-6' />
    </div>
  </div>
</template>

<script setup lang='ts'>
import { computed, onBeforeMount, ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'

import { kyClient } from '@/common'
import { API_GET_POSTS } from '@/urls'
import { ObjectListResponseSchema } from '@/models/api'
import { PostSchema, Post } from '@/models/blog'
import BlogPostRow from '@/components/BlogPostRow.vue'
import Paginator from '@/components/Paginator.vue'

const route = useRoute()
const posts = ref<Post[]>([])
const totalPages = ref(1)

const currentPage = computed(() => Number(route.query.page) || 1)

async function fetchData() {
  const searchParams = {
    page: currentPage.value
  }
  const resp = await kyClient.get(API_GET_POSTS, { searchParams }).json()
  const data = ObjectListResponseSchema.parse(resp)
  posts.value = PostSchema.array().parse(data.objects)
  totalPages.value = data.total_pages
}

onBeforeMount(fetchData)

onMounted(() => {
  watch(
    () => route.query,
    fetchData,
    { flush: 'post' }
  )
})
</script>
