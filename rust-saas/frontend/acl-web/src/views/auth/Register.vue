<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Mail, Lock, ArrowRight, AlertCircle, User } from 'lucide-vue-next'
import { useAuthStore } from '@/stores'

const router = useRouter()
const authStore = useAuthStore()

const email = ref('')
const name = ref('')
const password = ref('')
const confirmPassword = ref('')
const loading = ref(false)
const error = ref('')

const handleSubmit = async () => {
  if (!email.value || !password.value || !confirmPassword.value) {
    error.value = '请填写所有字段'
    return
  }

  if (password.value !== confirmPassword.value) {
    error.value = '两次密码输入不一致'
    return
  }

  if (password.value.length < 6) {
    error.value = '密码长度至少为 6 位'
    return
  }

  loading.value = true
  error.value = ''

  try {
    await authStore.register(email.value, password.value)
    router.push('/sessions')
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : '注册失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center p-4 bg-surface-50">
    <div class="w-full max-w-md">
      <div class="text-center mb-8">
        <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-gradient-to-br from-primary-500 to-accent-500 mb-4 shadow-[0_4px_20px_rgba(12,142,233,0.2)]">
          <span class="text-2xl font-bold text-white">A</span>
        </div>
        <h1 class="text-3xl font-bold gradient-text">ACL</h1>
        <p class="text-surface-400 mt-2">Agent Control Layer</p>
      </div>

      <div class="card p-8 shadow-card">
        <h2 class="text-xl font-semibold mb-6 text-surface-800">注册</h2>

        <div v-if="error" class="mb-4 p-3 rounded-lg bg-red-50 border border-red-200 text-red-500 flex items-center gap-2">
          <AlertCircle class="w-4 h-4 flex-shrink-0" />
          <span class="text-sm">{{ error }}</span>
        </div>

        <form @submit.prevent="handleSubmit" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">用户名</label>
            <div class="relative input-wrapper">
              <User class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-surface-400" />
              <input
                v-model="name"
                type="text"
                placeholder="你的用户名"
                class="w-full py-3 pr-4 bg-white border border-surface-200 rounded-lg focus:outline-none focus:border-primary-500"
                style="padding-left: 52px;"
              />
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">邮箱</label>
            <div class="relative input-wrapper">
              <Mail class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-surface-400" />
              <input
                v-model="email"
                type="email"
                placeholder="your@email.com"
                class="w-full py-3 pr-4 bg-white border border-surface-200 rounded-lg focus:outline-none focus:border-primary-500"
                style="padding-left: 52px;"
              />
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">密码</label>
            <div class="relative input-wrapper">
              <Lock class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-surface-400" />
              <input
                v-model="password"
                type="password"
                placeholder="········"
                class="w-full py-3 pr-4 bg-white border border-surface-200 rounded-lg focus:outline-none focus:border-primary-500"
                style="padding-left: 52px;"
              />
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">确认密码</label>
            <div class="relative input-wrapper">
              <Lock class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-surface-400" />
              <input
                v-model="confirmPassword"
                type="password"
                placeholder="········"
                class="w-full py-3 pr-4 bg-white border border-surface-200 rounded-lg focus:outline-none focus:border-primary-500"
                style="padding-left: 52px;"
              />
            </div>
          </div>

          <button
            type="submit"
            :disabled="loading"
            class="w-full py-3 px-4 bg-gradient-to-r from-primary-500 to-accent-500 rounded-lg font-medium flex items-center justify-center gap-2 text-white hover:from-primary-600 hover:to-accent-600 transition-all disabled:opacity-50"
          >
            <span v-if="loading">注册中...</span>
            <template v-else>
              注册
              <ArrowRight class="w-4 h-4" />
            </template>
          </button>
        </form>

        <div class="mt-6 text-center">
          <p class="text-surface-400 text-sm">
            已有账户?
            <router-link to="/login" class="text-primary-600 hover:text-primary-500 font-medium">
              登录
            </router-link>
          </p>
        </div>
      </div>

      <p class="text-center text-surface-300 text-sm mt-8">
        ACL Platform
      </p>
    </div>
  </div>
</template>