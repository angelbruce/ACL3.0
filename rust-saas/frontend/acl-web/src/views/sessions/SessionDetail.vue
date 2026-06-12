<script setup lang="ts">
import { onMounted, ref, nextTick, computed } from 'vue'
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
      sessionStore.fetchSession(sessionId.value),
      sessionStore.fetchMessages(sessionId.value),
      llmStore.fetchModels(),
      agentStore.fetchAgents(),
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

const getMessageIcon = (type: SessionType) => {
  switch (type) { case 'User': return User; case 'Assistant': return Bot; case 'Tool': return Wrench; default: return Bot }
}

const parseMarkdown = (text: string) => {
  let result = text
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\[DONE\]/g, '.')
  
  const lines = result.split('\n')
  const processedLines: string[] = []
  let inList = false
  
  for (const line of lines) {
    const headingMatch = line.match(/^(#{1,6})\s+(.+)$/)
    if (headingMatch) {
      if (inList) {
        processedLines.push('</div>')
        inList = false
      }
      const level = headingMatch[1].length
      processedLines.push(`<h${level} class="heading-${level}">${headingMatch[2].trim()}</h${level}>`)
    } else if (/^---+$/.test(line)) {
      if (inList) {
        processedLines.push('</div>')
        inList = false
      }
      processedLines.push('<hr class="divider" />')
    } else {
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
  }
  
  if (inList) {
    processedLines.push('</div>')
  }
  

  let start = false,head = false;
  for(let i = 0; i < processedLines.length; i++) {
    let line = processedLines[i]
    let match =  /([^\|]+?)\|/g;
    let matches =[... line.matchAll(match)];
    if(matches.length === 0) {
      if(start) {
        processedLines[i] = '</table>'+ processedLines[i]
      }
      start = false
      continue
    } 

    if(!start) {
      start = true
      head = true
    }

    let list = []
    for(let m of matches) {
      list.push(m[1].replace(':---', '').trim())
    }
   
    let flag = false;
    for(let v of list) {
      if(v !== '') {
        flag = true
        break
      }
    }

    if(!flag) {
      processedLines[i] = ""
      continue
    }

    console.log('list', list)

    let body='';
    if(head) {
      head = false
      body =  '<table class="flex-1 w-full wrap text-center table table-striped table-hover table-bordered table-sm table-responsive-md ">'
            + '<tr><td class="text-center font-bold border border-surface-900 px-2 py-2 bg-gray-600 text-white">' 
            + list.join('</td><td class="text-center font-bold border border-surface-900 px-2 py-2 bg-gray-600 text-white">') + '</td></tr>'
    }
    else {
      body = '<tr><td class="text-left border border-surface-900 px-2 py-2 bg-white">' 
      + list.join('</td><td class="text-left border border-surface-900 px-2 py-2 bg-white">') + '</td></tr>'
    }
    
    processedLines[i] = body
  }

  return processedLines.join('\n')
}

const getMessageClass = (type: SessionType) => {
  switch (type) {
    case 'User': return 'bg-primary-50 border-primary-100'
    case 'Assistant': return 'bg-surface-50 border-surface-200'
    case 'Tool': return 'bg-green-50 border-green-100'
    default: return 'bg-surface-50 border-surface-200'
  }
}

const getAvatarClass = (type: SessionType) => {
  switch (type) {
    case 'User': return 'bg-primary-100 text-primary-600'
    case 'Assistant': return 'bg-surface-100 text-surface-600'
    case 'Tool': return 'bg-green-100 text-green-600'
    default: return 'bg-surface-100 text-surface-600'
  }
}

const copyMessage = async (message: SessionItem) => {
  await navigator.clipboard.writeText(message.description)
  copiedId.value = message.id
  setTimeout(() => { copiedId.value = null }, 2000)
}

const selectModel = async (model: LlmModel) => { 
  selectedModel.value = model; 
  showModelDropdown.value = false 
  await updateSessionAgentModel()
}
const selectAgent = async (agent: Agent) => { 
  selectedAgent.value = agent; 
  showAgentDropdown.value = false; 
  await agentStore.fetchAgent(agent.id) 
  await updateSessionAgentModel()
}

const updateSessionAgentModel = async () => {
  if (!sessionStore.currentSession) return
  try {
    await sessionService.updateSession(sessionId.value, {
      agent_id: selectedAgent.value?.id || null,
      model_id: selectedModel.value?.id || null
    })
  } catch {
    // 忽略错误
  }
}

const sendMessage = async () => {
  if (!inputMessage.value.trim() || sending.value) return

  if (!authStore.user) { router.push('/login'); return }

  const userMessage = inputMessage.value.trim()
  inputMessage.value = ''

  try {
    await sessionStore.addMessage(sessionId.value, { description: userMessage, session_type: 'User' })
  } catch { return }

  sending.value = true
  streamingContent.value = ''

  try {
    if (!selectedModel.value) throw new Error('请先选择模型')

    const chatMessages = sessionStore.messages.map((m) => ({
      role: m.session_type === 'User' ? 'user' : m.session_type === 'Assistant' ? 'assistant' : 'system',
      content: m.description,
    }))

    const agentId = selectedAgent.value?.id

    await llmService.chatStream(
      { model_id: selectedModel.value.id, messages: chatMessages, agent_id: agentId, stream: true },
      (data: StreamResponse) => { streamingContent.value += data.content; nextTick(() => scrollToBottom()) },
      (error: Error) => { console.error('[ERROR] Stream error:', error) }
    )

    await sessionStore.addMessage(sessionId.value, { description: streamingContent.value, session_type: 'Assistant' })
  } catch (error) {
    await sessionStore.addMessage(sessionId.value, { description: `错误: ${error instanceof Error ? error.message : '未知错误'}`, session_type: 'System' })
  } finally {
    sending.value = false
    streamingContent.value = ''
    scrollToBottom()
  }
}

const formatTime = (dateStr: string) => new Date(dateStr).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center gap-4 p-4 bg-white border-b border-surface-200">
      <button @click="router.push('/sessions')" class="p-2 rounded-lg text-surface-400 hover:text-surface-700 hover:bg-surface-50 transition-colors">
        <ArrowLeft class="w-5 h-5" />
      </button>
      <div class="flex-1">
        <h1 class="font-semibold text-surface-800">{{ sessionStore.currentSession?.description || `会话 #${sessionId}` }}</h1>
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

      <!-- Agent Selector -->
      <div class="relative">
        <button @click="showAgentDropdown = !showAgentDropdown; showModelDropdown = false" class="flex items-center gap-2 px-3 py-2 bg-surface-50 border border-surface-200 rounded-lg hover:bg-surface-100 transition-colors text-sm text-surface-600">
          <Brain class="w-4 h-4 text-surface-400" />
          <span>{{ selectedAgent?.name || '选择 Agent' }}</span>
          <ChevronDown class="w-4 h-4 text-surface-400" />
        </button>
        <div v-if="showAgentDropdown" class="absolute right-0 top-full mt-2 w-64 bg-white border border-surface-200 rounded-lg shadow-lg z-50 max-h-64 overflow-y-auto">
          <div v-for="agent in agentStore.agents" :key="agent.id">
            <button @click="selectAgent(agent)" :class="['w-full px-4 py-2 text-left text-sm hover:bg-surface-50 transition-colors', selectedAgent?.id === agent.id ? 'bg-primary-50 text-primary-600' : 'text-surface-700']">{{ agent.name }}</button>
          </div>
        </div>
      </div>

      <button v-if="selectedAgent" @click="showAgentDetails = !showAgentDetails; showModelDropdown = false; showAgentDropdown = false" class="p-2 rounded-lg hover:bg-surface-50 transition-colors" :class="showAgentDetails ? 'text-primary-600' : 'text-surface-400'">
        <Zap class="w-5 h-5" />
      </button>
    </div>

    <!-- Agent Details Panel -->
    <div v-if="showAgentDetails && agentStore.currentAgent" class="border-b border-surface-200 bg-surface-50 p-4">
      <div class="space-y-4">
        <div>
          <h3 class="font-semibold text-primary-600 flex items-center gap-2"><Brain class="w-5 h-5" /> {{ agentStore.currentAgent.name }}</h3>
          <p v-if="agentStore.currentAgent.defination" class="text-sm text-surface-500 mt-1">{{ agentStore.currentAgent.defination }}</p>
        </div>
        <div v-if="agentStore.currentAgent.skills?.length">
          <h4 class="text-sm font-medium text-surface-600 mb-2">技能</h4>
          <div class="space-y-2">
            <div v-for="skill in agentStore.currentAgent.skills" :key="skill.id" class="p-2 bg-white rounded-lg text-xs text-surface-500 border border-surface-200">{{ skill.skill_prompt }}</div>
          </div>
        </div>
        <div v-if="agentStore.currentAgent.tools?.length">
          <h4 class="text-sm font-medium text-surface-600 mb-2">工具</h4>
          <div class="grid grid-cols-2 gap-2">
            <div v-for="tool in agentStore.currentAgent.tools" :key="tool.id" class="p-2 bg-green-50 border border-green-100 rounded-lg">
              <div class="font-medium text-green-600 text-sm">{{ tool.name }}</div>
              <div class="text-xs text-surface-500 mt-1">{{ tool.description }}</div>
            </div>
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
:deep(.heading-1) {
  font-size: 1.5rem;
  font-weight: bold;
  margin: 1rem 0 0.5rem;
  color: #1e293b;
}

:deep(.heading-2) {
  font-size: 1.25rem;
  font-weight: bold;
  margin: 0.75rem 0 0.5rem;
  color: #334155;
}

:deep(.heading-3) {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0.5rem 0 0.25rem;
  color: #475569;
}

:deep(.heading-4) {
  font-size: 1rem;
  font-weight: 600;
  margin: 0.5rem 0 0.25rem;
  color: #64748b;
}

:deep(.heading-5) {
  font-size: 0.875rem;
  font-weight: 600;
  margin: 0.25rem 0;
  color: #64748b;
}

:deep(.heading-6) {
  font-size: 0.875rem;
  font-weight: 500;
  margin: 0.25rem 0;
  color: #94a3b8;
}

:deep(.divider) {
  border: none;
  height: 1px;
  background: linear-gradient(to right, transparent, #cbd5e1, transparent);
  margin: 1rem 0;
}

:deep(.done-badge) {
  display: inline-block;
  padding: 0.25rem 0.75rem;
  background: linear-gradient(135deg, #10b981, #059669);
  color: white;
  font-size: 0.75rem;
  font-weight: 600;
  border-radius: 9999px;
  margin: 0 0.25rem;
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
}
</style>