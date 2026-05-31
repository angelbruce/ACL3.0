import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { User } from '@/types'
import { authService } from '@/api'

export const useAuthStore = defineStore(
  'auth',
  () => {
    const user = ref<User | null>(null)
    const accessToken = ref<string | null>(localStorage.getItem('access_token'))
    const refreshToken = ref<string | null>(localStorage.getItem('refresh_token'))
    const loading = ref(false)
    const error = ref<string | null>(null)
    const users = ref<User[]>([])

    const isAuthenticated = computed(() => !!accessToken.value)

    const fetchUsers = async () => {
      if (!accessToken.value) return
      loading.value = true
      try {
        const response = await authService.getUsers()
        users.value = response
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : 'Failed to fetch users'
        error.value = message
        throw err
      } finally {
        loading.value = false
      }
    }

    const login = async (email: string, password: string) => {
      loading.value = true
      error.value = null
      try {
        const response = await authService.login({ email, password })
        accessToken.value = response.access_token
        refreshToken.value = response.refresh_token
        localStorage.setItem('access_token', response.access_token)
        localStorage.setItem('refresh_token', response.refresh_token)

        // Fetch user info
        const userInfo = await authService.getUser(response.user_id)
        user.value = userInfo
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : 'Login failed'
        error.value = message
        throw err
      } finally {
        loading.value = false
      }
    }

    const register = async (email: string, password: string) => {
      loading.value = true
      error.value = null
      try {
        const response = await authService.register({ email, password })
        accessToken.value = response.access_token
        refreshToken.value = response.refresh_token
        localStorage.setItem('access_token', response.access_token)
        localStorage.setItem('refresh_token', response.refresh_token)

        // Fetch user info
        const userInfo = await authService.getUser(response.user_id)
        user.value = userInfo
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : 'Registration failed'
        error.value = message
        throw err
      } finally {
        loading.value = false
      }
    }

    const logout = async () => {
      try {
        await authService.logout()
      } catch {
        // Ignore logout errors
      } finally {
        user.value = null
        accessToken.value = null
        refreshToken.value = null
        localStorage.removeItem('access_token')
        localStorage.removeItem('refresh_token')
      }
    }

    const checkAuth = async () => {
      if (!accessToken.value) return false
      try {
        const tokenData = JSON.parse(atob(accessToken.value.split('.')[1]))
        const exp = tokenData.exp * 1000
        if (Date.now() >= exp) {
          // Token expired
          if (refreshToken.value) {
            try {
              const response = await authService.refresh(refreshToken.value)
              accessToken.value = response.access_token
              refreshToken.value = response.refresh_token
              localStorage.setItem('access_token', response.access_token)
              localStorage.setItem('refresh_token', response.refresh_token)
              return true
            } catch {
              await logout()
              return false
            }
          }
          await logout()
          return false
        }
        return true
      } catch {
        return false
      }
    }

    return {
      user,
      accessToken,
      refreshToken,
      loading,
      error,
      isAuthenticated,
      users,
      fetchUsers,
      login,
      register,
      logout,
      checkAuth,
    }
  },
  {
    persist: {
      key: 'auth',
      paths: ['accessToken', 'refreshToken', 'user'],
    },
  }
)
