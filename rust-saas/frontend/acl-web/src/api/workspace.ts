import { workspaceApi, fetchStream, api } from './client'

export interface FileInfo {
  name: string
  path: string
  size: number
  is_directory: boolean
  created_at: string
  updated_at: string
  owner_id: number
  owner_name?: string
  is_shared: boolean
}

export interface KanbanBoard {
  id: number
  name: string
  description: string | null
  is_public: boolean
  created_by: number
  created_at: string
  updated_at: string
}

export interface KanbanItem {
  id: number
  board_id: number
  user_id: number
  file_path: string
  file_name: string
  shared_at: string
}

export interface KanbanBoardWithItems {
  board: KanbanBoard
  items: KanbanItem[]
  subscriber_count: number
}

export interface KanbanSubscription {
  id: number
  board_id: number
  user_id: number
  subscribed_at: string
}

export interface SubscribedBoard {
  board: KanbanBoard
  items: SharedFileInfo[]
}

export interface ProjectSummary {
  id: number
  user_id: number
  project_id: number
  file_name: string
  summary: string
  created_at: string
  updated_at: string
}

export interface CreateOrUpdateProjectSummaryRequest {
  file_name: string
  summary: string
}

export interface SharedFileInfo {
  id: number
  file_name: string
  file_path: string
  shared_at: string
  owner_id: number
  owner_name?: string
}

export interface CreateKanbanBoardRequest {
  name: string
  description?: string
  is_public?: boolean
}

export interface UpdateKanbanBoardRequest {
  name?: string
  description?: string
  is_public?: boolean
}

export interface ShareFileRequest {
  file_path: string
}

import type { Project, ProjectFile, ProjectChatMessage, CreateProjectRequest, UpdateProjectRequest, ProjectContainerConfig } from '@/types'

export interface ProjectInfo {
  name: string
  path: string
  created_at: string
  updated_at: string
}


