import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { LlmModel, CreateLlmModelRequest } from '@/types'
import { llmService } from '@/api'

export const useLlmStore = defineStore('llm', () => {
  const models = ref<LlmModel[]>([])
  const currentModel = ref<LlmModel | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const defaultModel = computed(() => models.value.find((m) => m.is_default) || null)

  const fetchModels = async () => {
    loading.value = true
    error.value = null
    try {
      models.value = await llmService.getModels()
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch models'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchModel = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      currentModel.value = await llmService.getModel(id)
      return currentModel.value
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch model'
      throw err
    } finally {
      loading.value = false
    }
  }

  const createModel = async (model: CreateLlmModelRequest) => {
    loading.value = true
    error.value = null
    try {
      const created = await llmService.createModel(model)
      models.value.push(created)
      return created
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to create model'
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateModel = async (id: number, model: CreateLlmModelRequest) => {
    loading.value = true
    error.value = null
    try {
      const updated = await llmService.updateModel(id, model)
      const index = models.value.findIndex((m) => m.id === id)
      if (index !== -1) {
        models.value[index] = updated
      }
      if (currentModel.value?.id === id) {
        currentModel.value = updated
      }
      return updated
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to update model'
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteModel = async (id: number) => {
    loading.value = true
    error.value = null
    try {
      await llmService.deleteModel(id)
      models.value = models.value.filter((m) => m.id !== id)
      if (currentModel.value?.id === id) {
        currentModel.value = null
      }
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to delete model'
      throw err
    } finally {
      loading.value = false
    }
  }

  const setDefaultModel = async (id: number) => {
    // Update all models to non-default, then set the selected one as default
    const updatedModels = models.value.map((m) => ({
      ...m,
      is_default: m.id === id,
    }))
    models.value = updatedModels

    // Persist to backend
    for (const model of updatedModels) {
      const request: CreateLlmModelRequest = {
        name: model.name,
        access_url: model.access_url,
        api_key: model.api_key,
        is_default: model.is_default,
      }
      await llmService.updateModel(model.id, request)
    }
  }

  return {
    models,
    currentModel,
    defaultModel,
    loading,
    error,
    fetchModels,
    fetchModel,
    createModel,
    updateModel,
    deleteModel,
    setDefaultModel,
  }
})
