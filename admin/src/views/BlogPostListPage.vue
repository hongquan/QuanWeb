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
          <BlogPostRow v-for='(post, index) in posts' :key='post.id' :post='post' :isOdd='index % 2 !== 0' />
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang='ts'>
import { onBeforeMount, ref } from 'vue'

import { kyClient } from '@/common'
import { API_GET_POSTS } from '@/urls'
import { ObjectListResponseSchema } from '@/models/api'
import { PostSchema, Post } from '@/models/blog'
import BlogPostRow from '@/components/BlogPostRow.vue'

const posts = ref<Post[]>([])

async function fetchData() {
  const resp = await kyClient.get(API_GET_POSTS).json()
  const data = ObjectListResponseSchema.parse(resp)
  posts.value = PostSchema.array().parse(data.objects)
}

onBeforeMount(fetchData)
</script>
