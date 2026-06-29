<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, X, Trash2, FileText, Loader2, Edit2, Code } from 'lucide-vue-next'
import { useWorkspaceStore, } from '@/stores/workspace'
import { ProjectPurpose, CreateProjectRequest,Project } from '@/types/index'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const router = useRouter()
const workspaceStore = useWorkspaceStore()

const projects = computed(() => workspaceStore.projects)
const currentProject = ref<Project | null>(null)
const loading = ref(true)
const error = ref('')
const showCreateModal = ref(false)
const newProjectName = ref('')
const newProjectPurpose = ref<ProjectPurpose>('article')
const newProjectDescription = ref('')
const showDeleteConfirm = ref(false)
const deleteTarget = ref<{ type: 'project'; id: number; name: string } | null>(null)
const deletingId = ref<number | null>(null)

const getPurposeIcon = (purpose: ProjectPurpose) => {
  return purpose === 'article' ? FileText : Code
}

const getPurposeColor = (purpose: ProjectPurpose) => {
  switch (purpose) {
    case 'education': return 'from-orange-100 to-orange-200 text-orange-500'
    case 'article': return 'from-blue-100 to-blue-200 text-blue-500'
    case 'coding': return 'from-green-100 to-green-200 text-green-500'
    case 'mcp': return 'from-cyan-100 to-cyan-200 text-cyan-500'
    default: return ''
  }
}

const getPurposeLabel = (purpose: ProjectPurpose) => {
  switch (purpose) {
    case 'article': return '文章创作'
    case 'coding': return '代码开发'
    case 'education': return '教育'
    case 'mcp': return 'MCP'
    default: return '未知'
  }
}

const fetchProjects = async () => {
  loading.value = true
  error.value = ''
  try {
    await workspaceStore.fetchProjects()
    if (projects.value.length > 0) {
      currentProject.value = projects.value[0]
    }
  } catch (e) {
    console.error('Failed to load projects:', e)
    error.value = '加载项目失败'
  } finally {
    loading.value = false
  }
}

const handleCreateProject = async () => {
  if (!newProjectName.value.trim()) return
  
  try {
    const request: CreateProjectRequest = {
      name: newProjectName.value,
      purpose: type.value,
      description: newProjectDescription.value || undefined,
    }
    const project = await workspaceStore.createProject(request)
    currentProject.value = project
    showCreateModal.value = false
    newProjectName.value = ''
    newProjectDescription.value = ''
  
    error.value = ''
  } catch (e: any) {
    error.value = e?.response?.data?.message || '创建项目失败'
    console.error(e)
  }
}

const openCreateModal = () => {
  showCreateModal.value = true
  newProjectName.value = ''
  newProjectPurpose.value = 'article'
  newProjectDescription.value = ''
  error.value = ''
}

const handleDeleteProject = async (project: Project, event: Event) => {
  event.stopPropagation()
  if (confirm(`确定要删除项目 "${project.name}" 吗？`)) {
    deletingId.value = project.id
    try {
      await workspaceStore.deleteProject(project.id)
      if (currentProject.value?.id === project.id) {
        currentProject.value = projects.value.length > 0 ? projects.value[0] : null
      }
    } catch (e) {
      console.error('Failed to delete project:', e)
    } finally {
      deletingId.value = null
    }
  }
}

const goToProject = (project: Project) => {
  router.push(`/projects/${project.id}`)
}

const handleSelectProject = (project: Project) => {
  currentProject.value = project
}


const type = ref<ProjectPurpose>('education')

const categoryProjects = computed(() => {
  if(projects.value === null || projects.value.length === 0) return []
  var filterProjects = projects.value.filter((project) => project.purpose === type.value)
  return filterProjects ||[]
})

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

