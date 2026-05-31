<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Plus, Building2, Trash2, Edit2, Loader2, FolderOpen } from 'lucide-vue-next'
import { useAdminStore } from '@/stores'
import type { Department } from '@/types'

const adminStore = useAdminStore()

const showForm = ref(false)
const editingDepartment = ref<Department | null>(null)
const deletingId = ref<number | null>(null)

const formData = ref<{
  name: string
  parent_id: number | null
  description: string
}>({
  name: '',
  parent_id: null,
  description: '',
})

onMounted(async () => {
  await adminStore.loadDepartments()
})

const openForm = (department?: Department) => {
  if (department) {
    editingDepartment.value = department
    formData.value = {
      name: department.name,
      parent_id: department.parent_id || null,
      description: department.description || '',
    }
  } else {
    editingDepartment.value = null
    formData.value = {
      name: '',
      parent_id: null,
      description: '',
    }
  }
  showForm.value = true
}

const closeForm = () => {
  showForm.value = false
  editingDepartment.value = null
}

const handleSubmit = async () => {
  if (!formData.value.name.trim()) {
    alert('请输入部门名称')
    return
  }
  try {
    const request = {
      name: formData.value.name,
      parent_id: formData.value.parent_id,
      description: formData.value.description || undefined,
    }
    if (editingDepartment.value) {
      await adminStore.updateDepartment(editingDepartment.value.id, request)
    } else {
      await adminStore.createDepartment(request)
    }
    closeForm()
  } catch (error) {
    console.error('保存失败:', error)
  }
}

const deleteDepartment = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个部门吗?')) {
    deletingId.value = id
    try {
      await adminStore.deleteDepartment(id)
    } finally {
      deletingId.value = null
    }
  }
}

