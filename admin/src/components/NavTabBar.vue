<template>
  <nav
    class='text-sm font-medium text-center text-gray-500 border-b border-gray-200 dark:text-gray-400 dark:border-gray-700'
  >
    <ul class='flex flex-wrap -mb-px'>
      <NavLink route-name='post.list'>
        Posts
      </NavLink>
      <NavLink route-name='category.list'>
        Categories
      </NavLink>
      <NavLink route-name='presentation.list'>
        Presentations
      </NavLink>
      <NavLink route-name='books'>
        Books
      </NavLink>
    </ul>
  </nav>
</template>

<script setup lang='ts'>
import NavLink from './NavLink.vue'
</script>
