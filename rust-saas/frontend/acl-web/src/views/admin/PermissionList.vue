<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { Plus, Key, Trash2, Loader2, Menu } from 'lucide-vue-next'
import { useAdminStore } from '@/stores'
import type { Permission } from '@/types'

const adminStore = useAdminStore()

const showForm = ref(false)
const deletingId = ref<number | null>(null)

const formData = ref<{
  menu_id: number
  name: string
  description: string
}>({
  menu_id: 0,
  name: '',
  description: '',
})

onMounted(async () => {
  await adminStore.loadMenus()
  await adminStore.loadPermissions()
})

const closeForm = () => {
  showForm.value = false
  formData.value = {
    menu_id: 0,
    name: '',
    description: '',
  }
}

const handleSubmit = async () => {
  if (!formData.value.name.trim()) {
    alert('请输入权限名称')
    return
  }
  if (!formData.value.menu_id) {
    alert('请选择所属菜单')
    return
  }
  try {
    const request = {
      menu_id: formData.value.menu_id,
      name: formData.value.name,
      description: formData.value.description || undefined,
    }
    await adminStore.createPermission(request)
    closeForm()
  } catch (error) {
    console.error('保存失败:', error)
  }
}

const deletePermission = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个权限吗?')) {
    deletingId.value = id
    try {
      await adminStore.deletePermission(id)
    } finally {
      deletingId.value = null
    }
  }
}

const permissionsByMenu = computed(() => {
  const grouped: Record<number, Permission[]> = {}
  adminStore.permissions.forEach(p => {
    if (!grouped[p.menu_id]) {
      grouped[p.menu_id] = []
    }
    grouped[p.menu_id].push(p)
  })
  return grouped
})

const getMenuName = (menuId: string) => {
  const menu = adminStore.menus.find(m => m.id+'' === menuId)
  return menu?.name || '未知菜单'
}

const getMenuIcon = (menuId: string) => {
  const menu = adminStore.menus.find(m => m.id+'' === menuId)
  return menu?.icon || 'Key'
}
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">权限管理</h1>
        <p class="page-subtitle">permission management</p>
      </div>
      <button @click="showForm = true" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        添加权限
      </button>
    </div>

    <div v-if="adminStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="adminStore.permissions.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <Key class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无权限</h3>
      <p class="text-surface-400 mb-6">创建权限来控制功能访问</p>
      <button @click="showForm = true" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 创建权限
      </button>
    </div>

    <div v-else class="space-y-6">
      <div v-for="(permissions, menuId) in permissionsByMenu" :key="menuId" class="card">
        <div class="p-4 border-b border-surface-100">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-100 to-purple-100 flex items-center justify-center">
              <Menu class="w-5 h-5 text-indigo-500" />
            </div>
            <div>
              <h3 class="font-semibold text-surface-800">{{ getMenuName(menuId) }}</h3>
              <p class="text-xs text-surface-400">{{ permissions.length }} 个权限</p>
            </div>
          </div>
        </div>
        <div class="p-4">
          <div class="grid gap-3">
            <div
              v-for="permission in permissions"
              :key="permission.id"
              class="flex items-center justify-between p-3 bg-surface-50 rounded-lg group"
            >
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-primary-50 flex items-center justify-center">
                  <Key class="w-4 h-4 text-primary-500" />
                </div>
                <div>
                  <h4 class="font-medium text-surface-700">{{ permission.name }}</h4>
                  <p v-if="permission.description" class="text-xs text-surface-400">{{ permission.description }}</p>
                </div>
              </div>
              <button
                @click="deletePermission(permission.id, $event)"
                :disabled="deletingId === permission.id"
                class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors opacity-0 group-hover:opacity-100"
              >
                <Loader2 v-if="deletingId === permission.id" class="w-4 h-4 animate-spin" />
                <Trash2 v-else class="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Form dialog -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 bg-black/30 flex items-center justify-center z-50 p-4" @click.self="closeForm">
        <div class="card p-6 w-full max-w-md animate-fade-in">
          <h2 class="text-lg font-semibold mb-4 text-surface-800">添加权限</h2>
          <form @submit.prevent="handleSubmit" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">所属菜单 <span class="text-red-500">*</span></label>
              <select v-model="formData.menu_id" class="input-base w-full">
                <option :value="0">请选择菜单</option>
                <option v-for="menu in adminStore.menus" :key="menu.id" :value="menu.id">
                  {{ menu.name }}
                </option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">权限名称 <span class="text-red-500">*</span></label>
              <input v-model="formData.name" type="text" placeholder="输入权限名称" class="input-base w-full" />
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">描述</label>
              <textarea v-model="formData.description" rows="2" placeholder="输入权限描述（可选）" class="input-base w-full"></textarea>
            </div>
            <div class="flex gap-3 pt-2">
              <button type="button" @click="closeForm" class="btn btn-outline flex-1 justify-center">取消</button>
              <button type="submit" class="btn btn-primary flex-1 justify-center">添加</button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>
  </div>
</template>
