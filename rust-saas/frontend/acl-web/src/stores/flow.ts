import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Flow, FlowRuntime, FlowRuntimeNode, CreateFlowRequest } from '@/types'
import { flowService } from '@/api'

export const useFlowStore = defineStore('flow', () => {
  const flows = ref<Flow[]>([])
  const currentFlow = ref<Flow | null>(null)
  const runtimes = ref<FlowRuntime[]>([])
  const currentRuntime = ref<{ runtime: FlowRuntime; nodes: FlowRuntimeNode[] } | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchFlows = async () => {
    loading.value = true
    error.value = null
    try {
      flows.value = await flowService.getFlows()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch flows'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchFlow = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      currentFlow.value = await flowService.getFlow(id)
      return currentFlow.value
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch flow'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取指定流程的所有运行时
   * @param flowId 流程ID
   */
  const fetchRuntimes = async (flowId: number) => {
    loading.value = true
    error.value = null
    try {
      runtimes.value = await flowService.getFlowRuntimes(flowId)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch runtimes'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取指定流程的运行时
   * @param flowId 运行时流程ID
   */
  const fetchRuntime = async (id: number): Promise<{ runtime: FlowRuntime; nodes: FlowRuntimeNode[] | null } | null> => {
    loading.value = true
    error.value = null
    try {
      const result = await flowService.getFlowRuntime(id)
      currentRuntime.value = { runtime: result[0], nodes: result[1] || [] }
      return currentRuntime.value
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch runtime'
      throw err
    } finally {
      loading.value = false
    }
  }


  const getFlowRuntimeByFlowId = async (flowId: number) => {
    loading.value = true
    error.value = null
    try {
      const runtime = await flowService.getFlowRuntimeByFlowId(flowId)
      return runtime
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch runtime'
      throw err
    } finally {
      loading.value = false
    }
  }


  const createFlow = async (data: CreateFlowRequest) => {
    loading.value = true
    error.value = null
    try {
      const flow = await flowService.createFlow(data)
      flows.value.push(flow)
      return flow
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to create flow'
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateFlow = async (id: number, data: CreateFlowRequest) => {
    loading.value = true
    error.value = null
    try {
      const flow = await flowService.updateFlow(id, data)
      const index = flows.value.findIndex((f) => f.id === id)
      if (index !== -1) {
        flows.value[index] = flow
      }
      if (currentFlow.value?.id === id) {
        currentFlow.value = flow
      }
      return flow
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to update flow'
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteFlow = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      await flowService.deleteFlow(id)
      flows.value = flows.value.filter((f) => f.id !== id)
      if (currentFlow.value?.id === id) {
        currentFlow.value = null
      }
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to delete flow'
      throw err
    } finally {
      loading.value = false
    }
  }

  const startFlow = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      const runtime = await flowService.startFlow(id)
      runtimes.value.push(runtime)
      return runtime
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to start flow'
      throw err
    } finally {
      loading.value = false
    }
  }

  const stopFlow = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      const runtime = await flowService.stopFlow(id)
      const index = runtimes.value.findIndex((r) => r.id === id)
      if (index !== -1) {
        runtimes.value[index] = runtime
      }
      return runtime
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to stop flow'
      throw err
    } finally {
      loading.value = false
    }
  }

  const sendHumanInput = async (flowId: number, nodeId: number, message: string) => {
    loading.value = true
    error.value = null
    try {
      await flowService.sendHumanInput(flowId, nodeId, message)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to send human input'
      throw err
    } finally {
      loading.value = false
    }
  }

  const getFlowRuntimeSessions = async (runtimeId: number) => {
    loading.value = true
    error.value = null
    try {
      return await flowService.getFlowRuntimeSessions(runtimeId)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to get sessions'
      throw err
    } finally {
      loading.value = false
    }
  }

  const getFlowRuntimeSessionItems = async (runtimeId: number, sessionId: number) => {
    loading.value = true
    error.value = null
    try {
      return await flowService.getFlowRuntimeSessionItems(runtimeId, sessionId)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to get session items'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    flows,
    currentFlow,
    runtimes,
    currentRuntime,
    loading,
    error,
    getFlowRuntimeByFlowId,
    fetchFlows,
    fetchFlow,
    fetchRuntimes,
    fetchRuntime,
    createFlow,
    updateFlow,
    deleteFlow,
    startFlow,
    stopFlow,
    sendHumanInput,
    getFlowRuntimeSessions,
    getFlowRuntimeSessionItems,
  }
})
