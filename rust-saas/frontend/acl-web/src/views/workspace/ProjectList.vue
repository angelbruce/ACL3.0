<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { FileText, Code, Plus, Clock, Trash2, Edit3, X, Sparkles } from 'lucide-vue-next'
import { useWorkspaceStore, useAuthStore } from '@/stores'
import type { Project, ProjectPurpose, CreateProjectRequest } from '@/types'

const router = useRouter()
const workspaceStore = useWorkspaceStore()
const authStore = useAuthStore()

const loading = ref(true)
const error = ref<string | null>(null)
const showCreateModal = ref(false)
const newProjectName = ref('')
const newProjectPurpose = ref<ProjectPurpose>('article')
const newProjectDescription = ref('')

const projects = computed(() => workspaceStore.projects)

const getPurposeIcon = (purpose: ProjectPurpose) => {
  return purpose === 'article' ? FileText : Code
}

const getPurposeLabel = (purpose: ProjectPurpose) => {
  switch(purpose) {
    case 'article':
      return '文章创作'
    case 'coding':
      return '代码开发'
    case 'education':
      return '教育'
    case 'mcp':
      return 'MCP项目'
    default:
      return '未知项目'
  }
}

const getPurposeClass = (purpose: ProjectPurpose) => {
  return purpose === 'article' 
    ? 'bg-blue-500 text-white' 
    : 'bg-green-500 text-white'
}

const getPurposeBgClass = (purpose: ProjectPurpose) => {
  return purpose === 'article' 
    ? 'bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-100' 
    : 'bg-gradient-to-br from-green-50 to-emerald-50 border-green-100'
}

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

const goToProject = (project: Project) => {
  router.push(`/projects/${project.id}`)
}

const openCreateModal = () => {
  showCreateModal.value = true
  newProjectName.value = ''
  newProjectPurpose.value = 'article'
  newProjectDescription.value = ''
}

const closeCreateModal = () => {
  showCreateModal.value = false
}

const createProject = async () => {
  if (!newProjectName.value.trim()) {
    error.value = '请输入项目名称'
    return
  }

  const request: CreateProjectRequest = {
    name: newProjectName.value.trim(),
    purpose: newProjectPurpose.value,
    description: newProjectDescription.value || undefined,
  }

  try {
    await workspaceStore.createProject(request)
    closeCreateModal()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '创建项目失败'
  }
}

const deleteProject = async (project: Project) => {
  if (!confirm(`确定要删除项目 "${project.name}" 吗？`)) {
    return
  }

  try {
    await workspaceStore.deleteProject(project.id)
  } catch (err) {
    error.value = err instanceof Error ? err.message : '删除项目失败'
  }
}

