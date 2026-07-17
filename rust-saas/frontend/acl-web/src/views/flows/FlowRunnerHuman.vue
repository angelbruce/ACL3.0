<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import { ArrowLeft, Bot, User, Loader2 } from 'lucide-vue-next'
import { useFlowStore } from '@/stores'
import {parseMarkdown} from "@/utils/markdown"

const route = useRoute()
const flowStore = useFlowStore()

const messagesContainer = ref<HTMLElement | null>(null)
const loading = ref(false)
const sessions = ref<any[]>([])
const currentSession = ref<any>(null)
const sessionItems = ref<any[]>([])
const pollingInterval = ref<number | null>(null)
let lastSessionItemCount = 0
let lastSessionId = ref<number>(0);
let lastRuntimeId = ref<number>(0);
const props = defineProps({
  flowId: {
    type: Number,
    default: 0
  },
  flowRuntimeId: {
    type: Number,
    default: 0
  }
})

const fetchSessions = async () => {
  if (props.flowRuntimeId === 0) return
 

  loading.value = true
  try {
    sessions.value  = await flowStore.getFlowRuntimeSessions(props.flowRuntimeId)
    if (sessions.value.length > 0) {
      if (!currentSession.value || !sessions.value.find(s => s.id === currentSession.value.id)) {
        currentSession.value = sessions.value[sessions.value.length - 1]
      }

      await fetchSessionItems(currentSession.value.id)
    } else {
      sessionItems.value = []
    }
  } catch (error) {
    console.error('获取会话失败:', error)
  } finally {
    loading.value = false
  }
}

const fetchSessionItems = async (sessionId: number) => {

  if (props.flowRuntimeId === 0) return

  try {
    const newItems = await flowStore.getFlowRuntimeSessionItems(props.flowRuntimeId, sessionId)
    if(lastSessionId.value != sessionId) {
      sessionItems.value = newItems
      lastSessionId.value = sessionId;
    }
    else if (newItems.length !== lastSessionItemCount) {
      lastSessionItemCount = newItems.length
      let vals = sessionItems.value || []
      for(let i = vals.length;i < newItems.length; i++) {
        sessionItems.value.push(newItems[i]);
      }

      lastSessionId.value = sessionId;
      // scrollToBottom()
    }
  } catch (error) {
    console.error('获取会话消息失败:', error)
  }
}

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

const getAvatarClass = (type: string) => {
  if (type === 'User') return 'bg-primary-500 text-white'
  if (type === 'assistant') return 'bg-surface-100 text-surface-600'
  return 'bg-surface-100 text-surface-600'
}

const getMessageIcon = (type: string) => {
  if (type === 'User') return User
  return Bot
}

const getMessageClass = (type: string) => {
  if (type === 'User') return 'bg-primary-500 text-white'
  return 'bg-surface-50 text-surface-700'
}

const formatTime = (dateStr: string) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
}

watch(() => props.flowRuntimeId, async (newVal) => {
  if (newVal > 0) {
    await fetchSessions()
    if (pollingInterval.value) {
      clearInterval(pollingInterval.value)
    }
    pollingInterval.value = window.setInterval(fetchSessions, 5000)
  } else {
    if (pollingInterval.value) {
      clearInterval(pollingInterval.value)
      pollingInterval.value = null
    }
  }
})

onMounted(async () => {
  await fetchSessions()
  if (props.flowRuntimeId > 0) {
    pollingInterval.value = window.setInterval(fetchSessions, 5000)
  }
})

onUnmounted(() => {
  if (pollingInterval.value) {
    clearInterval(pollingInterval.value)
  }
})
</script>


<template>
  <div class="flex flex-col h-full">
    <div class="flex items-center gap-4 p-4 bg-white border-b border-surface-200">
      <div class="flex-1">
        <p class="text-sm font-medium text-surface-900">
          会话 
          <!-- <span v-if="props.flowRuntimeId > 0" class="text-primary-500">#{{ props.flowRuntimeId }}</span>
          <span v-else class="text-surface-400">（未选择）</span> -->
        </p>
        <p class="text-xs text-surface-400 mt-0.5">{{ sessionItems.length }} 条消息</p>
      </div>
    </div>

    <!-- <div v-if="sessions.length > 0" class="flex gap-2 p-2 bg-surface-50 border-b border-surface-200 overflow-x-auto">
      <button
        v-for="session in sessions"
        :key="session.id"
        @click="currentSession = session; fetchSessionItems(session.id)"
        :class="[
          'px-3 py-1.5 text-sm rounded-lg whitespace-nowrap transition-colors',
          currentSession?.id === session.id
            ? 'bg-primary-500 text-white'
            : 'bg-white text-surface-600 hover:bg-surface-100'
        ]"
      >
        会话 {{ session.id }}
      </button>
    </div> -->

    <div ref="messagesContainer" class="flex-1 overflow-y-auto p-4 space-y-4 bg-white">
      <!-- <div v-if="loading" class="flex items-center justify-center h-full">
        <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
      </div> -->

      <template v-if="sessionItems.length > 0">
        <div
          v-for="message in sessionItems"
          :key="message.id"
          :class="['flex gap-3', message.session_type === 'User' || message.session_type === 'user' ? 'flex-row-reverse' : '']"
        >
          <div :class="['w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0', getAvatarClass(message.session_type)]">
            <component :is="getMessageIcon(message.session_type)" class="w-4 h-4" />
          </div>
          <div :class="['flex-1', (message.session_type === 'User' || message.session_type === 'user') ? 'max-w-xl text-right' : 'max-w-3xl']">
            <div :class="['rounded-xl p-4 border inline-block', getMessageClass(message.session_type)]">
              <p :class="['whitespace-pre-wrap text-black  text-sm leading-relaxed', (message.session_type === 'User' || message.session_type === 'user') ? 'text-white' : 'text-surface-700']"
              v-html=" parseMarkdown(message.content)"
              >
              </p>
            </div>
            <div :class="['flex items-center gap-2 mt-1 text-xs text-surface-400', (message.session_type === 'User' || message.session_type === 'user') ? 'justify-end' : '']">
              <span>{{ formatTime(message.created_at) }}</span>
            </div>
          </div>
        </div>
      </template>

      <div v-else class="flex flex-col items-center justify-center h-full text-center text-surface-400">
        <Bot class="w-12 h-12 mb-4 opacity-50" />
        <p>暂无会话消息</p>
        <p class="text-sm mt-2">在控制面板右侧选择运行时后将显示会话内容</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
</style>