onMounted(fetchProjects)
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">工作区</h1>
        <p class="page-subtitle">workspace management</p>
          
      </div>
      <div class="flex items-center gap-4 justify-center">
          <div class="cursor-pointer hover:text-red-600  font-medium text-black-600 text-sm" @click="type = 'education'"><span :class="type === 'education' ? 'text-blue-600' : ''" >教育</span></div>
          <div class="cursor-pointer hover:text-red-600  font-medium text-black-600 text-sm" @click="type = 'article'"><span :class="type === 'article' ? 'text-blue-600' : ''" >有声小说</span></div>   
          <div class="cursor-pointer hover:text-red-600  font-medium text-black-600 text-sm" @click="type = 'coding'"><span :class="type === 'coding' ? 'text-blue-600' : ''" >系统开发</span></div>  
          <div class="cursor-pointer hover:text-red-600  font-medium text-black-600 text-sm" @click="type = 'mcp'"><span :class="type === 'mcp' ? 'text-blue-600' : ''" >mcp</span></div> 
        </div>
      <button @click="openCreateModal" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        新建项目
      </button>
    </div>

    <div v-if="loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="categoryProjects.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-blue-50 border border-blue-100 flex items-center justify-center mb-4">
        <FileText class="w-8 h-8 text-blue-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无项目</h3>
      <p class="text-surface-400 mb-6">创建一个项目来开始您的创作</p>
      <button @click="openCreateModal" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 创建项目
      </button>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <div
        v-for="project in categoryProjects"
        :key="project.id"
        @click="goToProject(project)"
        class="card p-5 cursor-pointer group"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div :class="['w-12 h-12 rounded-xl bg-gradient-to-br', getPurposeColor(project.purpose), 'flex items-center justify-center']">
              <component :is="getPurposeIcon(project.purpose)" class="w-6 h-6" />
            </div>
            <div>
              <p class="text-xs text-surface-400 mb-0.5">用途</p>
              <span :class="['px-2 py-0.5 rounded-full text-xs font-medium', getPurposeColor(project.purpose)]">
                {{ getPurposeLabel(project.purpose) }}
              </span>
              <p class="text-xs text-surface-400 mt-1">{{ formatDate(project.created_at) }}</p>
            </div>
          </div>
          <div class="flex items-center gap-1">
            <button @click.stop="handleDeleteProject(project, $event)" :disabled="deletingId === project.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
              <Loader2 v-if="deletingId === project.id" class="w-4 h-4 animate-spin" />
              <Trash2 v-else class="w-4 h-4" />
            </button>
          </div>
        </div>

        <div class="min-h-[80px]">
          <p class="text-xs text-surface-400 mb-1">名称</p>
          <h3 class="font-semibold text-base text-surface-800 truncate" :title="project.name">{{ project.name }}</h3>
        </div>

        <div class="mt-3 pt-3 border-t border-surface-100">
          <p class="text-xs text-surface-400 mb-1">描述</p>
          <p v-if="project.description" class="text-sm text-surface-600 line-clamp-2">{{ project.description }}</p>
          <p v-else class="text-sm text-surface-300 italic">暂无描述</p>
        </div>
      </div>
    </div>
  </div>

  <div v-if="showCreateModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-white rounded-xl p-6 w-full max-w-md mx-4">
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-xl font-semibold text-surface-800">创建新项目</h2>
        <button @click="showCreateModal = false" class="p-2 text-surface-400 hover:text-surface-600 hover:bg-surface-100 rounded-lg transition-colors">
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
            class="w-full px-4 py-2 border border-surface-200 rounded-lg focus:outline-none focus:ring-2 
            text-sm
            focus:ring-primary-500 focus:border-transparent"
            @keydown.enter="handleCreateProject"
          />
        </div>

        <!-- <div>
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
        </div> -->

        <div>
          <label class="block text-sm font-medium text-surface-700 mb-2">项目描述</label>
          <textarea 
            v-model="newProjectDescription"
            placeholder="输入项目描述（项目用途、目标、内容等，最终结果描述）"
            rows="10"
            class="w-full px-4 py-2 border border-surface-200 rounded-lg focus:outline-none focus:ring-2 
            text-sm
            focus:ring-primary-500 focus:border-transparent resize-none"
          ></textarea>
        </div>

        <p v-if="error" class="text-red-500 text-sm">{{ error }}</p>
      </div>

      <div class="flex gap-3 mt-6">
        <button 
          @click="showCreateModal = false"
          class="flex-1 px-4 py-2 border border-surface-200 text-surface-600 rounded-lg hover:bg-surface-50 transition-colors"
        >
          取消
        </button>
        <button 
          @click="handleCreateProject"
          class="flex-1 px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
        >
          创建
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
