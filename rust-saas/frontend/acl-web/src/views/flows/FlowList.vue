<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, Workflow, Trash2, Clock, Loader2, Play } from 'lucide-vue-next'
import { useFlowStore } from '@/stores'
import { authService } from '@/api'

const router = useRouter()
const flowStore = useFlowStore()
const deletingId = ref<number | null>(null)

onMounted(async () => { await flowStore.fetchFlows() })

const startFlow = (id: number, event: Event) => {
  event.stopPropagation()
  router.push(`/flows/${id}/run`)
}

const deleteFlow = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个工作流吗?')) {
    deletingId.value = id
    try { await flowStore.deleteFlow(id) }
    finally { deletingId.value = null }
  }
}

const formatDate = (dateStr: string) => new Date(dateStr).toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">工作流</h1>
        <p class="page-subtitle">flow orchestration</p>
      </div>
      <button @click="router.push('/flows/new')" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 新建工作流
      </button>
    </div>

    <div v-if="flowStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="flowStore.flows.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <Workflow class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无工作流</h3>
      <p class="text-surface-400 mb-6">创建一个工作流来编排 Agent 任务</p>
      <button @click="router.push('/flows/new')" class="btn btn-primary"><Plus class="w-4 h-4" /> 创建工作流</button>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <div v-for="flow in flowStore.flows" :key="flow.id" class="card p-5 cursor-pointer group" @click="router.push(`/flows/${flow.id}/edit`)">
        <div class="flex items-start justify-between mb-4">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-primary-100 to-accent-100 flex items-center justify-center group-hover:shadow-card transition-shadow">
              <Workflow class="w-6 h-6 text-primary-500" />
            </div>
            <div>
              <h3 class="font-semibold text-base text-surface-800">{{ flow.name }}</h3>
              <div class="flex items-center gap-1 text-xs text-surface-400 mt-1 font-mono">
                <Clock class="w-3 h-3" /> {{ formatDate(flow.created_at) }}
              </div>
            </div>
          </div>
          <div class="flex items-center gap-1">
            <button @click.stop="router.push(`/flows/${flow.id}/edit`)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors" title="编辑">
              <Workflow class="w-4 h-4" />
            </button>
            <button @click.stop="startFlow(flow.id, $event)" class="p-2 text-surface-400 hover:text-green-500 hover:bg-green-50 rounded-lg transition-colors" title="运行">
              <Play class="w-4 h-4" />
            </button>
            <button @click.stop="deleteFlow(flow.id, $event)" :disabled="deletingId === flow.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors" :title="deletingId === flow.id ? '删除中...' : '删除'">
              <Loader2 v-if="deletingId === flow.id" class="w-4 h-4 animate-spin" />
              <Trash2 v-else class="w-4 h-4" />
            </button>
          </div>
        </div>

        <div class="flex items-center gap-2 mb-4">
          <span class="tag tag-blue">{{ flow.config.vertices?.length || 0 }} 节点</span>
          <span class="tag tag-cyan">{{ flow.config.edges?.length || 0 }} 连接</span>
        </div>
      </div>
    </div>
  </div>
</template>