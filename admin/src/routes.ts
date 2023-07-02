import { RouteRecordRaw, RouteComponent } from 'vue-router'

import NotFound from '@/views/NotFound.vue'
import MainWrap from '@/views/MainWrap.vue'

const LoginPage = (): Promise<RouteComponent> => import('./views/LoginPage.vue')
const BlogPostListPage = (): Promise<RouteComponent> => import('./views/BlogPostListPage.vue')
const BlogCategoryListPage = (): Promise<RouteComponent> => import('./views/BlogCategoryListPage.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: MainWrap,
    children: [
      { path: 'posts', component: BlogPostListPage, name: 'post-list' },
      { path: 'categories', component: BlogCategoryListPage, name: 'category-list' },
    ],
  },
  { path: '/login', component: LoginPage, name: 'login' },
  { path: '/:path(.*)', component: NotFound },
]
