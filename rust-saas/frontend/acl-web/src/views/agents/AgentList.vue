<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, Bot, Trash2, Edit2, Loader2 } from 'lucide-vue-next'
import { useAgentStore } from '@/stores'

const router = useRouter()
const agentStore = useAgentStore()

const deletingId = ref<number | null>(null)

onMounted(async () => {
  await agentStore.fetchAgents()
})

const deleteAgent = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个 Agent 吗?')) {
    deletingId.value = id
    try {
      await agentStore.deleteAgent(id)
    } finally {
      deletingId.value = null
    }
  }
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">Agent</h1>
        <p class="page-subtitle">agent configuration</p>
      </div>
      <button @click="router.push('/agents/new')" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        新建 Agent
      </button>
    </div>

    <div v-if="agentStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="agentStore.agents.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <Bot class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无 Agent</h3>
      <p class="text-surface-400 mb-6">创建一个 Agent 来开始自动化任务</p>
      <button @click="router.push('/agents/new')" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 创建 Agent
      </button>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <div
        v-for="agent in agentStore.agents"
        :key="agent.id"
        @click="router.push(`/agents/${agent.id}/edit`)"
        class="card p-5 cursor-pointer group"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-primary-100 to-accent-100 flex items-center justify-center">
              <Bot class="w-6 h-6 text-primary-500" />
            </div>
            <div>
              <p class="text-xs text-surface-400 mb-0.5">名称</p>
              <h3 class="font-semibold text-base text-surface-800 truncate" :title="agent.name">{{ agent.name }}</h3>
              <p class="text-xs text-surface-400 mt-0.5">{{ formatDate(agent.created_at) }}</p>
            </div>
          </div>
          <div class="flex items-center gap-1">
            <button @click.stop="router.push(`/agents/${agent.id}/edit`)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
              <Edit2 class="w-4 h-4" />
            </button>
            <button @click.stop="deleteAgent(agent.id, $event)" :disabled="deletingId === agent.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
              <Loader2 v-if="deletingId === agent.id" class="w-4 h-4 animate-spin" />
              <Trash2 v-else class="w-4 h-4" />
            </button>
          </div>
        </div>

        <div class="min-h-[80px]">
          <p class="text-xs text-surface-400 mb-1">定义</p>
          <p v-if="agent.defination" class="text-sm text-surface-600 line-clamp-3"><el-tooltip placement="top"  :content="agent.defination">
            {{ agent.defination }}</el-tooltip></p>
          <p v-else class="text-sm text-surface-300 italic">暂无定义</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>