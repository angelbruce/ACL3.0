import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Agent, AgentDetail, CreateAgentRequest } from '@/types'
import { agentService } from '@/api'

export const useAgentStore = defineStore('agent', () => {
  const agents = ref<Agent[]>([])
  const currentAgent = ref<AgentDetail | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchAgents = async () => {
    loading.value = true
    error.value = null
    try {
      agents.value = await agentService.getAgents()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch agents'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchAgent = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      currentAgent.value = await agentService.getAgent(id)
      return currentAgent.value
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch agent'
      throw err
    } finally {
      loading.value = false
    }
  }

  const createAgent = async (data: CreateAgentRequest) => {
    loading.value = true
    error.value = null
    try {
      const agent = await agentService.createAgent(data)
      agents.value.push(agent)
      return agent
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to create agent'
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateAgent = async (id: number, data: CreateAgentRequest) => {
    loading.value = true
    error.value = null
    try {
      const agent = await agentService.updateAgent(id, data)
      const index = agents.value.findIndex((a) => a.id === id)
      if (index !== -1) {
        agents.value[index] = agent
      }
      if (currentAgent.value?.id === id) {
        currentAgent.value = await agentService.getAgent(id)
      }
      return agent
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to update agent'
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteAgent = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      await agentService.deleteAgent(id)
      agents.value = agents.value.filter((a) => a.id !== id)
      if (currentAgent.value?.id === id) {
        currentAgent.value = null
      }
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to delete agent'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    agents,
    currentAgent,
    loading,
    error,
    fetchAgents,
    fetchAgent,
    createAgent,
    updateAgent,
    deleteAgent,
  }
})
