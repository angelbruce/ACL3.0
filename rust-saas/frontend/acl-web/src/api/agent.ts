import { agentApi, api } from './client'
import type { Agent, AgentDetail, CreateAgentRequest } from '@/types'

export const agentService = {
  getAgents: () => api.get<Agent[]>(agentApi, '/api/agents'),

  getAgent: (id: number) => api.get<AgentDetail>(agentApi, `/api/agents/${id}`),

  createAgent: (data: CreateAgentRequest) =>
    api.post<Agent>(agentApi, '/api/agents', data),

  updateAgent: (id: number, data: CreateAgentRequest) =>
    api.put<Agent>(agentApi, `/api/agents/${id}`, data),

  deleteAgent: (id: number) => api.delete<null>(agentApi, `/api/agents/${id}`),
}
