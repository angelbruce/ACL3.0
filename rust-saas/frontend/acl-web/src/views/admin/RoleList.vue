<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Plus, Shield, Trash2, Edit2, Loader2, Crown } from 'lucide-vue-next'
import { useAdminStore } from '@/stores'
import type { Role } from '@/types'

const adminStore = useAdminStore()

const showForm = ref(false)
const editingRole = ref<Role | null>(null)
const deletingId = ref<number | null>(null)
const showPermissionModal = ref(false)
const selectedRole = ref<Role | null>(null)

const formData = ref<{
  name: string
  description: string
  is_super_admin: boolean
}>({
  name: '',
  description: '',
  is_super_admin: false,
})

onMounted(async () => {
  await adminStore.loadRoles()
})

const openForm = (role?: Role) => {
  if (role) {
    editingRole.value = role
    formData.value = {
      name: role.name,
      description: role.description || '',
      is_super_admin: role.is_super_admin,
    }
  } else {
    editingRole.value = null
    formData.value = {
      name: '',
      description: '',
      is_super_admin: false,
    }
  }
  showForm.value = true
}

const closeForm = () => {
  showForm.value = false
  editingRole.value = null
}

const handleSubmit = async () => {
  if (!formData.value.name.trim()) {
    alert('请输入角色名称')
    return
  }
  try {
    const request = {
      name: formData.value.name,
      description: formData.value.description || undefined,
      is_super_admin: formData.value.is_super_admin,
    }
    if (editingRole.value) {
      await adminStore.updateRole(editingRole.value.id, request)
    } else {
      await adminStore.createRole(request)
    }
    closeForm()
  } catch (error) {
    console.error('保存失败:', error)
  }
}

const deleteRole = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个角色吗?')) {
    deletingId.value = id
    try {
      await adminStore.deleteRole(id)
    } finally {
      deletingId.value = null
    }
  }
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

const openPermissionModal = (role: Role) => {
  selectedRole.value = role
  showPermissionModal.value = true
}

const closePermissionModal = () => {
  showPermissionModal.value = false
  selectedRole.value = null
}
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">角色管理</h1>
        <p class="page-subtitle">role management</p>
      </div>
      <button @click="openForm()" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        添加角色
      </button>
    </div>

    <div v-if="adminStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="adminStore.roles.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <Shield class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无角色</h3>
      <p class="text-surface-400 mb-6">创建角色来管理权限</p>
      <button @click="openForm()" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 创建角色
      </button>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <div
        v-for="role in adminStore.roles"
        :key="role.id"
        class="card p-5 group"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 rounded-xl flex items-center justify-center" :class="role.is_super_admin ? 'bg-gradient-to-br from-amber-100 to-orange-100' : 'bg-gradient-to-br from-violet-100 to-purple-100'">
              <Crown v-if="role.is_super_admin" class="w-6 h-6 text-amber-500" />
              <Shield v-else class="w-6 h-6 text-violet-500" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3 class="font-semibold text-base text-surface-800 truncate" :title="role.name">{{ role.name }}</h3>
                <span v-if="role.is_super_admin" class="tag tag-amber text-xs">超级管理员</span>
              </div>
              <p class="text-xs text-surface-400 mt-0.5">{{ formatDate(role.created_at) }}</p>
            </div>
          </div>
          <div class="flex items-center gap-1">
            <button @click.stop="openForm(role)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
              <Edit2 class="w-4 h-4" />
            </button>
            <button @click.stop="deleteRole(role.id, $event)" :disabled="deletingId === role.id || role.is_super_admin" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors" :class="{ 'opacity-50 cursor-not-allowed': role.is_super_admin }">
              <Loader2 v-if="deletingId === role.id" class="w-4 h-4 animate-spin" />
              <Trash2 v-else class="w-4 h-4" />
            </button>
          </div>
        </div>

        <div>
          <p class="text-xs text-surface-400 mb-1">描述</p>
          <p v-if="role.description" class="text-sm text-surface-600">{{ role.description }}</p>
          <p v-else class="text-sm text-surface-300 italic">暂无描述</p>
        </div>

        <div class="mt-3 pt-3 border-t border-surface-100">
          <button @click.stop="openPermissionModal(role)" class="btn btn-outline w-full text-sm">
            管理权限
          </button>
        </div>
      </div>
    </div>

    <!-- Form dialog -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 bg-black/30 flex items-center justify-center z-50 p-4" @click.self="closeForm">
        <div class="card p-6 w-full max-w-md animate-fade-in">
          <h2 class="text-lg font-semibold mb-4 text-surface-800">{{ editingRole ? '编辑角色' : '添加角色' }}</h2>
          <form @submit.prevent="handleSubmit" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">角色名称 <span class="text-red-500">*</span></label>
              <input v-model="formData.name" type="text" placeholder="输入角色名称" class="input-base w-full" />
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">描述</label>
              <textarea v-model="formData.description" rows="2" placeholder="输入角色描述（可选）" class="input-base w-full"></textarea>
            </div>
            <div class="flex items-center gap-2">
              <input v-model="formData.is_super_admin" type="checkbox" id="is_super_admin" class="w-4 h-4 rounded border-surface-300 text-primary-500" />
              <label for="is_super_admin" class="text-sm text-surface-700">设为超级管理员</label>
            </div>
            <div class="flex gap-3 pt-2">
              <button type="button" @click="closeForm" class="btn btn-outline flex-1 justify-center">取消</button>
              <button type="submit" class="btn btn-primary flex-1 justify-center">{{ editingRole ? '保存' : '添加' }}</button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>

    <!-- Permission modal -->
    <Teleport to="body">
      <div v-if="showPermissionModal && selectedRole" class="fixed inset-0 bg-black/30 flex items-center justify-center z-50 p-4" @click.self="closePermissionModal">
        <div class="card p-6 w-full max-w-lg animate-fade-in max-h-[80vh] overflow-y-auto">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-lg font-semibold text-surface-800">{{ selectedRole.name }} - 权限管理</h2>
            <button @click="closePermissionModal" class="p-2 hover:bg-surface-100 rounded-lg">
              <span class="text-surface-400 text-xl">&times;</span>
            </button>
          </div>
          
          <div v-if="selectedRole.is_super_admin" class="text-center py-8">
            <Crown class="w-12 h-12 text-amber-400 mx-auto mb-3" />
            <p class="text-surface-600">超级管理员拥有所有权限</p>
          </div>
          
          <div v-else class="space-y-4">
            <p class="text-sm text-surface-500">选择此角色拥有的权限：</p>
            <div class="space-y-2">
              <label v-for="permission in adminStore.permissions" :key="permission.id" class="flex items-center gap-3 p-3 bg-surface-50 rounded-lg cursor-pointer hover:bg-surface-100">
                <input type="checkbox" class="w-4 h-4 rounded border-surface-300 text-primary-500" />
                <div>
                  <p class="font-medium text-surface-700">{{ permission.name }}</p>
                  <p class="text-xs text-surface-400">{{ permission.description }}</p>
                </div>
              </label>
            </div>
          </div>

          <div class="flex gap-3 mt-6">
            <button @click="closePermissionModal" class="btn btn-outline flex-1 justify-center">取消</button>
            <button class="btn btn-primary flex-1 justify-center">保存权限</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
