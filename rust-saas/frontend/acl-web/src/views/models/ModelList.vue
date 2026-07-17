<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Plus, Settings, Trash2, Star, Loader2, Eye, EyeOff, Edit2 } from 'lucide-vue-next'
import { useLlmStore } from '@/stores'
import type { LlmModel } from '@/types'

const llmStore = useLlmStore()

const showForm = ref(false)
const editingModel = ref<LlmModel | null>(null)
const showApiKey = ref<number | null>(null)
const deletingId = ref<number | null>(null)

const formData = ref<Partial<LlmModel>>({ name: '', access_url: '', api_key: '', is_default: false })

onMounted(async () => { await llmStore.fetchModels() })

const openForm = (model?: LlmModel) => {
  if (model) { editingModel.value = model; formData.value = { ...model } }
  else { editingModel.value = null; formData.value = { name: '', access_url: '', api_key: '', is_default: false } }
  showForm.value = true
}
const closeForm = () => { showForm.value = false; editingModel.value = null }
const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.access_url || !formData.value.api_key) { alert('请填写所有必填字段'); return }
  const request = { name: formData.value.name, access_url: formData.value.access_url, api_key: formData.value.api_key, is_default: formData.value.is_default }
  try {
    if (editingModel.value) await llmStore.updateModel(editingModel.value.id, request)
    else await llmStore.createModel(request)
    closeForm()
  } catch {}
}
const deleteModel = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个模型吗?')) {
    deletingId.value = id
    try { await llmStore.deleteModel(id) }
    finally { deletingId.value = null }
  }
}
const setDefault = async (id: number, event: Event) => {
  event.stopPropagation()
  await llmStore.setDefaultModel(id)
}
const toggleApiKey = (id: number) => { showApiKey.value = showApiKey.value === id ? null : id }
const maskApiKey = (key: string) => { if (key.length <= 8) return '········'; return key.slice(0, 4) + '········' + key.slice(-4) }
const formatDate = (dateStr: string) => new Date(dateStr).toLocaleDateString('zh-CN', { year: 'numeric', month: 'short', day: 'numeric' })
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">模型配置</h1>
        <p class="page-subtitle">llm model configuration</p>
      </div>
      <button @click="openForm()" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        添加模型
      </button>
    </div>

    <div v-if="llmStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="llmStore.models.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <Settings class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无模型</h3>
      <p class="text-surface-400 mb-6">添加一个 LLM 模型来开始对话</p>
      <button @click="openForm()" class="btn btn-primary"><Plus class="w-4 h-4" /> 添加模型</button>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <div
        v-for="model in llmStore.models"
        :key="model.id"
        class="card p-5 group relative"
      >
        <span v-if="model.is_default" class="absolute top-3 right-3 tag tag-amber">默认</span>
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-primary-100 to-accent-100 flex items-center justify-center">
              <Settings class="w-6 h-6 text-primary-500" />
            </div>
            <div class="min-w-0 flex-1">
              <h3 class="font-semibold text-base text-surface-800 truncate w-full max-w-[180px]" :title="model.name">{{ model.name }}</h3>
              <!-- <p class="text-xs text-surface-400 mt-0.5">{{ formatDate(model.created_at) }}</p> -->
            </div>
          </div>
        </div>

        <div class="mb-3" v-if="model.access_url && model.access_url.length > 0">
          <p class="text-xs text-surface-400 mb-1">模型位置</p>
          <p class="text-xs text-surface-600 truncate mb-2">{{ model.access_url }}</p>
          <p class="text-xs text-surface-400 mb-1">API Key</p>
          <div class="flex items-center gap-2 p-2 bg-surface-50 rounded-lg">
            <code class="text-xs font-mono text-surface-600 flex-1 truncate">{{ showApiKey === model.id ? model.api_key : maskApiKey(model.api_key) }}</code>
            <button @click.stop="toggleApiKey(model.id)" class="p-1 text-surface-400 hover:text-surface-700 rounded hover:bg-surface-100 transition-colors">
              <EyeOff v-if="showApiKey === model.id" class="w-3.5 h-3.5" />
              <Eye v-else class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>


         <div class="mb-3" v-if="!model.access_url || model.access_url.length === 0">
          <p class="text-xs text-surface-400 mb-1">模型位置</p>
          <p class="text-xs text-surface-600 truncate mb-2">本地模型</p>
          <p class="text-xs text-surface-400 mb-1">API Key</p>
          <div class="flex items-center gap-2 p-2 bg-surface-50 rounded-lg">
            <code class="text-xs font-mono text-surface-600 flex-1 truncate">{{ showApiKey === model.id ? model.api_key : maskApiKey(model.api_key) }}</code>
            <button @click.stop="toggleApiKey(model.id)" class="p-1 text-surface-400 hover:text-surface-700 rounded hover:bg-surface-100 transition-colors">
              <EyeOff v-if="showApiKey === model.id" class="w-3.5 h-3.5" />
              <Eye v-else class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <div class="flex items-center gap-2 pt-2 border-t border-surface-100">
          <button @click.stop="openForm(model)" class="btn btn-outline flex-1 justify-center text-sm">
            <Edit2 class="w-3.5 h-3.5" />
            编辑
          </button>
          <button v-if="!model.is_default" @click.stop="setDefault(model.id, $event)" class="btn btn-outline text-sm justify-center text-amber-600 border-amber-200 hover:bg-amber-50">
            <Star class="w-3.5 h-3.5" />
          </button>
          <button @click.stop="deleteModel(model.id, $event)" :disabled="deletingId === model.id" class="btn btn-danger text-sm justify-center">
            <Loader2 v-if="deletingId === model.id" class="w-3.5 h-3.5 animate-spin" />
            <Trash2 v-else class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>

    <!-- Form dialog -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 bg-black/30 flex items-center justify-center z-50 p-4" @click.self="closeForm">
        <div class="card p-6 w-full max-w-lg animate-fade-in">
          <h2 class="text-lg font-semibold mb-4 text-surface-800">{{ editingModel ? '编辑模型' : '添加模型' }}</h2>
          <form @submit.prevent="handleSubmit" class="space-y-4">
            <div><label class="block text-sm font-medium text-surface-700 mb-2">名称 <span class="text-red-500">*</span></label><input v-model="formData.name" type="text" placeholder="例如: GPT-4" class="input-base w-full" /></div>
            <div><label class="block text-sm font-medium text-surface-700 mb-2">API URL <span class="text-red-500">*</span></label><input v-model="formData.access_url" type="url" placeholder="https://api.openai.com/v1" class="input-base w-full" /></div>
            <div><label class="block text-sm font-medium text-surface-700 mb-2">API Key <span class="text-red-500">*</span></label><input v-model="formData.api_key" type="password" placeholder="sk-········" class="input-base w-full" /></div>
            <div class="flex items-center gap-2"><input v-model="formData.is_default" type="checkbox" id="is_default" class="w-4 h-4 rounded border-surface-300 text-primary-500" /><label for="is_default" class="text-sm text-surface-700">设为默认模型</label></div>
            <div class="flex gap-3 pt-2"><button type="button" @click="closeForm" class="btn btn-outline flex-1 justify-center">取消</button><button type="submit" class="btn btn-primary flex-1 justify-center">{{ editingModel ? '保存' : '添加' }}</button></div>
          </form>
        </div>
      </div>
    </Teleport>
  </div>
</template>