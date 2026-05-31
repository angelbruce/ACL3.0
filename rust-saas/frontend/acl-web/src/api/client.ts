import axios, { AxiosInstance, AxiosError, InternalAxiosRequestConfig } from 'axios'

let host = window.location.host || 'localhost:3000'
let URL_AUTH_API_BASE= `http://localhost:8080`;
  let URL_AGENT_API_BASE= `http://localhost:8081`;
  let URL_FLOW_API_BASE= `http://localhost:8082`;
  let URL_SESSION_API_BASE= `http://localhost:8083`;
  let URL_LLM_API_BASE= `http://localhost:8084`;
  let URL_MCP_API_BASE= `http://localhost:8085`;
  let URL_ADMIN_API_BASE= `http://localhost:8086`;
  let URL_WORKSPACE_API_BASE= `http://localhost:8087`;  

if(host !== 'localhost:3000') {
   URL_AUTH_API_BASE= `http://${host}/foreign/auth`;
   URL_AGENT_API_BASE= `http://${host}/foreign/agents`;
   URL_FLOW_API_BASE= `http://${host}/foreign/flows`;
   URL_SESSION_API_BASE= `http://${host}/foreign/sessions`;
   URL_LLM_API_BASE= `http://${host}/foreign/llms`;
   URL_MCP_API_BASE= `http://${host}/foreign/mcp`;
   URL_ADMIN_API_BASE= `http://${host}/foreign/admin`;
   URL_WORKSPACE_API_BASE= `http://${host}/foreign/workspace`;
}


// API Base URLs
export const API_BASE = {
  AUTH: import.meta.env.VITE_API_AUTH_URL || URL_AUTH_API_BASE,
  AGENT: import.meta.env.VITE_API_AGENT_URL || URL_AGENT_API_BASE,
  FLOW: import.meta.env.VITE_API_FLOW_URL || URL_FLOW_API_BASE,
  SESSION: import.meta.env.VITE_API_SESSION_URL || URL_SESSION_API_BASE,
  LLM: import.meta.env.VITE_API_LLM_URL || URL_LLM_API_BASE,
  MCP: import.meta.env.VITE_API_MCP_URL || URL_MCP_API_BASE,
  ADMIN: import.meta.env.VITE_API_ADMIN_URL || URL_ADMIN_API_BASE,
  WORKSPACE: import.meta.env.VITE_API_WORKSPACE_URL || URL_WORKSPACE_API_BASE,
}

// Create axios instances for each service
const createApiClient = (baseURL: string): AxiosInstance => {
  const client = axios.create({
    baseURL,
    timeout: 30000,
    headers: {
      'Content-Type': 'application/json',
    },
  })

  // Request interceptor
  client.interceptors.request.use(
    (config: InternalAxiosRequestConfig) => {
      const token = localStorage.getItem('access_token')
      console.log(token)
      if (token && config.headers) {
        config.headers.Authorization = `Bearer ${token}`
      }
      return config
    },
    (error) => Promise.reject(error)
  )

  // Response interceptor
  client.interceptors.response.use(
    (response) => response,
    async (error: AxiosError) => {
      if (error.response?.status === 401) {
        // Try to refresh token
        const refreshToken = localStorage.getItem('refresh_token')
        if (refreshToken) {
          try {
            const response = await axios.post(`${API_BASE.AUTH}/api/auth/refresh`, {
              refresh_token: refreshToken,
            })
            const { access_token, refresh_token } = response.data
            localStorage.setItem('access_token', access_token)
            localStorage.setItem('refresh_token', refresh_token)

            // Retry original request
            if (error.config && error.config.headers) {
              error.config.headers.Authorization = `Bearer ${access_token}`
              return axios(error.config)
            }
          } catch (refreshError) {
            // Refresh failed, redirect to login
            localStorage.removeItem('access_token')
            localStorage.removeItem('refresh_token')
            window.location.href = '/login'
          }
        } else {
          window.location.href = '/login'
        }
      }
      return Promise.reject(error)
    }
  )

  return client
}

// Export API clients
export const authApi = createApiClient(API_BASE.AUTH)
export const agentApi = createApiClient(API_BASE.AGENT)
export const flowApi = createApiClient(API_BASE.FLOW)
export const sessionApi = createApiClient(API_BASE.SESSION)
export const llmApi = createApiClient(API_BASE.LLM)
export const mcpApi = createApiClient(API_BASE.MCP)
export const workspaceApi = createApiClient(API_BASE.WORKSPACE)

// Generic API methods
export const api = {
  get: async <T>(client: AxiosInstance, url: string): Promise<T> => {
    const response = await client.get<T>(url)
    return response.data
  },

  post: async <T>(client: AxiosInstance, url: string, data?: unknown): Promise<T> => {
    const response = await client.post<T>(url, data)
    return response.data
  },

  put: async <T>(client: AxiosInstance, url: string, data?: unknown): Promise<T> => {
    const response = await client.put<T>(url, data)
    return response.data
  },

  delete: async <T>(client: AxiosInstance, url: string): Promise<T> => {
    const response = await client.delete<T>(url)
    return response.data
  },
}
