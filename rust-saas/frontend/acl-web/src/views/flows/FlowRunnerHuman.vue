<script setup lang="ts">
import { ref, onMounted, computed ,nextTick} from 'vue'
import { GitFork, Search as SearchIcon } from 'lucide-vue-next'
import { Flow,FlowRuntime,FlowRuntimeNode } from '@/types/index.ts'
import { graphApi } from '@/vec/api'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import '@vue-flow/controls/dist/style.css'
import type { Entity, Relation } from '@/vec/types'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Send, Loader2, Bot, User, Wrench, Copy, Check, ChevronDown, Settings, Brain, Zap } from 'lucide-vue-next'
import { useSessionStore, useLlmStore, useAuthStore, useAgentStore } from '@/stores'
import { llmService, sessionService, type StreamResponse, authService } from '@/api'
import type { SessionItem, SessionType, LlmModel, Agent } from '@/types'

const route = useRoute()
const router = useRouter()
const sessionStore = useSessionStore()
const llmStore = useLlmStore()
const authStore = useAuthStore()
const agentStore = useAgentStore()

const messagesContainer = ref<HTMLElement | null>(null)
const inputMessage = ref('')
const sending = ref(false)
const streamingContent = ref('')
const copiedId = ref<number | null>(null)
const showModelDropdown = ref(false)
const showAgentDropdown = ref(false)
const showAgentDetails = ref(false)
const selectedModel = ref<LlmModel | null>(null)
const selectedAgent = ref<Agent | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)

const sessionId = computed(() => Number(route.params.id))
const formatTime = (dateStr: string) => new Date(dateStr).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })

let props = defineProps({
  flowId: {
    type: Number,
    default: 0
  },
  flowRuntimeId: {
    type: Number,
    default: 0
  }
})

const copyMessage = async (message: SessionItem) => {
  await navigator.clipboard.writeText(message.description)
  copiedId.value = message.id
  setTimeout(() => { copiedId.value = null }, 2000)
}

const selectModel = async (model: LlmModel) => { 
  selectedModel.value = model; 
  showModelDropdown.value = false 
}