const getParentName = (parentId: number | undefined) => {
  if (!parentId) return ''
  const parent = adminStore.departments.find(d => d.id === parentId)
  return parent?.name || ''
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

const getChildren = (parentId: number | null) => {
  return adminStore.departments.filter(d => d.parent_id === parentId)
}

defineProps<{
  departments?: Department[]
}>()
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">部门管理</h1>
        <p class="page-subtitle">department management</p>
      </div>
      <button @click="openForm()" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        添加部门
      </button>
    </div>

    <div v-if="adminStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="adminStore.departments.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <FolderOpen class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无部门</h3>
      <p class="text-surface-400 mb-6">创建部门来组织人员结构</p>
      <button @click="openForm()" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 创建部门
      </button>
    </div>

    <div v-else class="space-y-4 max-w-2xl">
      <template v-for="department in getChildren(null)" :key="department.id">
        <div class="card p-4 group">
          <div class="flex items-start justify-between">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-cyan-100 to-blue-100 flex items-center justify-center">
                <Building2 class="w-5 h-5 text-cyan-500" />
              </div>
              <div class="flex-1 min-w-0">
                <h3 class="font-semibold text-surface-800 truncate" :title="department.name">
                  {{ department.name }}
                </h3>
                <p v-if="department.parent_id" class="text-xs text-surface-400 mt-0.5">
                  上级: {{ getParentName(department.parent_id) }}
                </p>
                <p v-if="department.description" class="text-xs text-surface-500 mt-1 line-clamp-2">
                  {{ department.description }}
                </p>
                <p class="text-xs text-surface-400 mt-1">{{ formatDate(department.created_at) }}</p>
              </div>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button @click.stop="openForm(department)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
                <Edit2 class="w-4 h-4" />
              </button>
              <button @click.stop="deleteDepartment(department.id, $event)" :disabled="deletingId === department.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
                <Loader2 v-if="deletingId === department.id" class="w-4 h-4 animate-spin" />
                <Trash2 v-else class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- 递归子部门 -->
          <div v-if="getChildren(department.id).length > 0" class="mt-3 pl-4 border-l-2 border-surface-100">
            <template v-for="child in getChildren(department.id)" :key="child.id">
              <div class="card p-4 group mt-4">
                <div class="flex items-start justify-between">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-cyan-100 to-blue-100 flex items-center justify-center">
                      <Building2 class="w-5 h-5 text-cyan-500" />
                    </div>
                    <div class="flex-1 min-w-0">
                      <h3 class="font-semibold text-surface-800 truncate" :title="child.name">
                        {{ child.name }}
                      </h3>
                      <p v-if="child.parent_id" class="text-xs text-surface-400 mt-0.5">
                        上级: {{ getParentName(child.parent_id) }}
                      </p>
                      <p v-if="child.description" class="text-xs text-surface-500 mt-1 line-clamp-2">
                        {{ child.description }}
                      </p>
                      <p class="text-xs text-surface-400 mt-1">{{ formatDate(child.created_at) }}</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button @click.stop="openForm(child)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
                      <Edit2 class="w-4 h-4" />
                    </button>
                    <button @click.stop="deleteDepartment(child.id, $event)" :disabled="deletingId === child.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
                      <Loader2 v-if="deletingId === child.id" class="w-4 h-4 animate-spin" />
                      <Trash2 v-else class="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <!-- 第三层子部门 -->
                <div v-if="getChildren(child.id).length > 0" class="mt-3 pl-4 border-l-2 border-surface-100">
                  <template v-for="grandchild in getChildren(child.id)" :key="grandchild.id">
                    <div class="card p-4 group mt-4">
                      <div class="flex items-start justify-between">
                        <div class="flex items-center gap-3">
                          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-cyan-100 to-blue-100 flex items-center justify-center">
                            <Building2 class="w-5 h-5 text-cyan-500" />
                          </div>
                          <div class="flex-1 min-w-0">
                            <h3 class="font-semibold text-surface-800 truncate" :title="grandchild.name">
                              {{ grandchild.name }}
                            </h3>
                            <p v-if="grandchild.parent_id" class="text-xs text-surface-400 mt-0.5">
                              上级: {{ getParentName(grandchild.parent_id) }}
                            </p>
                            <p v-if="grandchild.description" class="text-xs text-surface-500 mt-1 line-clamp-2">
                              {{ grandchild.description }}
                            </p>
                            <p class="text-xs text-surface-400 mt-1">{{ formatDate(grandchild.created_at) }}</p>
                          </div>
                        </div>
                        <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button @click.stop="openForm(grandchild)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
                            <Edit2 class="w-4 h-4" />
                          </button>
                          <button @click.stop="deleteDepartment(grandchild.id, $event)" :disabled="deletingId === grandchild.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
                            <Loader2 v-if="deletingId === grandchild.id" class="w-4 h-4 animate-spin" />
                            <Trash2 v-else class="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                  </template>
                </div>
              </div>
            </template>
          </div>
        </div>
      </template>
    </div>

    <!-- Form dialog -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 bg-black/30 flex items-center justify-center z-50 p-4" @click.self="closeForm">
        <div class="card p-6 w-full max-w-md animate-fade-in">
          <h2 class="text-lg font-semibold mb-4 text-surface-800">{{ editingDepartment ? '编辑部门' : '添加部门' }}</h2>
          <form @submit.prevent="handleSubmit" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">部门名称 <span class="text-red-500">*</span></label>
              <input v-model="formData.name" type="text" placeholder="输入部门名称" class="input-base w-full" />
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">上级部门</label>
              <select v-model="formData.parent_id" class="input-base w-full">
                <option :value="null">无（顶级部门）</option>
                <option v-for="dept in adminStore.departments" :key="dept.id" :value="dept.id">
                  {{ dept.name }}
                </option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">描述</label>
              <textarea v-model="formData.description" rows="2" placeholder="输入部门描述（可选）" class="input-base w-full"></textarea>
            </div>
            <div class="flex gap-3 pt-2">
              <button type="button" @click="closeForm" class="btn btn-outline flex-1 justify-center">取消</button>
              <button type="submit" class="btn btn-primary flex-1 justify-center">{{ editingDepartment ? '保存' : '添加' }}</button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>
  </div>
</template>
