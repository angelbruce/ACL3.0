import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Session, SessionItem, CreateSessionRequest, AddMessageRequest } from '@/types'
import { sessionService } from '@/api'

export const useSessionStore = defineStore('session', () => {
  const sessions = ref<Session[]>([])
  const currentSession = ref<Session | null>(null)
  const messages = ref<SessionItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const fetchSessions = async () => {
    loading.value = true
    error.value = null
    try {
      sessions.value = await sessionService.getSessions()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch sessions'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchSession = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      currentSession.value = await sessionService.getSession(id)
      return currentSession.value
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch session'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchMessages = async (sessionId: number) => {
    loading.value = true
    error.value = null
    try {
      messages.value = await sessionService.getMessages(sessionId)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch messages'
      throw err
    } finally {
      loading.value = false
    }
  }

  const createSession = async (data: CreateSessionRequest) => {
    loading.value = true
    error.value = null
    try {
      const session = await sessionService.createSession(data)
      sessions.value.push(session)
      return session
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to create session'
      throw err
    } finally {
      loading.value = false
    }
  }

  const addMessage = async (sessionId: number, data: AddMessageRequest) => {
    error.value = null
    try {
      const message = await sessionService.addMessage(sessionId, data)
      messages.value.push(message)
      return message
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to add message'
      throw err
    }
  }

  const deleteSession = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      await sessionService.deleteSession(id)
      sessions.value = sessions.value.filter((s) => s.id !== id)
      if (currentSession.value?.id === id) {
        currentSession.value = null
        messages.value = []
      }
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to delete session'
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    sessions,
    currentSession,
    messages,
    loading,
    error,
    fetchSessions,
    fetchSession,
    fetchMessages,
    createSession,
    addMessage,
    deleteSession,
  }
})
