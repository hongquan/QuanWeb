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
          <BlogCategoryRow v-for='(item, index) in categories' :key='item.id' :category='item' :isOdd='Boolean(index % 2)' />
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang='ts'>
import { onBeforeMount, ref } from 'vue'

import { kyClient } from '@/common'
import { API_GET_CATEGORIES } from '@/urls'
import { ObjectListResponseSchema } from '@/models/api'
import { Category, CategorySchema } from '@/models/blog'
import BlogCategoryRow from '@/components/BlogCategoryRow.vue'

const categories = ref<Category[]>([])

async function fetchData() {
  const resp = await kyClient.get(API_GET_CATEGORIES).json()
  const data = ObjectListResponseSchema.parse(resp)
  categories.value = CategorySchema.array().parse(data.objects)
}

onBeforeMount(fetchData)
</script>
