import { sessionApi, api } from './client'
import type { Session, SessionItem, CreateSessionRequest, AddMessageRequest } from '@/types'

export interface UpdateSessionRequest {
  agent_id?: number | null
  model_id?: number | null
}

export const sessionService = {
  getSessions: () => api.get<Session[]>(sessionApi, '/api/sessions'),

  getSession: (id: number) => api.get<Session>(sessionApi, `/api/sessions/${id}`),

  createSession: (data: CreateSessionRequest) =>
    api.post<Session>(sessionApi, '/api/sessions', data),

  updateSession: (id: number, data: UpdateSessionRequest) =>
    api.put<Session>(sessionApi, `/api/sessions/${id}`, data),

  deleteSession: (id: number) => api.delete<null>(sessionApi, `/api/sessions/${id}`),

  getMessages: (id: number) =>
    api.get<SessionItem[]>(sessionApi, `/api/sessions/${id}/messages`),

  addMessage: (id: number, data: AddMessageRequest) =>
    api.post<SessionItem>(sessionApi, `/api/sessions/${id}/messages`, data),
}
