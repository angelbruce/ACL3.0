// User types
export interface User {
  id: number
  email: string
  password_hash?: string
  created_at: string
}

// Auth types
export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  email: string
  password: string
}

export interface AuthResponse {
  access_token: string
  refresh_token: string
  user_id: number
}

// Agent types
export interface Agent {
  id: number
  name: string
  defination: string | null
  created_at: string
  updated_at: string
}

export interface AgentDetail extends Agent {
  tools: AgentTool[]
  skills: AgentSkill[]
  content_stores: ContentStoreConfig[]
}

export interface AgentTool {
  id: number
  agent_id: number
  name: string
  description: string
  input_schema: string
  output_schema: string
}

export interface AgentSkill {
  id: number
  agent_id: number
  skill_prompt: string
}

export interface ContentStoreConfig {
  id: number
  agent_id: number
  store_type: string
  config: string
}

export interface CreateAgentRequest {
  name: string
  defination?: string
  tools?: AgentToolCreate[]
  skills?: AgentSkillCreate[]
}

export interface AgentToolCreate {
  name: string;
  description: string;
  input_schema: string;
  output_schema: string;
  server_id?: number | null;
  server_name?: string;
}

export interface AgentSkillCreate {
  skill_prompt: string
}

// Flow types
export interface Flow {
  id: number
  name: string
  config: FlowConfigModel
  created_at: string
}

export interface FlowConfigModel {
  vertices: Vertex[]
  edges: Edge[]
}

export interface Vertex {
  id: string
  agent: number | null
  value: string
  type: string
  prompt?: string
  paths?: string[]
  degree?: number
  x?: number
  y?: number
}

export interface Edge {
  src: string
  target: string
  value: string
  style?: string
}

export interface FlowRuntime {
  id: number
  flow_id: number
  is_over: boolean
  created_at: string
}

export interface FlowRuntimeNode {
  id: number
  flow_runtime_id: number
  flow_id: number
  action_id: number
  action: string
  prompt?: string
  status: 'Running' | 'RunningOver' | 'Stop'
  next_choice?: string
  created_at: string,
  human: number
}

export interface CreateFlowRequest {
  name: string
  config: FlowConfigModel
}

// Session types
export interface Session {
  id: number
  user_id: number
  description: string | null
  agent_id: number | null
  model_id: number | null
  agent_name?: string
  model_name?: string
  created_at: string
}

export interface SessionItem {
  id: number
  session_id: number
  description: string
  session_type: SessionType
  created_at: string
}

export type SessionType = 'System' | 'User' | 'Assistant' | 'Tool'

export interface CreateSessionRequest {
  user_id: number
  description?: string
  agent_id?: number | null
  model_id?: number | null
}

export interface AddMessageRequest {
  description: string
  session_type: SessionType
}

// LLM types
export interface LlmModel {
  id: number
  name: string
  access_url: string
  api_key: string
  is_default: boolean
}

export interface CreateLlmModelRequest {
  name: string
  access_url: string
  api_key: string
  is_default: boolean
}

export interface ChatMessage {
  role: string
  content: string
}

export interface LlmRequest {
  model_id: number
  messages: ChatMessage[]
  agent_id?: number
  stream: boolean
  project_id?: number
}

// MCP types
export interface MCPTool {
  name: string
  description: string
  inputSchema: Record<string, unknown>
  outputSchema: Record<string, unknown>
  serverId?: number | null
}

export interface MCPToolCallResult {
  success: boolean
  content: string
  error?: string
}

export interface McpServer {
  id: number
  name: string
  description: string | null
  server_type: string
  url: string
  headers: Record<string, string> | null
  enabled: boolean
  stateless: boolean
  created_at: string
  updated_at: string
}

export interface CreateMcpServerRequest {
  name: string
  description?: string
  server_type: string
  url: string
  headers?: Record<string, string>
  enabled?: boolean
  stateless?: boolean
}

export interface McpServerWithTools {
  server: McpServer
  tools: MCPTool[]
}

// API Response types
export interface ErrorResponse {
  error: string
  message: string
}

export interface ApiResponse<T> {
  data?: T
  error?: string
}

// Workspace/Project types
export type ProjectPurpose = 'article' | 'coding'

export interface Project {
  id: number
  name: string
  purpose: ProjectPurpose
  description: string | null
  model_id: number | null
  agent_id: number | null
  model_name?: string
  agent_name?: string
  last_accessed_at: string
  created_at: string
  updated_at: string
}

export interface ProjectFile {
  id: number
  project_id: number
  name: string
  content: string | null
  directory: string | null
  created_at: string
  updated_at: string
}

export interface ProjectSettings {
  model_id: number | null
  agent_id: number | null
  name: string | null
  description: string | null
}

export interface CreateProjectRequest {
  name: string
  purpose: ProjectPurpose
  description?: string
  model_id?: number | null
  agent_id?: number | null
}

export interface UpdateProjectRequest {
  name: string
  description: string | null
  model_id: number | null
  agent_id: number | null
}

export interface ProjectChatMessage {
  id: number
  project_id: number
  content: string
  reasoning_content?: string
  role: 'user' | 'assistant' | 'system'
  created_at: string
}

export interface ProjectChatRequest {
  project_id: number
  model_id: number
  agent_id?: number
  message: string
}

export interface ProjectContainerConfig {
  id: number
  project_id: number
  project_dir: string
  published_ports: string
  image_name: string
  volumes: string
  environment: string
  command: string
  working_dir: string
  tags: string
  container_name: string
  cpu_usage: string
  memory_usage: string
  creator_id: number
  created_at: string
  updated_at: string
}
// Admin types
export * from './admin'
