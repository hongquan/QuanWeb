import { RouteRecordRaw, RouteComponent } from 'vue-router'

import NotFound from '@/views/NotFound.vue'
import MainWrap from '@/views/MainWrap.vue'
import SimpleWrap from '@/views/SimpleWrap.vue'
import BookPageWrap from '@/views/BookPageWrap.vue'
import { authRequired } from '@/guards'

const LoginPage = (): Promise<RouteComponent> => import('./views/LoginPage.vue')
const LogoutPage = (): Promise<RouteComponent> => import('./views/LogoutPage.vue')
const BlogPostList = (): Promise<RouteComponent> => import('./views/BlogPostList.vue')
const BlogPostEdit = (): Promise<RouteComponent> => import('./views/BlogPostEdit.vue')
const BlogCategoryList = (): Promise<RouteComponent> => import('./views/BlogCategoryList.vue')
const BlogCategoryEdit = (): Promise<RouteComponent> => import('./views/BlogCategoryEdit.vue')
const PresentationList = (): Promise<RouteComponent> => import('./views/PresentationList.vue')
const PresentationEdit = (): Promise<RouteComponent> => import('./views/PresentationEdit.vue')
const BookAuthorList = (): Promise<RouteComponent> => import('./views/BookAuthorList.vue')
const BookAuthorEdit = (): Promise<RouteComponent> => import('./views/BookAuthorEdit.vue')
const BookList = (): Promise<RouteComponent> => import('./views/BookList.vue')
const BookEdit = (): Promise<RouteComponent> => import('./views/BookEdit.vue')

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
          { path: '', component: PresentationList, name: 'presentation.list' },
          { path: 'new', component: PresentationEdit, name: 'presentation.new' },
          { path: ':id', component: PresentationEdit, name: 'presentation.edit', props: true },
        ],
      },
      {
        path: 'books', component: BookPageWrap, name: 'books', redirect: () => ({ name: 'book.list' }),
        children: [
          {
            path: 'authors', component: SimpleWrap, redirect: () => ({ name: 'book-author.list' }),
            children: [
              { path: '', component: BookAuthorList, name: 'book-author.list' },
              { path: 'new', component: BookAuthorEdit, name: 'book-author.new' },
              { path: ':id', component: BookAuthorEdit, name: 'book-author.edit', props: true },
            ],
          },
          {
            path: 'books', component: SimpleWrap, redirect: () => ({ name: 'book.list' }),
            children: [
              { path: '', component: BookList, name: 'book.list' },
              { path: 'new', component: BookEdit, name: 'book.new' },
              { path: ':id', component: BookEdit, name: 'book.edit', props: true },
            ],
          },
        ],
      },
    ],
  },
  { path: '/login', component: LoginPage, name: 'login' },
  { path: '/logout', component: LogoutPage, name: 'logout' },
  { path: '/:path(.*)', component: NotFound },
]
