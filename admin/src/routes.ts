import { RouteRecordRaw, RouteComponent } from 'vue-router'

import NotFound from '@/views/NotFound.vue'
import MainWrap from '@/views/MainWrap.vue'
import SimpleWrap from '@/views/SimpleWrap.vue'

const LoginPage = (): Promise<RouteComponent> => import('./views/LoginPage.vue')
const BlogPostList = (): Promise<RouteComponent> => import('./views/BlogPostList.vue')
const BlogPostEdit = (): Promise<RouteComponent> => import('./views/BlogPostEdit.vue')
const BlogCategoryList = (): Promise<RouteComponent> => import('./views/BlogCategoryList.vue')
const BlogCategoryEdit = (): Promise<RouteComponent> => import('./views/BlogCategoryEdit.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: MainWrap,
    redirect: () => ({ name: 'post.list' }),
    children: [
      {
        path: 'posts', component: SimpleWrap, name: 'post', redirect: () => ({ name: 'post.list' }),
        children: [
          { path: '', component: BlogPostList, name: 'post.list' },
          { path: 'new', component: BlogPostEdit, name: 'post.new' },
          { path: ':postId', component: BlogPostEdit, name: 'post.edit', props: true },
        ],
      },
      {
        path: 'categories', component: SimpleWrap, redirect: () => ({ name: 'category.list' }),
        children: [
          { path: '', component: BlogCategoryList, name: 'category.list' },
          { path: 'new', component: BlogCategoryEdit, name: 'category.new' },
          { path: ':categoryId', component: BlogCategoryEdit, name: 'category.edit', props: true },
        ],
      },
    ],
  },
  { path: '/login', component: LoginPage, name: 'login' },
  { path: '/:path(.*)', component: NotFound },
]
