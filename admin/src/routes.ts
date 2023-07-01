import { RouteRecordRaw, RouteComponent } from 'vue-router'

import NotFound from '@/views/NotFound.vue'
import MainWrap from '@/views/MainWrap.vue'

const LoginPage = (): Promise<RouteComponent> => import('./views/LoginPage.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: MainWrap,
  },
  { path: '/login', component: LoginPage, name: 'login' },
  { path: '/:path(.*)', component: NotFound },
]
