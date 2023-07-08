import { RouteLocationNormalized } from 'vue-router'

import { kyClient } from './common'
import { useStore } from './stores'
import { API_GET_ME } from './urls'
import { User, UserSchema } from './models/user'

const WHITELIST = ['login', 'not-found']

async function getMe(): Promise<User | null> {
  try {
    const resp = await kyClient.get(API_GET_ME).json()
    return UserSchema.parse(resp)
  } catch (e) {
    console.info('Failed to get user info', e)
  }
  return null
}

export async function authRequired(to: RouteLocationNormalized) {
  const accessedRouteName = to.name?.toString() || ''
  if (WHITELIST.includes(accessedRouteName)) {
    return true
  }
  const store = useStore()
  if (store.user) {
    // Check if this login session expired
    const user = await getMe()
    if (user) {
      store.user = user
      return true
    }
    store.$reset()
    return { name: 'login', query: { attempt: to.fullPath } }
  }
  const user = await getMe()
  if (user) {
    store.user = user
    return true
  }
  return { name: 'login', query: { attempt: to.fullPath } }
}
