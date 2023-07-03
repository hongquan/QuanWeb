<template>
  <div>
    <div class='relative overflow-x-auto shadow-md sm:rounded-lg'>
      <table class='w-full text-sm text-left text-gray-500 dark:text-gray-400'>
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
            <th />
          </tr>
        </thead>
        <tbody>
          <BlogCategoryRow
            v-for='(item, index) in categories'
            :key='item.id || index'
            :category='item'
            :is-odd='Boolean(index % 2)'
            @deleted='onDeleted'
          />
        </tbody>
      </table>
    </div>
    <div class='text-center'>
      <Paginator
        :total-pages='totalPages'
        :current-page='currentPage'
        class='mt-6'
      />
    </div>
  </div>
</template>

<script setup lang='ts'>
import { onBeforeMount, ref, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'

import { kyClient } from '@/common'
import { API_GET_CATEGORIES } from '@/urls'
import { ObjectListResponseSchema } from '@/models/api'
import { Category, CategorySchema } from '@/models/blog'
import BlogCategoryRow from '@/components/BlogCategoryRow.vue'
import Paginator from '@/components/Paginator.vue'

const route = useRoute()
const categories = ref<Category[]>([])
const totalPages = ref(1)

const currentPage = computed(() => Number(route.query.page) || 1)

async function fetchData() {
  const searchParams = {
    page: currentPage.value,
  }
  const resp = await kyClient.get(API_GET_CATEGORIES, { searchParams }).json()
  const data = ObjectListResponseSchema.parse(resp)
  categories.value = CategorySchema.array().parse(data.objects)
}

function onDeleted(id: string) {
  categories.value = categories.value.filter((item) => item.id !== id)
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
