<template>
  <div />
</template>
<script setup lang='ts'>
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { toast } from 'vue-sonner'

import { kyClient } from '@/common'
import { API_LOGOUT } from '@/urls'

const router = useRouter()

onMounted(async () => {
  await kyClient.post(API_LOGOUT)
  toast.success('Logout successfully')
  await router.push('/')
})
</script>
