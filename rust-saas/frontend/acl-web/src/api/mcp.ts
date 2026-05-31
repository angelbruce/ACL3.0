import { mcpApi, api } from './client'
import type { MCPTool, MCPToolCallResult, McpServer, CreateMcpServerRequest, McpServerWithTools } from '@/types'

export const mcpService = {
  getTools: () => api.get<MCPTool[]>(mcpApi, '/api/mcp/tools'),

  callTool: (name: string, args: Record<string, unknown>) =>
    api.post<MCPToolCallResult>(mcpApi, `/api/mcp/tools/${name}`, args),

  // MCP Server management
  getServers: () => api.get<McpServer[]>(mcpApi, '/api/mcp/servers'),

  getServer: (id: number) => api.get<McpServer>(mcpApi, `/api/mcp/servers/${id}`),

  createServer: (data: CreateMcpServerRequest) =>
    api.post<McpServer>(mcpApi, '/api/mcp/servers', data),

  updateServer: (id: number, data: CreateMcpServerRequest) =>
    api.put<McpServer>(mcpApi, `/api/mcp/servers/${id}`, data),

  deleteServer: (id: number) => api.delete<null>(mcpApi, `/api/mcp/servers/${id}`),

  getServerTools: (id: number) =>
    api.get<McpServerWithTools>(mcpApi, `/api/mcp/servers/${id}/tools`),

  toggleServer: (id: number, enabled: boolean) =>
    api.post<McpServer>(mcpApi, `/api/mcp/servers/${id}/toggle/${enabled}`),

  refreshServers: () => api.post<null>(mcpApi, '/api/mcp/servers/refresh'),
}
