<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Plus, Menu, Trash2, Edit2, Loader2, ChevronRight } from 'lucide-vue-next'
import { useAdminStore } from '@/stores'
import type { Menu as MenuType } from '@/types'

const adminStore = useAdminStore()

const showForm = ref(false)
const editingMenu = ref<MenuType | null>(null)
const deletingId = ref<number | null>(null)

const formData = ref<{
  name: string
  path: string
  parent_id: number | null
  icon: string
  sort_order: number
}>({
  name: '',
  path: '',
  parent_id: null,
  icon: '',
  sort_order: 0,
})

onMounted(async () => {
  await adminStore.loadMenus()
})

const openForm = (menu?: MenuType) => {
  if (menu) {
    editingMenu.value = menu
    formData.value = {
      name: menu.name,
      path: menu.path || '',
      parent_id: menu.parent_id || null,
      icon: menu.icon || '',
      sort_order: menu.sort_order || 0,
    }
  } else {
    editingMenu.value = null
    formData.value = {
      name: '',
      path: '',
      parent_id: null,
      icon: '',
      sort_order: 0,
    }
  }
  showForm.value = true
}

const closeForm = () => {
  showForm.value = false
  editingMenu.value = null
}

const handleSubmit = async () => {
  if (!formData.value.name.trim()) {
    alert('请输入菜单名称')
    return
  }
  try {
    const request = {
      name: formData.value.name,
      path: formData.value.path || undefined,
      parent_id: formData.value.parent_id,
      icon: formData.value.icon || undefined,
      sort_order: formData.value.sort_order,
    }
    if (editingMenu.value) {
      await adminStore.updateMenu(editingMenu.value.id, request)
    } else {
      await adminStore.createMenu(request)
    }
    closeForm()
  } catch (error) {
    console.error('保存失败:', error)
  }
}

const deleteMenu = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个菜单吗?')) {
    deletingId.value = id
    try {
      await adminStore.deleteMenu(id)
    } finally {
      deletingId.value = null
    }
  }
}

const getParentName = (parentId: number | undefined) => {
  if (!parentId) return ''
  const parent = adminStore.menus.find(m => m.id === parentId)
  return parent?.name || ''
}

