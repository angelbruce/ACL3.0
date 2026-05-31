import { flowApi, api } from './client'
import type { Flow, FlowRuntime, FlowRuntimeNode, CreateFlowRequest } from '@/types'

export const flowService = {
  getFlows: () => api.get<Flow[]>(flowApi, '/api/flows'),

  getFlow: (id: number) => api.get<Flow>(flowApi, `/api/flows/${id}`),

  createFlow: (data: CreateFlowRequest) =>
    api.post<Flow>(flowApi, '/api/flows', data),

  updateFlow: (id: number, data: CreateFlowRequest) =>
    api.put<Flow>(flowApi, `/api/flows/${id}`, data),

  deleteFlow: (id: number) => api.delete<null>(flowApi, `/api/flows/${id}`),

  startFlow: (id: number) => api.post<FlowRuntime>(flowApi, `/api/flows/${id}/start`),

  stopFlow: (id: number) => api.post<FlowRuntime>(flowApi, `/api/flows/${id}/stop`),

  getFlowRuntimes: (id: number) =>
    api.get<FlowRuntime[]>(flowApi, `/api/flows/${id}/runtimes`),

  getFlowRuntime: (id: number) =>
    api.get<{ 0: FlowRuntime; 1: FlowRuntimeNode[] }>(flowApi, `/api/flows/${id}/runtime`),
}
