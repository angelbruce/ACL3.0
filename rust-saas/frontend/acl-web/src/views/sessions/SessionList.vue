<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, MessageSquare, Trash2, Clock, Loader2 } from 'lucide-vue-next'
import { useSessionStore, useAuthStore } from '@/stores'
import { authService } from '@/api'

const router = useRouter()
const sessionStore = useSessionStore()
const authStore = useAuthStore()

const showNewDialog = ref(false)
const newDescription = ref('')
const loading = ref(false)

onMounted(async () => {
  if (!authStore.user && authStore.isAuthenticated) {
    try {
      const tokenData = JSON.parse(atob(authStore.accessToken!.split('.')[1]))
      const userInfo = await authService.getUser(tokenData.user_id)
      authStore.user = userInfo
    } catch {
      // 获取用户信息失败
    }
  }
  await sessionStore.fetchSessions()
})

const createSession = async () => {
  if (!authStore.user) return
  loading.value = true
  try {
    const session = await sessionStore.createSession({
      user_id: authStore.user.id,
      description: newDescription.value || undefined,
    })
    showNewDialog.value = false
    newDescription.value = ''
    router.push(`/sessions/${session.id}`)
  } catch {
  } finally {
    loading.value = false
  }
}

const deleteSession = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个会话吗?')) {
    await sessionStore.deleteSession(id)
  }
}

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">会话</h1>
        <p class="page-subtitle">chat sessions</p>
      </div>
      <button @click="showNewDialog = true" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        新建会话
      </button>
    </div>

    <div v-if="sessionStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="sessionStore.sessions.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <MessageSquare class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无会话</h3>
      <p class="text-surface-400 mb-6">开始一个新会话来与 Agent 对话</p>
      <button @click="showNewDialog = true" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 创建会话
      </button>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <div
        v-for="session in sessionStore.sessions"
        :key="session.id"
        @click="router.push(`/sessions/${session.id}`)"
        class="card p-4 cursor-pointer group"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-primary-50 flex items-center justify-center">
              <MessageSquare class="w-5 h-5 text-primary-500" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="font-medium text-surface-800 truncate">
                {{ session.description || `会话 #${session.id}` }}
              </h3>
              <div class="flex items-center gap-1 text-xs text-surface-400 mt-1">
                <Clock class="w-3 h-3" />
                {{ formatDate(session.created_at) }}
              </div>
            </div>
          </div>
          <button
            @click="deleteSession(session.id, $event)"
            class="p-2 text-surface-300 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
        <div class="flex flex-wrap gap-2 mt-2 pt-2 border-t border-surface-100">
          <span v-if="session.agent_name" class="tag tag-cyan text-xs">{{ session.agent_name }}</span>
          <span v-if="session.model_name" class="tag tag-blue text-xs">{{ session.model_name }}</span>
        </div>
      </div>
    </div>

    <!-- New session dialog -->
    <Teleport to="body">
      <div
        v-if="showNewDialog"
        class="fixed inset-0 bg-black/30 flex items-center justify-center z-50 p-4"
        @click.self="showNewDialog = false"
      >
        <div class="card p-6 w-full max-w-md animate-fade-in">
          <h2 class="text-lg font-semibold mb-4 text-surface-800">新建会话</h2>
          <form @submit.prevent="createSession">
            <div class="mb-4">
              <label class="block text-sm font-medium text-surface-700 mb-2">描述 (可选)</label>
              <input
                v-model="newDescription"
                type="text"
                placeholder="输入会话描述..."
                class="input-base w-full"
              />
            </div>
            <div class="flex gap-3">
              <button type="button" @click="showNewDialog = false" class="btn btn-outline flex-1 justify-center">取消</button>
              <button type="submit" :disabled="loading" class="btn btn-primary flex-1 justify-center">
                {{ loading ? '创建中...' : '创建' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>
  </div>
</template>