export const workspaceService = {
  listProjects: () => api.get<Project[]>(workspaceApi, '/api/projects'),

  getProject: (id: number) => api.get<Project>(workspaceApi, `/api/projects/${id}`),

  createProject: (data: CreateProjectRequest) =>
    api.post<Project>(workspaceApi, '/api/projects', data),

  updateProject: (id: number, data: UpdateProjectRequest) =>
    api.put<Project>(workspaceApi, `/api/projects/${id}`, data),

  deleteProject: (id: number) => api.delete<{ message: string }>(workspaceApi, `/api/projects/${id}`),

  listProjectFiles: (projectId: number) => 
    api.get<ProjectFile[]>(workspaceApi, `/api/projects/${projectId}/files`),

  createProjectFile: (projectId: number, name: string, content?: string) =>
    api.post<ProjectFile>(workspaceApi, `/api/projects/${projectId}/files`, { name, content }),

  updateProjectFile: (fileId: number, content: string) =>
    api.put<ProjectFile>(workspaceApi, `/api/project-files/${fileId}`, { content }),

  deleteProjectFile: (fileId: number) => api.delete<{ message: string }>(workspaceApi, `/api/project-files/${fileId}`),

  getProjectMessages: (projectId: number) => api.get<ProjectChatMessage[]>(workspaceApi, `/api/projects/${projectId}/messages`),

  addProjectMessage: (projectId: number, content: string, role: 'user' | 'assistant' | 'system') =>
    api.post<ProjectChatMessage>(workspaceApi, `/api/projects/${projectId}/messages`, { content, role }),

  chatWithProject: (projectId: number, modelId: number, agentId: number | undefined, message: string, onMessage: (content: string) => void, onError: (error: Error) => void) => {
    const url = `/api/projects/${projectId}/chat`
    const body = JSON.stringify({ project_id: projectId, model_id: modelId, agent_id: agentId, message })
    
    return new Promise<void>((resolve, reject) => {
      const eventSource = new EventSource(`${workspaceApi.defaults.baseURL}${url}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
        },
        body,
      })

      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          if (data.type === 'message') {
            onMessage(data.content)
          } else if (data.type === 'end') {
            eventSource.close()
            resolve()
          }
        } catch (error) {
          onError(error instanceof Error ? error : new Error('Parse error'))
        }
      }

      eventSource.onerror = (error) => {
        eventSource.close()
        onError(error instanceof Error ? error : new Error('Stream error'))
        reject(error)
      }
    })
  },

  listFiles: () => api.get<FileInfo[]>(workspaceApi, '/api/workspace/files'),

  downloadFile: (path: string) => {
    return workspaceApi.get(`/api/workspace/files/${encodeURIComponent(path)}`, {
      responseType: 'blob'
    })
  },

  deleteFile: (path: string) => api.delete<{ message: string }>(workspaceApi, `/api/workspace/files/${encodeURIComponent(path)}`),

  getPublicKanbanBoards: () => api.get<KanbanBoard[]>(workspaceApi, '/api/kanban/boards'),

  createKanbanBoard: (data: CreateKanbanBoardRequest) =>
    api.post<KanbanBoard>(workspaceApi, '/api/kanban/boards', data),

  updateKanbanBoard: (id: number, data: UpdateKanbanBoardRequest) =>
    api.put<KanbanBoard>(workspaceApi, `/api/kanban/boards/${id}`, data),

  getKanbanBoard: (id: number) => api.get<KanbanBoardWithItems>(workspaceApi, `/api/kanban/boards/${id}`),

  deleteKanbanBoard: (id: number) => api.delete<{ message: string }>(workspaceApi, `/api/kanban/boards/${id}`),

  shareFileToBoard: (boardId: number, data: ShareFileRequest) =>
    api.post<KanbanItem>(workspaceApi, `/api/kanban/boards/${boardId}/files`, data),

  removeFileFromBoard: (itemId: number) => api.delete<{ message: string }>(workspaceApi, `/api/kanban/items/${itemId}`),

  subscribeBoard: (boardId: number) => api.post<KanbanSubscription>(workspaceApi, `/api/kanban/boards/${boardId}/subscribe`, {}),

  unsubscribeBoard: (boardId: number) => api.post<{ message: string }>(workspaceApi, `/api/kanban/boards/${boardId}/unsubscribe`, {}),

  getSubscribedBoards: () => api.get<SubscribedBoard[]>(workspaceApi, '/api/kanban/subscriptions'),

  downloadSharedFile: (boardId: number, filePath: string) => {
    return workspaceApi.get(`/api/kanban/boards/${boardId}/files/${encodeURIComponent(filePath)}`, {
      responseType: 'blob'
    })
  },


  deleteProjectMessage: (projectId: number, messageId: number) => api.delete<{ message: string }>(workspaceApi, `/api/projects/${projectId}/messages/${messageId}`),

  getProjectSummaries: (projectId: number) => 
    api.get<ProjectSummary[]>(workspaceApi, `/api/projects/${projectId}/summaries`),

  createOrUpdateProjectSummary: (projectId: number, data: CreateOrUpdateProjectSummaryRequest) =>
    api.post<ProjectSummary>(workspaceApi, `/api/projects/${projectId}/summaries`, data),

  
  getArticleVoiceLink: (articleId:number): Promise<string> => {
     return api.post<string>(workspaceApi, `/api/projects-files/voice/link/${articleId}`).then((res) => {
       let data = res;
       var baseURL = workspaceApi.defaults.baseURL
        let link = baseURL + "/voice" + data
        return new Promise((resolve) => resolve(link))
     })
  },

  getArticleVoice: (articleId:number): Promise<Blob> => {
    return new Promise<Blob>((resolve, reject) => {
      let buffer = new Uint8Array()
      fetchStream(`/api/projects-files/voice/${articleId}`, (value) => {
        if(value != null) {
          buffer  = new Uint8Array([...buffer, ...value])
        } else {
          let blob = new Blob([buffer], { type: 'audio/wav' })
          resolve(blob)
        }
      }).catch((err) => {
        reject(err)
      })
    })
  },

  getProjectContainerConfigs: (projectId: number) => api.get<ProjectContainerConfig[]>(workspaceApi, `/api/project-container-configs/${projectId}`),
  saveProjectContainerConfigs: (projectId:number, data: ProjectContainerConfig[]) => api.post<ProjectContainerConfig[]>(workspaceApi, `/api/project-container-configs/${projectId}`, data),
  startContainer: (projectId: number) => api.post<{ message: string }>(workspaceApi, `/api/project-container-configs/${projectId}/start`),
  executeCommand: (projectId: number, configId: number, command: string) => api.post<{ success: boolean, output: string, error?: string }>(workspaceApi, '/api/projects/execute-command', {
    project_id: projectId,
    config_id: configId,
    command
  }),

  executeCommandStream: (projectId: number, configId: number, command: string, onData: (data: string) => void, onError: (error: Error) => void, onComplete: () => void) => {
    const url = `${workspaceApi.defaults.baseURL}/api/projects/execute-command-stream`
    const accessToken = localStorage.getItem('access_token')

    fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${accessToken}`,
      },
      body: JSON.stringify({
        project_id: projectId,
        config_id: configId,
        command
      }),
    })
    .then(async (response) => {
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const reader = response.body?.getReader()
      if (!reader) {
        throw new Error('No response body')
      }

      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() || ''

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.substring(6)
            onData(data)
          }
        }
      }

      onComplete()
    })
    .catch(onError)
  },
  getContainerStatus: (projectId: number, configId?: number) => {
    const params = configId ? `?config_id=${configId}` : ''
    return api.get<{ statuses: any[], target_status: any | null }>(workspaceApi, `/api/project-container-configs/${projectId}/status${params}`)
  },
  stopContainer: (projectId: number) => api.post<{ message: string }>(workspaceApi, `/api/project-container-configs/${projectId}/stop`),
  cleanupContainer: (projectId: number) => api.post<{ message: string }>(workspaceApi, `/api/project-container-configs/${projectId}/cleanup`),
  
  // 刷新项目文件到容器并执行命令
  refreshFileToContainer: (projectId: number, data: {
    file_id: number
    config_id: number
    content: string
    command: string
  }) => api.post<{ success: boolean, message: string, output: string }>(workspaceApi, `/api/projects/${projectId}/refresh-file`, data),

  // Workspace 专用的 Chat/Stream 接口
  workspaceChatStream: async (
    request: {
      model_id: number
      project_id: number
      config_id: number
      agent_id?: number
      messages: { role: string; content?: string; tool_call_id?: string; name?: string; tool_calls?: unknown }[]
    },
    onMessage: (data: { content: string; tool_calls?: unknown; finish_reason?: string }) => void,
    onError?: (error: Error) => void
  ): Promise<void> => {
    const token = localStorage.getItem('access_token')
    const response = await fetch(`${workspaceApi.defaults.baseURL}/api/chat/stream`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(request),
    })

    if (!response.body) {
      onError?.(new Error('No response body'))
      return
    }

    const reader = response.body.getReader()
    const decoder = new TextDecoder()

    return new Promise((resolve, reject) => {
      const processStream = () => {
        reader.read().then(({ done, value }) => {
          if (done) {
            resolve()
            return
          }

          const chunk = decoder.decode(value, { stream: true })
          const lines = chunk.split('\n')

          for (const line of lines) {
            if (line.startsWith('data: ')) {
              const dataStr = line.slice(6).trim()
              if (dataStr === '[DONE]' || dataStr === 'DONE') {
                continue
              }
              try {
                const data = JSON.parse(dataStr)
                onMessage(data)
              } catch {
                // Ignore parse errors
              }
            }
          }

          processStream()
        }).catch((error) => {
          onError?.(error)
          reject(error)
        })
      }

      processStream()
    })
  },
}
