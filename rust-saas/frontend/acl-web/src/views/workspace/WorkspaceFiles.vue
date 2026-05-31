<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { Plus, X, Folder, FolderOpen, FileText, Download, Trash2 } from 'lucide-vue-next'
import { workspaceService, type FileInfo, type ProjectInfo, type CreateProjectRequest } from '@/api/workspace'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const projects = ref<ProjectInfo[]>([])
const currentProject = ref<ProjectInfo | null>(null)
const files = ref<FileInfo[]>([])
const loading = ref(true)
const error = ref('')
const showCreateModal = ref(false)
const newProjectName = ref('')
const showDeleteConfirm = ref(false)
const deleteTarget = ref<{ type: 'project' | 'file'; name: string; path?: string } | null>(null)

const activeTab = computed(() => currentProject.value?.name || 'root')

const fetchProjects = async () => {
  try {
    projects.value = await workspaceService.listProjects()
    if (projects.value.length > 0) {
      currentProject.value = projects.value[0]
      await fetchProjectFiles()
    } else {
      files.value = []
    }
  } catch (e) {
    console.error('Failed to load projects:', e)
    projects.value = []
    files.value = []
  }
}

const fetchProjectFiles = async () => {
  loading.value = true
  error.value = ''
  try {
    if (currentProject.value) {
      files.value = await workspaceService.listProjectFiles(currentProject.value.name)
    } else {
      files.value = []
    }
  } catch (e) {
    error.value = 'Failed to load files'
    console.error(e)
    files.value = []
  } finally {
    loading.value = false
  }
}

const handleCreateProject = async () => {
  if (!newProjectName.value.trim()) return
  
  try {
    const request: CreateProjectRequest = { name: newProjectName.value }
    const project = await workspaceService.createProject(request)
    projects.value.unshift(project)
    currentProject.value = project
    showCreateModal.value = false
    newProjectName.value = ''
    await fetchProjectFiles()
  } catch (e: any) {
    alert(e?.response?.data?.message || 'Failed to create project')
    console.error(e)
  }
}

const handleDeleteProject = (projectName: string) => {
  deleteTarget.value = { type: 'project', name: projectName }
  showDeleteConfirm.value = true
}

const confirmDelete = async () => {
  if (!deleteTarget.value) return
  
  if (deleteTarget.value.type === 'project') {
    try {
      await workspaceService.deleteProject(deleteTarget.value.name)
      projects.value = projects.value.filter(p => p.name !== deleteTarget.value!.name)
      
      if (currentProject.value?.name === deleteTarget.value.name) {
        currentProject.value = projects.value.length > 0 ? projects.value[0] : null
        await fetchProjectFiles()
      }
    } catch (e) {
      console.error('Failed to delete project:', e)
    }
  } else if (deleteTarget.value.type === 'file' && deleteTarget.value.path) {
    try {
      await workspaceService.deleteFile(deleteTarget.value.path)
      files.value = files.value.filter(f => f.path !== deleteTarget.value!.path)
    } catch (e) {
      console.error('Failed to delete file:', e)
    }
  }
  
  showDeleteConfirm.value = false
  deleteTarget.value = null
}

const cancelDelete = () => {
  showDeleteConfirm.value = false
  deleteTarget.value = null
}

const handleSelectProject = async (project: ProjectInfo) => {
  currentProject.value = project
  await fetchProjectFiles()
}

const handleDownload = async (file: FileInfo) => {
  try {
    const response = await workspaceService.downloadFile(file.path)
    const blob = response.data
    const url = window.URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = file.name
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    window.URL.revokeObjectURL(url)
  } catch (e) {
    console.error('Failed to download file:', e)
  }
}

const handleDeleteFile = (file: FileInfo) => {
  deleteTarget.value = { type: 'file', name: file.name, path: file.path }
  showDeleteConfirm.value = true
}

const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatDate = (dateStr: string): string => {
  return new Date(dateStr).toLocaleString()
}

onMounted(fetchProjects)
</script>

