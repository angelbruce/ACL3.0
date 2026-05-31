<script setup lang="ts">
import { onMounted, ref, onUnmounted, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowLeft, Play, Square, Loader2, Clock, CheckCircle, XCircle,
  Send, Bot, User, Wrench, Copy, Check,
} from 'lucide-vue-next'
import { useFlowStore, useLlmStore } from '@/stores'
import { llmService, type StreamResponse } from '@/api'
import type { FlowRuntimeNode, ChatMessage } from '@/types'

const route = useRoute()
const router = useRouter()
const flowStore = useFlowStore()
const llmStore = useLlmStore()

const flowId = Number(route.params.id)

// -- 左侧：工作流执行状态 --
const loading = ref(true)
const nodes = ref<FlowRuntimeNode[]>([])
const pollingInterval = ref<number | null>(null)

onMounted(async () => {
  await Promise.all([
    fetchRuntime(),
    llmStore.fetchModels(),
  ])
  pollingInterval.value = window.setInterval(fetchRuntime, 2000)
})

onUnmounted(() => {
  if (pollingInterval.value) {
    clearInterval(pollingInterval.value)
  }
})

const fetchRuntime = async () => {
  try {
    await flowStore.fetchFlow(flowId)
    if (flowStore.runtimes.length > 0) {
      const runtime = await flowStore.fetchRuntime(flowStore.runtimes[0].id)
      if (runtime) {
        nodes.value = runtime.nodes
      }
    }
  } finally {
    loading.value = false
  }
}

const startFlow = async () => {
  await flowStore.startFlow(flowId)
  await fetchRuntime()
}

const stopFlow = async () => {
  if (flowStore.runtimes.length > 0) {
    await flowStore.stopFlow(flowStore.runtimes[0].id)
    await fetchRuntime()
  }
}

const currentRuntime = () => flowStore.runtimes.find((r) => !r.is_over)

const getStatusIcon = (status: string) => {
  switch (status) {
    case 'Running': return Loader2
    case 'RunningOver': return CheckCircle
    case 'Stop': return XCircle
    default: return Clock
  }
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'Running': return 'text-blue-600'
    case 'RunningOver': return 'text-emerald-600'
    case 'Stop': return 'text-red-500'
    default: return 'text-surface-400'
  }
}

