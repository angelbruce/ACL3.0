import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Project, ProjectFile, ProjectChatMessage, CreateProjectRequest, UpdateProjectRequest, ProjectSettings } from '@/types'
import { workspaceService } from '@/api'

export const useWorkspaceStore = defineStore('workspace', () => {
  const projects = ref<Project[]>([])
  const currentProject = ref<Project | null>(null)
  const projectFiles = ref<ProjectFile[]>([])
  const projectMessages = ref<ProjectChatMessage[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchProjects = async () => {
    loading.value = true
    error.value = null
    try {
      projects.value = await workspaceService.listProjects()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch projects'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchProject = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      currentProject.value = await workspaceService.getProject(id)
      return currentProject.value
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch project'
      throw err
    } finally {
      loading.value = false
    }
  }

  const createProject = async (data: CreateProjectRequest) => {
    loading.value = true
    error.value = null
    try {
      const project = await workspaceService.createProject(data)
      projects.value.push(project)
      return project
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to create project'
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateProject = async (id: number, data: UpdateProjectRequest) => {
    loading.value = true
    error.value = null
    try {
      const project = await workspaceService.updateProject(id, data)
      const index = projects.value.findIndex((p) => p.id === id)
      if (index !== -1) {
        projects.value[index] = project
      }
      if (currentProject.value?.id === id) {
        currentProject.value = project
      }
      return project
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to update project'
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteProject = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      await workspaceService.deleteProject(id)
      projects.value = projects.value.filter((p) => p.id !== id)
      if (currentProject.value?.id === id) {
        currentProject.value = null
        projectFiles.value = []
        projectMessages.value = []
      }
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to delete project'
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateProjectSettings = async (id: number, settings: ProjectSettings) => {
    loading.value = true
    error.value = null
    try {
      const project = await workspaceService.updateProject(id, settings)
      const index = projects.value.findIndex((p) => p.id === id)
      if (index !== -1) {
        projects.value[index] = project
      }
      if (currentProject.value?.id === id) {
        currentProject.value = project
      }
      return project
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to update project settings'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchProjectFiles = async (projectId: number) => {
    loading.value = true
    error.value = null
    try {
      projectFiles.value = await workspaceService.listProjectFiles(projectId)
      return projectFiles.value
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch project files'
      throw err
    } finally {
      loading.value = false
    }
  }

  const createProjectFile = async (projectId: number, name: string, content?: string) => {
    loading.value = true
    error.value = null
    try {
      const file = await workspaceService.createProjectFile(projectId, name, content)
      projectFiles.value.push(file)
      return file
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to create project file'
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateProjectFile = async (fileId: number, content: string) => {
    loading.value = true
    error.value = null
    try {
      const file = await workspaceService.updateProjectFile(fileId, content)
      const index = projectFiles.value.findIndex((f) => f.id === fileId)
      if (index !== -1) {
        projectFiles.value[index] = file
      }
      return file
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to update project file'
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteProjectFile = async (fileId: number) => {
    loading.value = true
    error.value = null
    try {
      await workspaceService.deleteProjectFile(fileId)
      projectFiles.value = projectFiles.value.filter((f) => f.id !== fileId)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to delete project file'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchProjectMessages = async (projectId: number) => {
    loading.value = true
    error.value = null
    try {
      projectMessages.value = await workspaceService.getProjectMessages(projectId)
      return projectMessages.value
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch project messages'
      throw err
    } finally {
      loading.value = false
    }
  }

  const addProjectMessage = async (projectId: number, content: string, role: 'user' | 'assistant' | 'system') => {
    error.value = null
    try {
      const message = await workspaceService.addProjectMessage(projectId, content, role)
      projectMessages.value.push(message)
      return message
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to add project message'
      throw err
    }
  }

  const clearError = () => {
    error.value = null
  }

  return {
    projects,
    currentProject,
    projectFiles,
    projectMessages,
    loading,
    error,
    fetchProjects,
    fetchProject,
    createProject,
    updateProject,
    deleteProject,
    updateProjectSettings,
    fetchProjectFiles,
    createProjectFile,
    updateProjectFile,
    deleteProjectFile,
    fetchProjectMessages,
    addProjectMessage,
    clearError,
  }
})