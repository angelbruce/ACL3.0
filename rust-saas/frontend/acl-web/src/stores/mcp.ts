import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { MCPTool, McpServer, CreateMcpServerRequest, McpServerWithTools } from '@/types'
import { mcpService } from '@/api'

export const useMcpStore = defineStore('mcp', () => {
  const tools = ref<MCPTool[]>([])
  const servers = ref<McpServer[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Tools
  const fetchTools = async () => {
    loading.value = true
    error.value = null
    try {
      tools.value = await mcpService.getTools()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch tools'
      throw err
    } finally {
      loading.value = false
    }
  }

  const callTool = async (name: string, args: Record<string, unknown>) => {
    loading.value = true
    error.value = null
    try {
      return await mcpService.callTool(name, args)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to call tool'
      throw err
    } finally {
      loading.value = false
    }
  }

  // Servers
  const fetchServers = async () => {
    loading.value = true
    error.value = null
    try {
      servers.value = await mcpService.getServers()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch servers'
      throw err
    } finally {
      loading.value = false
    }
  }

  const createServer = async (data: CreateMcpServerRequest) => {
    loading.value = true
    error.value = null
    try {
      const newServer = await mcpService.createServer(data)
      servers.value.push(newServer)
      await fetchTools()
      return newServer
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to create server'
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateServer = async (id: number, data: CreateMcpServerRequest) => {
    loading.value = true
    error.value = null
    try {
      const updatedServer = await mcpService.updateServer(id, data)
      const index = servers.value.findIndex(s => s.id === id)
      if (index !== -1) {
        servers.value[index] = updatedServer
      }
      await fetchTools()
      return updatedServer
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to update server'
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteServer = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      await mcpService.deleteServer(id)
      servers.value = servers.value.filter(s => s.id !== id)
      await fetchTools()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to delete server'
      throw err
    } finally {
      loading.value = false
    }
  }

  const toggleServer = async (id: number, enabled: boolean) => {
    loading.value = true
    error.value = null
    try {
      const updatedServer = await mcpService.toggleServer(id, enabled)
      const index = servers.value.findIndex(s => s.id === id)
      if (index !== -1) {
        servers.value[index] = updatedServer
      }
      await fetchTools()
      return updatedServer
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to toggle server'
      throw err
    } finally {
      loading.value = false
    }
  }

  const refreshServers = async () => {
    loading.value = true
    error.value = null
    try {
      await mcpService.refreshServers()
      await fetchServers()
      await fetchTools()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to refresh servers'
      throw err
    } finally {
      loading.value = false
    }
  }

  const getServerTools = async (id: number): Promise<McpServerWithTools> => {
    loading.value = true
    error.value = null
    try {
      return await mcpService.getServerTools(id)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to get server tools'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    tools,
    servers,
    loading,
    error,
    fetchTools,
    callTool,
    fetchServers,
    createServer,
    updateServer,
    deleteServer,
    toggleServer,
    refreshServers,
    getServerTools,
  }
})