const formatTime = (dateStr: string) => {
  return new Date(dateStr).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

// -- 右侧：聊天交互 --
interface ChatItem {
  id: number
  role: 'user' | 'assistant' | 'tool'
  content: string
  created_at: string
}

const messages = ref<ChatItem[]>([])
const inputMessage = ref('')
const sending = ref(false)
const streamingContent = ref('')
const copiedId = ref<number | null>(null)
const messagesContainer = ref<HTMLElement | null>(null)
let msgIdCounter = 0

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

const parseMarkdown = (text: string) => {
  let result = text.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
  const lines = result.split('\n')
  const processedLines: string[] = []
  let inList = false

  for (const line of lines) {
    const listMatch = line.match(/^\*\s+(.+)$/)
    if (listMatch) {
      if (!inList) {
        processedLines.push('<div class="list-container">')
        inList = true
      }
      processedLines.push(`<div class="list-item" style="display:block;">➢ ${listMatch[1].trim()}</div>`)
    } else {
      if (inList) {
        processedLines.push('</div>')
        inList = false
      }
      processedLines.push(line.replace(/\*(.+?)\*/g, '<em>$1</em>'))
    }
  }
  if (inList) processedLines.push('</div>')
  return processedLines.join('\n')
}

const getAvatarClass = (role: string) => {
  switch (role) {
    case 'user': return 'bg-primary-100 text-primary-600'
    case 'assistant': return 'bg-surface-100 text-surface-600'
    case 'tool': return 'bg-green-100 text-green-600'
    default: return 'bg-surface-100 text-surface-600'
  }
}

const getMessageClass = (role: string) => {
  switch (role) {
    case 'user': return 'bg-primary-50 border-primary-100'
    case 'assistant': return 'bg-surface-50 border-surface-200'
    case 'tool': return 'bg-green-50 border-green-100'
    default: return 'bg-surface-50 border-surface-200'
  }
}

const getMessageIcon = (role: string) => {
  switch (role) {
    case 'user': return User
    case 'assistant': return Bot
    case 'tool': return Wrench
    default: return Bot
  }
}

const copyMessage = async (msg: ChatItem) => {
  await navigator.clipboard.writeText(msg.content)
  copiedId.value = msg.id
  setTimeout(() => { copiedId.value = null }, 2000)
}

const sendMessage = async () => {
  if (!inputMessage.value.trim() || sending.value) return

  const model = llmStore.defaultModel || llmStore.models[0]
  if (!model) return

  const userMessage = inputMessage.value.trim()
  inputMessage.value = ''

  messages.value.push({
    id: ++msgIdCounter,
    role: 'user',
    content: userMessage,
    created_at: new Date().toISOString(),
  })
  scrollToBottom()

  sending.value = true
  streamingContent.value = ''

  try {
    const chatMessages: ChatMessage[] = messages.value.map((m) => ({
      role: m.role,
      content: m.content,
    }))

    await llmService.chatStream(
      {
        model_id: model.id,
        messages: chatMessages,
        stream: true,
      },
      (data: StreamResponse) => {
        streamingContent.value += data.content
        nextTick(() => scrollToBottom())
      },
      (error: Error) => {
        console.error('[ERROR] Stream error:', error)
      }
    )

    messages.value.push({
      id: ++msgIdCounter,
      role: 'assistant',
      content: streamingContent.value,
      created_at: new Date().toISOString(),
    })
  } catch (error) {
    messages.value.push({
      id: ++msgIdCounter,
      role: 'tool',
      content: `错误: ${error instanceof Error ? error.message : '未知错误'}`,
      created_at: new Date().toISOString(),
    })
  } finally {
    sending.value = false
    streamingContent.value = ''
    scrollToBottom()
  }
}
</script>

<template>
  <div class="flex h-full">
    <!-- ====== 左侧：工作流执行状态 ====== -->
    <div class="w-1/2 min-w-[380px] border-r border-surface-200 flex flex-col">
      <div class="p-4 border-b border-surface-200">
        <div class="flex items-center gap-4 mb-4">
          <button
            @click="router.push('/flows')"
            class="p-2 rounded-lg hover:bg-surface-100 transition-colors text-surface-500"
          >
            <ArrowLeft class="w-5 h-5" />
          </button>
          <div class="flex-1">
            <h1 class="text-xl font-bold text-surface-900">{{ flowStore.currentFlow?.name || '工作流执行' }}</h1>
            <p class="text-surface-500 text-sm mt-0.5">
              运行时 ID: {{ currentRuntime()?.id || '无' }}
            </p>
          </div>
        </div>
        <div class="flex gap-2">
          <button
            v-if="!currentRuntime()"
            @click="startFlow"
            class="btn btn-primary flex items-center gap-2 !bg-emerald-500 text-sm"
          >
            <Play class="w-4 h-4" /> 启动工作流
          </button>
          <button
            v-else
            @click="stopFlow"
            class="btn btn-danger flex items-center gap-2 text-sm"
          >
            <Square class="w-4 h-4" /> 停止工作流
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        <div v-if="loading" class="flex items-center justify-center py-12">
          <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
        </div>

        <template v-else>
          <div v-if="currentRuntime()" class="mb-4 card flex items-center gap-3 p-3">
            <div
              :class="[
                'w-3 h-3 rounded-full',
                currentRuntime()?.is_over ? 'bg-red-500' : 'bg-emerald-500 animate-pulse',
              ]"
            />
            <span class="font-medium text-surface-800 text-sm">
              {{ currentRuntime()?.is_over ? '已结束' : '运行中' }}
            </span>
            <span class="text-xs text-surface-500">
              开始于 {{ formatTime(currentRuntime()!.created_at) }}
            </span>
          </div>

          <div v-else class="card p-8 text-center">
            <Clock class="w-12 h-12 mx-auto text-surface-300 mb-3" />
            <h3 class="text-base font-medium text-surface-700 mb-1">工作流未运行</h3>
            <p class="text-sm text-surface-500">点击启动按钮开始执行工作流</p>
          </div>

          <div v-if="nodes.length > 0" class="space-y-3">
            <h2 class="text-sm font-semibold text-surface-800">执行节点</h2>
            <div class="space-y-2">
              <div
                v-for="node in nodes"
                :key="node.id"
                class="card flex items-center gap-3 p-3"
              >
                <div :class="['w-9 h-9 rounded-lg flex items-center justify-center', getStatusColor(node.status).replace('text-', 'bg-') + '/20']">
                  <component
                    :is="getStatusIcon(node.status)"
                    :class="['w-4 h-4', getStatusColor(node.status), node.status === 'Running' ? 'animate-spin' : '']"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <p class="font-medium text-sm text-surface-800">{{ node.action }}</p>
                  <p v-if="node.prompt" class="text-xs text-surface-500 mt-0.5 truncate">
                    {{ node.prompt }}
                  </p>
                </div>
                <div class="text-right flex-shrink-0">
                  <p :class="['text-xs font-medium', getStatusColor(node.status)]">
                    {{ node.status === 'Running' ? '运行中' : node.status === 'RunningOver' ? '完成' : '停止' }}
                  </p>
                  <p class="text-xs text-surface-400 mt-0.5">
                    {{ formatTime(node.created_at) }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- ====== 右侧：聊天交互界面 ====== -->
    <div class="w-1/2 flex flex-col min-w-[380px]">
      <div class="flex items-center p-3 bg-white border-b border-surface-200">
        <h2 class="font-semibold text-surface-800">交互</h2>
      </div>

      <div ref="messagesContainer" class="flex-1 overflow-y-auto p-3 space-y-3">
        <div v-if="messages.length === 0 && !streamingContent" class="flex flex-col items-center justify-center h-full text-center">
          <Bot class="w-10 h-10 text-surface-300 mb-2" />
          <p class="text-sm text-surface-500">发送消息开始交互</p>
        </div>

        <template v-else>
          <div
            v-for="message in messages"
            :key="message.id"
            :class="['flex gap-2', message.role === 'user' ? 'flex-row-reverse' : '']"
          >
            <div :class="['w-7 h-7 rounded-lg flex items-center justify-center flex-shrink-0', getAvatarClass(message.role)]">
              <component :is="getMessageIcon(message.role)" class="w-3.5 h-3.5" />
            </div>
            <div :class="['flex-1 min-w-0', message.role === 'user' ? 'text-right' : '']">
              <div :class="['rounded-xl px-3 py-2 border inline-block text-left', getMessageClass(message.role)]">
                <p :class="['whitespace-pre-wrap text-sm leading-relaxed text-surface-700']" v-html="parseMarkdown(message.content || '')"></p>
              </div>
              <div :class="['flex items-center gap-1.5 mt-0.5 text-xs text-surface-400', message.role === 'user' ? 'justify-end' : '']">
                <span>{{ formatTime(message.created_at) }}</span>
                <button v-if="message.role !== 'user'" @click="copyMessage(message)" class="p-0.5 hover:text-surface-700 transition-colors">
                  <Check v-if="copiedId === message.id" class="w-3 h-3 text-green-500" />
                  <Copy v-else class="w-3 h-3" />
                </button>
              </div>
            </div>
          </div>

          <div v-if="streamingContent" class="flex gap-2">
            <div class="w-7 h-7 rounded-lg bg-surface-100 flex items-center justify-center text-surface-600">
              <Bot class="w-3.5 h-3.5" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="bg-surface-50 border-surface-200 rounded-xl px-3 py-2 border">
                <p class="whitespace-pre-wrap text-sm leading-relaxed typing-cursor text-surface-700" v-html="parseMarkdown(streamingContent || '')"></p>
              </div>
            </div>
          </div>
        </template>
      </div>

      <div class="p-3 bg-white border-t border-surface-200">
        <form @submit.prevent="sendMessage" class="flex gap-2">
          <input
            v-model="inputMessage"
            :disabled="sending"
            placeholder="输入消息..."
            class="flex-1 input-base text-sm"
          />
          <button
            type="submit"
            :disabled="sending || !inputMessage.trim() || (!llmStore.defaultModel && !llmStore.models[0])"
            class="btn btn-primary !px-4"
          >
            <Loader2 v-if="sending" class="w-4 h-4 animate-spin" />
            <Send v-else class="w-4 h-4" />
          </button>
        </form>
      </div>
    </div>
  </div>
</template>

<style scoped>
</style>