<template>
  <div class="min-h-screen bg-gray-50 p-8">
    <div class="max-w-4xl mx-auto">
      <div class="bg-white rounded-lg shadow-md p-8">
        <div class="flex items-center mb-6">
          <Settings class="w-8 h-8 text-blue-600 mr-3" />
          <h1 class="text-2xl font-bold text-gray-800">系统初始化</h1>
        </div>

        <p class="text-gray-600 mb-8">
          首次使用系统时，请按顺序执行以下初始化操作。这些操作将创建必要的角色、菜单和权限。
        </p>

        <div class="space-y-4">
          <div class="border rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="font-semibold text-gray-800">1. 初始化超级管理员角色</h3>
                <p class="text-sm text-gray-500 mt-1">创建拥有所有权限的超级管理员角色</p>
              </div>
              <button
                @click="initSuperAdmin"
                :disabled="loading.step1"
                class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300"
              >
                {{ loading.step1 ? '执行中...' : '执行' }}
              </button>
            </div>
            <div v-if="success.step1" class="mt-3 text-green-600 text-sm flex items-center">
              <Check class="w-4 h-4 mr-1" /> 超级管理员角色已创建
            </div>
          </div>

          <div class="border rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="font-semibold text-gray-800">2. 初始化菜单</h3>
                <p class="text-sm text-gray-500 mt-1">创建系统管理、Agent管理、会话管理等默认菜单</p>
              </div>
              <button
                @click="initMenus"
                :disabled="loading.step2"
                class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300"
              >
                {{ loading.step2 ? '执行中...' : '执行' }}
              </button>
            </div>
            <div v-if="success.step2" class="mt-3 text-green-600 text-sm flex items-center">
              <Check class="w-4 h-4 mr-1" /> 菜单已创建
            </div>
          </div>

          <div class="border rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="font-semibold text-gray-800">3. 初始化权限</h3>
                <p class="text-sm text-gray-500 mt-1">为所有菜单创建访问、创建、编辑、删除权限</p>
              </div>
              <button
                @click="initPermissions"
                :disabled="loading.step3"
                class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300"
              >
                {{ loading.step3 ? '执行中...' : '执行' }}
              </button>
            </div>
            <div v-if="success.step3" class="mt-3 text-green-600 text-sm flex items-center">
              <Check class="w-4 h-4 mr-1" /> 权限已创建
            </div>
          </div>

          <div class="border rounded-lg p-4">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="font-semibold text-gray-800">4. 分配所有权限给超级管理员</h3>
                <p class="text-sm text-gray-500 mt-1">将所有权限分配给超级管理员角色</p>
              </div>
              <button
                @click="initSuperAdminAll"
                :disabled="loading.step4"
                class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300"
              >
                {{ loading.step4 ? '执行中...' : '执行' }}
              </button>
            </div>
            <div v-if="success.step4" class="mt-3 text-green-600 text-sm flex items-center">
              <Check class="w-4 h-4 mr-1" /> 权限已分配
            </div>
          </div>
        </div>

        <div v-if="allSuccess" class="mt-8 p-4 bg-green-50 border border-green-200 rounded-lg">
          <div class="flex items-center text-green-800">
            <CheckCircle class="w-6 h-6 mr-2" />
            <span class="font-semibold">系统初始化完成！</span>
          </div>
          <p class="text-green-700 mt-2">
            现在您可以将用户设为超级管理员，或开始创建其他角色和分配权限。
          </p>
          <router-link
            to="/admin/personnel"
            class="inline-block mt-3 px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
          >
            前往人员管理
          </router-link>
        </div>

        <div v-if="error" class="mt-6 p-4 bg-red-50 border border-red-200 rounded-lg">
          <p class="text-red-700">{{ error }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAdminStore } from '@/stores'
import { Settings, Check, CheckCircle } from 'lucide-vue-next'

const adminStore = useAdminStore()

const loading = ref({
  step1: false,
  step2: false,
  step3: false,
  step4: false,
})

const success = ref({
  step1: false,
  step2: false,
  step3: false,
  step4: false,
})

const error = ref('')

const allSuccess = computed(() => {
  return success.value.step1 && success.value.step2 && success.value.step3 && success.value.step4
})

const initSuperAdmin = async () => {
  loading.value.step1 = true
  error.value = ''
  try {
    await adminStore.initSuperAdmin()
    success.value.step1 = true
  } catch (e: any) {
    error.value = e.message || '初始化失败'
  } finally {
    loading.value.step1 = false
  }
}

const initMenus = async () => {
  loading.value.step2 = true
  error.value = ''
  try {
    await adminStore.initMenus()
    success.value.step2 = true
  } catch (e: any) {
    error.value = e.message || '初始化失败'
  } finally {
    loading.value.step2 = false
  }
}

const initPermissions = async () => {
  loading.value.step3 = true
  error.value = ''
  try {
    await adminStore.initPermissions()
    success.value.step3 = true
  } catch (e: any) {
    error.value = e.message || '初始化失败'
  } finally {
    loading.value.step3 = false
  }
}

const initSuperAdminAll = async () => {
  loading.value.step4 = true
  error.value = ''
  try {
    await adminStore.initSuperAdminAll()
    success.value.step4 = true
  } catch (e: any) {
    error.value = e.message || '初始化失败'
  } finally {
    loading.value.step4 = false
  }
}
</script>