<template>
  <div class="w-full md:px-1 h-full" >
    <div class="header">
      <div class="header-left">
        <h2>工作区</h2>
        <p class="text-gray-500">管理您的项目和文件</p>
      </div>
      <button class="btn btn-primary" @click="showCreateModal = true">
        <Plus class="w-4 h-4 mr-1" />
        新建项目
      </button>
    </div>

    <div v-if="projects.length === 0" class="empty-projects">
      <div class="empty-icon">📂</div>
      <p>暂无项目，创建您的第一个项目开始使用</p>
    </div>

    <div v-else class="workspace-content w-full">
      <div class="tabs ">
        <button
          v-for="project in projects"
          :key="project.name"
          :class="[
            'tab-btn',
            currentProject?.name === project.name ? 'active' : ''
          ]"
          @click="handleSelectProject(project)"
        >
          <FolderOpen class="w-4 h-4 mr-2" />
          {{ project.name }}
          <button 
            class="tab-delete"
            @click.stop="handleDeleteProject(project.name)"
          >
            <X class="w-3 h-3" />
          </button>
        </button>
      </div>

      <div class="files-container">
        <div v-if="loading" class="loading">
          <div class="loader"></div>
        </div>

        <div v-else-if="error" class="error">
          {{ error }}
        </div>

        <div v-else-if="files.length === 0" class="empty-files">
          <div class="empty-icon">📁</div>
          <p>此项目中暂无文件</p>
        </div>

        <div v-else class="files-list">
          <div
            v-for="file in files"
            :key="file.file_path"
            class="file-item"
          >
            <div class="file-icon">
              <Folder v-if="file.is_directory" class="w-6 h-6 text-yellow-500" />
              <FileText v-else class="w-6 h-6 text-blue-500" />
            </div>
            <div class="file-info">
              <span class="file-name">{{ file.file_name }}</span>
              <span class="file-meta">{{ formatSize(file.file_size) }} - {{ formatDate(file.updated_at) }}</span>
            </div>
            <div class="file-actions">
              <button 
                class="action-btn download-btn"
                title="下载"
                @click="handleDownload(file)"
              >
                <Download class="w-4 h-4" />
              </button>
              <button 
                class="action-btn delete-btn"
                title="删除"
                @click="handleDeleteFile(file)"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <ConfirmDialog
      :visible="showDeleteConfirm"
      :title="deleteTarget?.type === 'project' ? '删除项目' : '删除文件'"
      :message="deleteTarget ? `确定要删除 ${deleteTarget.name} 吗？此操作无法撤销。` : ''"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
    />

    <div v-if="showCreateModal" class="modal-overlay" @click.self="showCreateModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3>创建新项目</h3>
          <button class="modal-close" @click="showCreateModal = false">×</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>项目名称</label>
            <input
              v-model="newProjectName"
              type="text"
              placeholder="请输入项目名称"
              class="form-control"
              @keyup.enter="handleCreateProject"
            />
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="showCreateModal = false">取消</button>
          <button class="btn btn-primary" @click="handleCreateProject">创建</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workspace-container {
  margin: 0 auto;
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-left h2 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 4px;
}

.btn {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: background-color 0.2s;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
}

.btn-primary:hover {
  background-color: #2563eb;
}

.btn-secondary {
  background-color: #f3f4f6;
  color: #374151;
}

.btn-secondary:hover {
  background-color: #e5e7eb;
}

.empty-projects {
  text-align: center;
  padding: 80px 20px;
  color: #9ca3af;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.workspace-content {
  background: white;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.tabs {
  display: flex;
  gap: 4px;
  padding: 8px 8px 0;
  background-color: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  overflow-x: auto;
}

.tab-btn {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  border: none;
  background: transparent;
  border-radius: 6px 6px 0 0;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: #6b7280;
  transition: all 0.2s;
  white-space: nowrap;
}

.tab-btn:hover {
  background-color: #f3f4f6;
}

.tab-btn.active {
  background-color: white;
  color: #3b82f6;
  box-shadow: 0 -1px 0 0 #3b82f6;
}

.tab-delete {
  margin-left: 8px;
  padding: 2px;
  border: none;
  background: transparent;
  color: #9ca3af;
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.2s;
}

.tab-delete:hover {
  background-color: #fee2e2;
  color: #dc2626;
}

.files-container {
  padding: 16px;
}

.loading {
  display: flex;
  justify-content: center;
  padding: 40px;
}

.loader {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error {
  color: #ef4444;
  padding: 20px;
  text-align: center;
}

.empty-files {
  padding: 60px 20px;
  text-align: center;
  color: #9ca3af;
}

.files-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-item {
  display: flex;
  align-items: center;
  padding: 12px;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.file-item:hover {
  background-color: #f9fafb;
}

.file-icon {
  margin-right: 12px;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  display: block;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  display: block;
  font-size: 12px;
  color: #6b7280;
  margin-top: 2px;
}

.file-actions {
  display: flex;
  gap: 8px;
  margin-left: 12px;
  opacity: 0;
  transition: opacity 0.2s;
}

.file-item:hover .file-actions {
  opacity: 1;
}

.action-btn {
  padding: 6px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.download-btn {
  color: #3b82f6;
}

.download-btn:hover {
  background-color: #eff6ff;
}

.delete-btn {
  color: #ef4444;
}

.delete-btn:hover {
  background-color: #fee2e2;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 8px;
  width: 90%;
  max-width: 480px;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.modal-header h3 {
  margin: 0;
}

.modal-close {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #9ca3af;
}

.modal-body {
  padding: 20px;
}

.modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 16px 20px;
  border-top: 1px solid #e5e7eb;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 4px;
}

.form-control {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  box-sizing: border-box;
}

.form-control:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}
</style>
