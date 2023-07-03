<template>
  <nav
    class='relative z-0 inline-flex rounded-md shadow-sm -space-x-px'
    aria-label='Pagination'
  >
    <RouterLink
      :to='prevRoute'
      class='block px-3 py-2 ml-0 leading-tight text-gray-500 bg-white border border-gray-300 rounded-l-lg hover:bg-gray-100 hover:text-gray-700 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white'
    >
      <span class='sr-only'>Previous</span>
      <Icon
        class='h-5 w-5'
        icon='heroicons-solid:chevron-left'
      />
    </RouterLink>
    <template
      v-for='(l, i) in genLinks()'
      :key='i'
    >
      <RouterLink
        v-if='!l.isEllipsis'
        :to='getRouteForPage(l.page)'
        class='px-3 py-2 leading-tight min-w-8'
        :class='l.cssClass'
        :aria-current="l.isCurrent ? 'page': null"
      >
        {{ l.label }}
      </RouterLink>
      <span
        v-else
        class=''
      >…</span>
    </template>
    <RouterLink
      :to='nextRoute'
      class='block px-3 py-2 leading-tight text-gray-500 bg-white border border-gray-300 rounded-r-lg hover:bg-gray-100 hover:text-gray-700 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white'
    >
      <span class='sr-only'>Next</span>
      <Icon
        class='h-5 w-5'
        icon='heroicons-solid:chevron-right'
      />
    </RouterLink>
  </nav>
</template>

<script setup lang='ts'>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { Icon } from '@iconify/vue'

interface PageLinkItem {
  label: string
  page: number
  isEllipsis: boolean
  isCurrent: boolean
  cssClass: string
}

interface Props {
  totalPages?: number
  currentPage?: number
  padding?: number
}

const ACTIVE_CLASS = 'z-10 text-blue-600 border-blue-300 bg-blue-50 hover:bg-blue-100 hover:text-blue-700 dark:bg-gray-700 dark:text-white'
const INACTIVE_CLASS = 'text-gray-500 border-gray-300 bg-white hover:bg-gray-100 hover:text-gray-700 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white'

const props = withDefaults(defineProps<Props>(), {
  totalPages: 1,
  currentPage: 1,
  padding: 3,
})

const route = useRoute()
const prevRoute = computed(() => ({
  query: Object.assign({}, route.query, { page: Math.max(props.currentPage - 1, 1) }),
}))
const nextRoute = computed(() => ({
  query: Object.assign({}, route.query, { page: Math.min(props.currentPage + 1, props.totalPages) }),
}))

function getRouteForPage(p: number) {
  return {
    query: Object.assign({}, route.query, { page: p }),
  }
}

function genLinks(): Array<PageLinkItem> {
  const totalPages = props.totalPages
  const currentPage = props.currentPage
  const paddingWidth = props.padding
  const totalDisplay = paddingWidth * 2 + 1
  if (props.totalPages <= totalDisplay) {
    return Array.from(
      Array(totalPages),
      (v, i) => {
        i += 1
        return {
          label: i.toString(),
          page: i,
          isEllipsis: false,
          isCurrent: (i === currentPage),
          cssClass: (i === currentPage) ? ACTIVE_CLASS : INACTIVE_CLASS,
        }
      })
  }
  let ellipsisIndex = paddingWidth
  if (currentPage === paddingWidth) {
    ellipsisIndex = currentPage + 1
  } else if (currentPage === totalPages - paddingWidth + 1) {
    ellipsisIndex = totalDisplay - paddingWidth - 2
  }
  return Array.from(
    Array(totalDisplay),
    (v, i) => {
      let iPage = 0
      let isEllipsis = false
      // When current page is around beginning or around the end, we show one ellipsis
      if (currentPage <= paddingWidth || currentPage > totalPages - paddingWidth) {
        isEllipsis = (i === ellipsisIndex)
        iPage = (i < ellipsisIndex) ? i + 1 : totalPages - 2 * paddingWidth + i
      } else {
      // When current page is at the middle, we show two ellipses.
      // We don't base on ellipsisIndex anymore.
        if (i === 0) {
          // Always show first page
          iPage = 1
        } else if (i === totalDisplay - 1) {
          // Always show last page
          iPage = totalPages
        } else {
          // Show closest neighbor pages
          iPage = currentPage - paddingWidth + i
        }
        isEllipsis = (i === 1 || i === totalDisplay - 2)
      }
      return {
        label: isEllipsis ? '…' : iPage.toString(),
        page: iPage,
        isEllipsis,
        isCurrent: (iPage === currentPage),
        cssClass: (iPage === currentPage) ? ACTIVE_CLASS : INACTIVE_CLASS,
      }
    },
  )
}
</script>