onMounted(async () => {
  if (!authStore.user && authStore.isAuthenticated) {
    router.push('/login')
    return
  }

  loading.value = true
  try {
    await workspaceStore.fetchProjects()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载项目失败'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="min-h-screen bg-surface-50">
    <div class="max-w-6xl mx-auto px-4 py-8">
      <div class="flex items-center justify-between mb-8">
        <div>
          <h1 class="text-2xl font-bold text-surface-800">我的项目</h1>       
          <p class="text-surface-500 mt-1">管理您的工作区项目</p>
        </div>
        <button 
          @click="openCreateModal"
          class="flex items-center gap-2 px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors shadow-sm"
        >
          <Plus class="w-5 h-5" />
          新建项目
        </button>
      </div>

      <div v-if="loading" class="flex items-center justify-center py-16">
        <div class="w-8 h-8 border-4 border-primary-500 border-t-transparent rounded-full animate-spin"></div>
      </div>

      <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-xl p-6 text-center">
        <p class="text-red-600">{{ error }}</p>
        <button 
          @click="error = null; workspaceStore.fetchProjects()" 
          class="mt-4 text-primary-500 hover:text-primary-600"
        >
          重试
        </button>
      </div>

      <div v-else-if="projects.length === 0" class="text-center py-16">
        <div class="w-16 h-16 bg-surface-100 rounded-full flex items-center justify-center mx-auto mb-4">
          <Sparkles class="w-8 h-8 text-surface-400" />
        </div>
        <h3 class="text-lg font-medium text-surface-600 mb-2">暂无项目</h3>
        <p class="text-surface-400 mb-6">点击上方按钮创建您的第一个项目</p>
        <button 
          @click="openCreateModal"
          class="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
        >
          新建项目
        </button>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div 
          v-for="project in projects" 
          :key="project.id"
          @click="goToProject(project)"
          :class="['p-6 rounded-xl border cursor-pointer transition-all hover:shadow-lg hover:-translate-y-1', getPurposeBgClass(project.purpose)]"
        >
          <div class="flex items-start justify-between mb-4">
            <div :class="['p-3 rounded-lg', getPurposeClass(project.purpose)]">
              <component :is="getPurposeIcon(project.purpose)" class="w-6 h-6" />
            </div>
            <div class="flex gap-2">
              <button 
                @click.stop
                class="p-2 text-surface-400 hover:text-surface-600 hover:bg-surface-200 rounded-lg transition-colors"
              >
                <Edit3 class="w-4 h-4" />
              </button>
              <button 
                @click.stop="deleteProject(project)"
                class="p-2 text-surface-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>

          <h3 class="text-lg font-semibold text-surface-800 mb-1">{{ project.name }}</h3>
          <p class="text-sm text-surface-500 mb-4">{{ project.description || '暂无描述' }}</p>

          <div class="flex items-center gap-4 text-xs text-surface-400">
            <span :class="['px-2 py-1 rounded-full', getPurposeClass(project.purpose)]">
              {{ getPurposeLabel(project.purpose) }}
            </span>
            <span class="flex items-center gap-1">
              <Clock class="w-3 h-3" />
              {{ formatDate(project.last_accessed_at) }}
            </span>
          </div>

          <div class="mt-4 pt-4 border-t border-surface-200">
            <div class="flex items-center justify-between text-xs text-surface-400">
              <span v-if="project.model_name" class="flex items-center gap-1">
                <span class="w-2 h-2 rounded-full bg-blue-400"></span>
                {{ project.model_name }}
              </span>
              <span v-else class="text-surface-300">未设置模型</span>
              <span v-if="project.agent_name" class="flex items-center gap-1">
                <span class="w-2 h-2 rounded-full bg-green-400"></span>
                {{ project.agent_name }}
              </span>
              <span v-else class="text-surface-300">未设置Agent</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showCreateModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl p-6 w-full max-w-md mx-4">
        <div class="flex items-center justify-between mb-6">
          <h2 class="text-xl font-semibold text-surface-800">创建新项目</h2>
          <button @click="closeCreateModal" class="p-2 text-surface-400 hover:text-surface-600 hover:bg-surface-100 rounded-lg transition-colors">
            <X class="w-5 h-5" />
          </button>
        </div>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">项目名称 *</label>
            <input 
              v-model="newProjectName"
              type="text"
              placeholder="输入项目名称"
              class="w-full px-4 py-2 border border-surface-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              @keydown.enter="createProject"
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">项目用途 *</label>
            <div class="flex gap-3">
              <button 
                @click="newProjectPurpose = 'article'"
                :class="['flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg border transition-colors', newProjectPurpose === 'article' ? 'bg-blue-500 text-white border-blue-500' : 'border-surface-200 text-surface-600 hover:bg-surface-50']"
              >
                <FileText class="w-4 h-4" />
                文章创作
              </button>
              <button 
                @click="newProjectPurpose = 'coding'"
                :class="['flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg border transition-colors', newProjectPurpose === 'coding' ? 'bg-green-500 text-white border-green-500' : 'border-surface-200 text-surface-600 hover:bg-surface-50']"
              >
                <Code class="w-4 h-4" />
                代码开发
              </button>
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">项目描述</label>
            <textarea 
              v-model="newProjectDescription"
              placeholder="输入项目描述（可选）"
              rows="3"
              class="w-full px-4 py-2 border border-surface-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent resize-none"
            ></textarea>
          </div>

          <p v-if="error" class="text-red-500 text-sm">{{ error }}</p>
        </div>

        <div class="flex gap-3 mt-6">
          <button 
            @click="closeCreateModal"
            class="flex-1 px-4 py-2 border border-surface-200 text-surface-600 rounded-lg hover:bg-surface-50 transition-colors"
          >
            取消
          </button>
          <button 
            @click="createProject"
            class="flex-1 px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
          >
            创建
          </button>
        </div>
      </div>
    </div>
  </div>
</template>