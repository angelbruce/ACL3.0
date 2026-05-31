import { workspaceApi, api } from './client'

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

export interface ProjectInfo {
  name: string
  path: string
  created_at: string
  updated_at: string
}

export interface CreateProjectRequest {
  name: string
}

export const workspaceService = {
  listProjects: () => api.get<ProjectInfo[]>(workspaceApi, '/api/workspace/projects'),

  createProject: (data: CreateProjectRequest) =>
    api.post<ProjectInfo>(workspaceApi, '/api/workspace/projects', data),

  deleteProject: (name: string) => api.delete<{ message: string }>(workspaceApi, `/api/workspace/projects/${encodeURIComponent(name)}`),

  listProjectFiles: (projectName: string) => 
    api.get<FileInfo[]>(workspaceApi, `/api/workspace/projects/${encodeURIComponent(projectName)}/files`),

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
  }
}