onMounted(async () => {
  loading.value = true
  loadError.value = null

  if (!authStore.user && authStore.isAuthenticated) {
    try {
      const tokenData = JSON.parse(atob(authStore.accessToken!.split('.')[1]))
      const userInfo = await authService.getUser(tokenData.user_id)
      authStore.user = userInfo
    } catch {
      loadError.value = '加载用户信息失败'
      router.push('/login')
      return
    }
  }

  if (!authStore.user) {
    loadError.value = '请先登录'
    router.push('/login')
    return
  }

  try {
    await Promise.all([
      llmStore.fetchModels(),
    ])
    
    const session = sessionStore.currentSession
    if (session) {
      if (session.agent_id) {
        selectedAgent.value = agentStore.agents.find(a => a.id === session.agent_id) || null
      }
      if (session.model_id) {
        selectedModel.value = llmStore.models.find(m => m.id === session.model_id) || llmStore.defaultModel || llmStore.models[0] || null
      } else {
        selectedModel.value = llmStore.defaultModel || llmStore.models[0] || null
      }
    } else {
      selectedModel.value = llmStore.defaultModel || llmStore.models[0] || null
    }
    
    scrollToBottom()
  } catch (error) {
    loadError.value = error instanceof Error ? error.message : '加载会话数据失败'
  } finally {
    loading.value = false
  }
})

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center gap-4 p-4 bg-white border-b border-surface-200">
      <div class="flex-1">
        <p class="text-xs text-surface-400 mt-0.5 font-mono">{{ sessionStore.messages.length }} messages</p>
      </div>
      <!-- Model Selector -->
      <div class="relative">
        <button @click="showModelDropdown = !showModelDropdown; showAgentDropdown = false; showAgentDetails = false" class="flex items-center gap-2 px-3 py-2 bg-surface-50 border border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600">
          <Settings class="w-4 h-4 text-surface-400" />
          <span>{{ selectedModel?.name || '选择模型' }}</span>
          <ChevronDown class="w-4 h-4 text-surface-400" />
        </button>
        <div v-if="showModelDropdown" class="absolute right-0 top-full mt-2 w-64 bg-white border border-surface-200 rounded-lg shadow-lg z-50 max-h-64 overflow-y-auto">
          <div v-for="model in llmStore.models" :key="model.id">
            <button @click="selectModel(model)" :class="['w-full px-4 py-2 text-left text-sm hover:bg-surface-50 transition-colors', selectedModel?.id === model.id ? 'bg-primary-50 text-primary-600' : 'text-surface-700']">
              {{ model.name }}
              <span v-if="model.is_default" class="ml-2 text-xs text-green-500 font-mono">(default)</span>
            </button>
          </div>
        </div>
      </div>
    </div>
    <!-- Messages -->
    <div ref="messagesContainer" class="flex-1 overflow-y-auto p-4 space-y-4 bg-white">
      <div v-if="loading" class="flex items-center justify-center h-full">
        <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
      </div>

      <div v-else-if="loadError" class="flex flex-col items-center justify-center h-full text-center">
        <p class="text-red-500 mb-4">{{ loadError }}</p>
        <button @click="router.push('/sessions')" class="btn btn-primary">返回会话列表</button>
      </div>

      <template v-else>
        <div v-for="message in sessionStore.messages" :key="message.id" :class="['flex gap-3', message.session_type === 'User' ? 'flex-row-reverse' : '']">
          <div :class="['w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0', getAvatarClass(message.session_type)]">
            <component :is="getMessageIcon(message.session_type)" class="w-4 h-4" />
          </div>
          <div :class="['flex-1', message.session_type === 'User' ? 'max-w-xl text-right' : 'max-w-3xl']">
            <div :class="['rounded-xl p-4 border inline-block', getMessageClass(message.session_type)]">
              <p :class="['whitespace-pre-wrap text-sm leading-relaxed text-surface-700', message.session_type === 'User' ? 'text-right' : 'text-left']" v-html="parseMarkdown(message.description || '')"></p>
            </div>
            <div :class="['flex items-center gap-2 mt-1 text-xs text-surface-400', message.session_type === 'User' ? 'justify-end' : '']">
              <span>{{ formatTime(message.created_at) }}</span>
              <button v-if="message.session_type !== 'User'" @click="copyMessage(message)" class="p-1 hover:text-surface-700 transition-colors">
                <Check v-if="copiedId === message.id" class="w-3 h-3 text-green-500" />
                <Copy v-else class="w-3 h-3" />
              </button>
            </div>
          </div>
        </div>

        <!-- Streaming indicator -->
        <div v-if="streamingContent" class="flex gap-3">
          <div class="w-8 h-8 rounded-lg bg-surface-100 flex items-center justify-center text-surface-600">
            <Bot class="w-4 h-4" />
          </div>
          <div class="flex-1 max-w-3xl">
            <div class="border-surface-200 rounded-xl p-4 border">
              <p class="whitespace-pre-wrap text-sm leading-relaxed typing-cursor text-surface-700" v-html="parseMarkdown(streamingContent || '')"></p>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Input -->
    <div class="p-4 bg-white border-t border-surface-200">
      <form @submit.prevent="sendMessage" class="flex gap-3">
        <input
          v-model="inputMessage"
          :disabled="sending"
          placeholder="输入消息..."
          class="flex-1 input-base"
        />
        <button type="submit" :disabled="sending || !inputMessage.trim() || !selectedModel" class="btn btn-primary !px-6">
          <Loader2 v-if="sending" class="w-4 h-4 animate-spin" />
          <Send v-else class="w-4 h-4" />
        </button>
      </form>
    </div>
  </div>
    
    
</template>

<style scoped>
</style>