const getChildren = (parentId: number | null) => {
  return adminStore.menus.filter(m => m.parent_id === parentId).sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0))
}
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">菜单管理</h1>
        <p class="page-subtitle">menu management</p>
      </div>
      <button @click="openForm()" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        添加菜单
      </button>
    </div>

    <div v-if="adminStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="adminStore.menus.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <Menu class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无菜单</h3>
      <p class="text-surface-400 mb-6">创建菜单来定义功能入口</p>
      <button @click="openForm()" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 创建菜单
      </button>
    </div>

    <div v-else class="space-y-4 max-w-2xl">
      <template v-for="menu in getChildren(null)" :key="menu.id">
        <div class="card p-4 group">
          <div class="flex items-start justify-between">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-100 to-green-100 flex items-center justify-center">
                <Menu class="w-5 h-5 text-emerald-500" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <h3 class="font-semibold text-surface-800 truncate" :title="menu.name">{{ menu.name }}</h3>
                  <ChevronRight v-if="getChildren(menu.id).length > 0" class="w-4 h-4 text-surface-400" />
                </div>
                <p v-if="menu.parent_id" class="text-xs text-surface-400 mt-0.5">上级: {{ getParentName(menu.parent_id) }}</p>
                <p v-if="menu.path" class="text-xs text-surface-500 mt-1 font-mono">{{ menu.path }}</p>
              </div>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button @click.stop="openForm(menu)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
                <Edit2 class="w-4 h-4" />
              </button>
              <button @click.stop="deleteMenu(menu.id, $event)" :disabled="deletingId === menu.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
                <Loader2 v-if="deletingId === menu.id" class="w-4 h-4 animate-spin" />
                <Trash2 v-else class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- 第一层子菜单 -->
          <div v-if="getChildren(menu.id).length > 0" class="mt-3 pl-4 border-l-2 border-surface-100">
            <template v-for="child in getChildren(menu.id)" :key="child.id">
              <div class="card p-4 group mt-4">
                <div class="flex items-start justify-between">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-100 to-green-100 flex items-center justify-center">
                      <Menu class="w-5 h-5 text-emerald-500" />
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <h3 class="font-semibold text-surface-800 truncate" :title="child.name">{{ child.name }}</h3>
                        <ChevronRight v-if="getChildren(child.id).length > 0" class="w-4 h-4 text-surface-400" />
                      </div>
                      <p v-if="child.parent_id" class="text-xs text-surface-400 mt-0.5">上级: {{ getParentName(child.parent_id) }}</p>
                      <p v-if="child.path" class="text-xs text-surface-500 mt-1 font-mono">{{ child.path }}</p>
                    </div>
                  </div>
                  <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button @click.stop="openForm(child)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
                      <Edit2 class="w-4 h-4" />
                    </button>
                    <button @click.stop="deleteMenu(child.id, $event)" :disabled="deletingId === child.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
                      <Loader2 v-if="deletingId === child.id" class="w-4 h-4 animate-spin" />
                      <Trash2 v-else class="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <!-- 第二层子菜单 -->
                <div v-if="getChildren(child.id).length > 0" class="mt-3 pl-4 border-l-2 border-surface-100">
                  <template v-for="grandchild in getChildren(child.id)" :key="grandchild.id">
                    <div class="card p-4 group mt-4">
                      <div class="flex items-start justify-between">
                        <div class="flex items-center gap-3">
                          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-100 to-green-100 flex items-center justify-center">
                            <Menu class="w-5 h-5 text-emerald-500" />
                          </div>
                          <div class="flex-1 min-w-0">
                            <h3 class="font-semibold text-surface-800 truncate" :title="grandchild.name">{{ grandchild.name }}</h3>
                            <p v-if="grandchild.parent_id" class="text-xs text-surface-400 mt-0.5">上级: {{ getParentName(grandchild.parent_id) }}</p>
                            <p v-if="grandchild.path" class="text-xs text-surface-500 mt-1 font-mono">{{ grandchild.path }}</p>
                          </div>
                        </div>
                        <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button @click.stop="openForm(grandchild)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
                            <Edit2 class="w-4 h-4" />
                          </button>
                          <button @click.stop="deleteMenu(grandchild.id, $event)" :disabled="deletingId === grandchild.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
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
          <h2 class="text-lg font-semibold mb-4 text-surface-800">{{ editingMenu ? '编辑菜单' : '添加菜单' }}</h2>
          <form @submit.prevent="handleSubmit" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">菜单名称 <span class="text-red-500">*</span></label>
              <input v-model="formData.name" type="text" placeholder="输入菜单名称" class="input-base w-full" />
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">路径</label>
              <input v-model="formData.path" type="text" placeholder="/path/to/menu" class="input-base w-full" />
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">上级菜单</label>
              <select v-model="formData.parent_id" class="input-base w-full">
                <option :value="null">无（顶级菜单）</option>
                <option v-for="m in adminStore.menus" :key="m.id" :value="m.id">
                  {{ m.name }}
                </option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">图标名称</label>
              <input v-model="formData.icon" type="text" placeholder="例如: Settings" class="input-base w-full" />
            </div>
            <div>
              <label class="block text-sm font-medium text-surface-700 mb-2">排序序号</label>
              <input v-model.number="formData.sort_order" type="number" placeholder="0" class="input-base w-full" />
            </div>
            <div class="flex gap-3 pt-2">
              <button type="button" @click="closeForm" class="btn btn-outline flex-1 justify-center">取消</button>
              <button type="submit" class="btn btn-primary flex-1 justify-center">{{ editingMenu ? '保存' : '添加' }}</button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>
  </div>
</template>
