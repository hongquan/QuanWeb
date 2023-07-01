import { defineStore } from 'pinia'

import { User } from '@/models/user'

export const useStore = defineStore('quanweb', {
  state: () => ({
    user: null as User | null,
  }),
  getters: {
    isAuthenticated(state) {
      return Boolean(state.user)
    },
  },
  persist: true,
})
