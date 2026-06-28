import { llmApi, api } from './client'
import type { LlmModel, LlmRequest, ChatMessage, MCPTool, CreateLlmModelRequest } from '@/types'

export interface ChatResponse {
  content: string
  tool_calls?: { name: string; arguments: unknown }[]
}

export interface StreamResponse {
  content: string
  reasoning_content?: string
  tool_calls?: { name: string; arguments: unknown }[]
  finish_reason?: string
}

export const llmService = {
  getModels: () => api.get<LlmModel[]>(llmApi, '/api/models'),

  getModel: (id: number) => api.get<LlmModel>(llmApi, `/api/models/${id}`),

  createModel: (model: CreateLlmModelRequest) => api.post<LlmModel>(llmApi, '/api/models', model),

  updateModel: (id: number, model: CreateLlmModelRequest) =>
    api.put<LlmModel>(llmApi, `/api/models/${id}`, model),

  deleteModel: (id: number) => api.delete<null>(llmApi, `/api/models/${id}`),

  chat: (request: LlmRequest) => api.post<ChatResponse>(llmApi, '/api/chat', request),

  chatStream: async (
    request: LlmRequest,
    onMessage: (data: StreamResponse) => void,
    onError?: (error: Error) => void
  ): Promise<void> => {
    const token = localStorage.getItem('access_token')
    const response = await fetch(`${llmApi.defaults.baseURL}/api/chat/stream`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(request),
    })

    if (!response.body) {
      onError?.(new Error('No response body'))
      return
    }

    const reader = response.body.getReader()
    const decoder = new TextDecoder()

    return new Promise((resolve, reject) => {
      const processStream = () => {
        reader.read().then(({ done, value }) => {
          if (done) {
            resolve()
            return
          }

          const chunk = decoder.decode(value, { stream: true })
          const lines = chunk.split('\n')

          for (const line of lines) {
            if (line.startsWith('data: ')) {
              const dataStr = line.slice(6).trim()
              if (dataStr === '[DONE]' || dataStr === 'DONE') {
                continue
              }
              try {
                const data = JSON.parse(dataStr) as StreamResponse
                onMessage(data)
              } catch {
                // Ignore parse errors
              }
            }
          }

          processStream()
        }).catch((error) => {
          onError?.(error)
          reject(error)
        })
      }

      processStream()
    })
  },
}
