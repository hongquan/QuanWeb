import { RouteRecordRaw, RouteComponent } from 'vue-router'

import NotFound from '@/views/NotFound.vue'
import MainWrap from '@/views/MainWrap.vue'
import SimpleWrap from '@/views/SimpleWrap.vue'
import { authRequired } from '@/guards'

const LoginPage = (): Promise<RouteComponent> => import('./views/LoginPage.vue')
const LogoutPage = (): Promise<RouteComponent> => import('./views/LogoutPage.vue')
const BlogPostList = (): Promise<RouteComponent> => import('./views/BlogPostList.vue')
const BlogPostEdit = (): Promise<RouteComponent> => import('./views/BlogPostEdit.vue')
const BlogCategoryList = (): Promise<RouteComponent> => import('./views/BlogCategoryList.vue')
const BlogCategoryEdit = (): Promise<RouteComponent> => import('./views/BlogCategoryEdit.vue')
const PresentationList = (): Promise<RouteComponent> => import('./views/PresentationList.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: MainWrap,
    name: 'home',
    beforeEnter: authRequired,
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
      {
        path: 'presentations', component: SimpleWrap,
        children: [
          { path: '', component: PresentationList, name: 'presentation.list' }
        ]
      }
    ],
  },
  { path: '/login', component: LoginPage, name: 'login' },
  { path: '/logout', component: LogoutPage, name: 'logout' },
  { path: '/:path(.*)', component: NotFound },
]
