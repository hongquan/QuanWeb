import { RouteRecordRaw, RouteComponent } from 'vue-router'

import NotFound from '@/views/NotFound.vue'
import MainWrap from '@/views/MainWrap.vue'

const LoginPage = (): Promise<RouteComponent> => import('./views/LoginPage.vue')
const BlogPostListPage = (): Promise<RouteComponent> => import('./views/BlogPostListPage.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: MainWrap,
    children: [
      { path: 'posts', component: BlogPostListPage, name: 'blogposts' },
    ],
  },
  { path: '/login', component: LoginPage, name: 'login' },
  { path: '/:path(.*)', component: NotFound },
]
