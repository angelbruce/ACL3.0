<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Save, Loader2, Plus, Trash2, Check, X, RefreshCw, Settings, Wrench, Sparkles } from 'lucide-vue-next'
import { useAgentStore, useMcpStore } from '@/stores'
import type { CreateAgentRequest, AgentToolCreate, AgentSkillCreate } from '@/types'
import type { MCPTool, McpServer } from '@/types'

const route = useRoute()
const router = useRouter()
const agentStore = useAgentStore()
const mcpStore = useMcpStore()

const isEdit = computed(() => !!route.params.id)
const loading = ref(false)
const saving = ref(false)
const loadingTools = ref(false)

const formData = ref<CreateAgentRequest>({ name: '', defination: '', tools: [], skills: [] })

const availableTools = ref<MCPTool[]>([])
const selectedToolNames = ref<Set<string>>(new Set())
const mcpServers = ref<McpServer[]>([])

const newSkill = ref<AgentSkillCreate>({ skill_prompt: '' })
const showSkillForm = ref(false)
const editingIndex = ref<number | null>(null)
const editingSkill = ref<AgentSkillCreate>({ skill_prompt: '' })

const activeTab = ref<'basic' | 'tools' | 'skills'>('basic')

const tabs = [
  { id: 'basic', label: '基本信息', icon: Settings },
  { id: 'tools', label: '工具', icon: Wrench },
  { id: 'skills', label: '技能', icon: Sparkles },
] as const

const getServerName = (serverId: number | null | undefined): string => {
  if (serverId === null || serverId === undefined) return '内置'
  const server = mcpServers.value.find(s => s.id === serverId)
  return server?.name || '未知服务器'
}

onMounted(async () => {
  await Promise.all([loadTools(), loadServers()])
  if (isEdit.value) {
    loading.value = true
    try {
      const agent = await agentStore.fetchAgent(Number(route.params.id))
      if (agent) {
        formData.value = {
          name: agent.name,
          defination: agent.defination || '',
          tools: agent.tools.map((t) => ({ name: t.name, description: t.description, input_schema: t.input_schema, output_schema: t.output_schema, server_id: t.server_id ?? undefined })),
          skills: agent.skills.map((s) => ({ skill_prompt: s.skill_prompt })),
        }
        agent.tools.forEach(t => {
          const serverName = t.server_id ? getServerName(t.server_id) : '内置'
          selectedToolNames.value.add(`${serverName}-${t.name}`)
        })
      }
    } finally { loading.value = false }
  }
})

const loadTools = async () => { loadingTools.value = true; try { await mcpStore.fetchTools(); availableTools.value = mcpStore.tools } finally { loadingTools.value = false } }
const loadServers = async () => { try { await mcpStore.fetchServers(); mcpServers.value = mcpStore.servers } catch {} }

const toggleTool = (tool: MCPTool) => {
  const serverName = getServerName(tool.serverId)
  const toolKey = `${serverName}-${tool.name}`
  if (selectedToolNames.value.has(toolKey)) {
    selectedToolNames.value.delete(toolKey)
    const index = formData.value.tools?.findIndex(t => { const tServerName = getServerName(t.server_id ?? undefined); return `${tServerName}-${t.name}` === toolKey })
    if (index !== -1 && index >= 0) formData.value.tools?.splice(index, 1)
  } else {
    selectedToolNames.value.add(toolKey)
    formData.value.tools?.push({ name: tool.name, description: tool.description, input_schema: JSON.stringify(tool.inputSchema || {}), output_schema: JSON.stringify(tool.outputSchema || {}), server_id: tool.serverId })
  }
}

const addSkill = () => { if (!newSkill.value.skill_prompt) return; formData.value.skills?.push({ ...newSkill.value }); newSkill.value = { skill_prompt: '' }; showSkillForm.value = false }
const removeSkill = (index: number) => { formData.value.skills?.splice(index, 1); if (editingIndex.value === index) { editingIndex.value = null; editingSkill.value = { skill_prompt: '' } } }
const editSkill = (index: number) => { editingIndex.value = index; editingSkill.value = { ...formData.value.skills![index] } }
const saveSkill = () => { if (!editingSkill.value.skill_prompt) return; formData.value.skills![editingIndex.value!] = { ...editingSkill.value }; editingIndex.value = null; editingSkill.value = { skill_prompt: '' } }
const cancelEdit = () => { editingIndex.value = null; editingSkill.value = { skill_prompt: '' } }

const handleSubmit = async () => {
  if (!formData.value.name) { alert('请输入 Agent 名称'); return }
  saving.value = true
  try {
    if (isEdit.value) await agentStore.updateAgent(Number(route.params.id), formData.value)
    else await agentStore.createAgent(formData.value)
    router.push('/agents')
  } catch {} finally { saving.value = false }
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <div class="flex items-center gap-4 p-4 bg-white border-b border-surface-200">
      <button @click="router.push('/agents')" class="p-2 rounded-lg text-surface-400 hover:text-surface-700 hover:bg-surface-50 transition-colors">
        <ArrowLeft class="w-5 h-5" />
      </button>
      <h1 class="text-xl font-bold text-surface-900">{{ isEdit ? '编辑 Agent' : '新建 Agent' }}</h1>
    </div>

    <!-- Toolbar -->
    <div class="flex items-center justify-between px-6 py-3 bg-white border-b border-surface-200 shadow-sm">
      <div class="flex items-center gap-1 bg-surface-50 rounded-lg p-0.5">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          :class="[
            'flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all',
            activeTab === tab.id
              ? 'bg-primary-50 text-primary-600 shadow-sm'
              : 'text-surface-500 hover:text-surface-700 hover:bg-surface-100'
          ]"
        >
          <component :is="tab.icon" class="w-4 h-4" />
          {{ tab.label }}
        </button>
      </div>
      <div class="flex items-center gap-2">
        <button type="button" @click="router.push('/agents')" class="btn btn-outline">
          取消
        </button>
        <button type="submit" :disabled="saving" class="btn btn-primary" form="agent-form">
          <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />
          <Save v-else class="w-4 h-4" />
          {{ saving ? '保存中...' : '保存' }}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto bg-white p-6">
      <form id="agent-form" @submit.prevent="handleSubmit" class="max-w-3xl mx-auto">
        <div v-if="loading" class="flex items-center justify-center py-12">
          <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
        </div>

        <template v-else>
          <!-- Basic Info Tab -->
          <div v-show="activeTab === 'basic'" class="card p-6">
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-surface-700 mb-2">名称 <span class="text-red-500">*</span></label>
                <input v-model="formData.name" type="text" placeholder="输入 Agent 名称" class="input-base w-full" />
              </div>
              <div>
                <label class="block text-sm font-medium text-surface-700 mb-2">定义</label>
                <textarea v-model="formData.defination" rows="20" placeholder="描述 Agent 的角色和能力..." class="input-base w-full resize-none" />
              </div>
            </div>
          </div>

          <!-- Tools Tab -->
          <div v-show="activeTab === 'tools'" class="card p-6">
            <div class="flex items-center justify-between mb-4">
              <h3 class="font-semibold text-surface-800">工具列表</h3>
              <button type="button" @click="loadTools" class="btn btn-ghost text-sm">
                <Loader2 v-if="loadingTools" class="w-4 h-4 animate-spin" />
                <RefreshCw v-else class="w-4 h-4" />
                刷新列表
              </button>
            </div>

            <div v-if="formData.tools?.length" class="mb-4 p-3 bg-primary-50 rounded-lg border border-primary-200">
              <div class="flex items-center justify-between mb-2">
                <p class="text-sm font-medium text-primary-700">已选择工具</p>
                <span class="text-xs text-primary-500">{{ formData.tools.length }} 个</span>
              </div>
              <div class="flex flex-wrap gap-2">
                <span v-for="tool in formData.tools" :key="`${getServerName(tool.server_id ?? undefined)}-${tool.name}`" class="tag tag-blue" :title="getServerName(tool.server_id ?? undefined)">
                  {{ getServerName(tool.server_id ?? undefined) }} / {{ tool.name }}
                </span>
              </div>
            </div>

            <div v-if="availableTools.length" class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div
                v-for="tool in availableTools" :key="`${getServerName(tool.serverId)}-${tool.name}`" @click="toggleTool(tool)"
                :class="['p-3 rounded-lg cursor-pointer transition-all border-2', selectedToolNames.has(`${getServerName(tool.serverId)}-${tool.name}`) ? 'bg-primary-50 border-primary-500' : 'bg-surface-50 border-transparent hover:border-surface-300']"
              >
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <div class="flex items-center gap-2 flex-wrap">
                      <span class="font-medium text-sm text-surface-800">{{ tool.name }}</span>
                      <span class="tag bg-surface-200 text-surface-500">{{ getServerName(tool.serverId) }}</span>
                      <span v-if="selectedToolNames.has(`${getServerName(tool.serverId)}-${tool.name}`)" class="p-0.5 bg-primary-500 rounded-full">
                        <Check class="w-3 h-3 text-white" />
                      </span>
                    </div>
                    <p class="text-xs text-surface-400 mt-1 line-clamp-2">{{ tool.description }}</p>
                  </div>
                </div>
              </div>
            </div>
            <div v-else-if="loadingTools" class="flex items-center justify-center py-4">
              <Loader2 class="w-5 h-5 animate-spin text-primary-500" />
            </div>
            <p v-else class="text-sm text-surface-400 text-center py-4">暂无可用工具，请先在 MCP 服务器中添加工具</p>
          </div>

          <!-- Skills Tab -->
          <div v-show="activeTab === 'skills'" class="card p-6">
            <div class="flex items-center justify-between mb-4">
              <h3 class="font-semibold text-surface-800">技能列表</h3>
              <button type="button" @click="showSkillForm = !showSkillForm" class="btn btn-primary text-sm">
                <Plus class="w-4 h-4" />
                添加技能
              </button>
            </div>

            <div v-if="showSkillForm" class="mb-4 p-4 bg-surface-50 rounded-lg space-y-3">
              <textarea v-model="newSkill.skill_prompt" rows="3" placeholder="技能提示词..." class="input-base w-full resize-none" />
              <div class="flex gap-2">
                <button type="button" @click="addSkill" class="btn btn-primary">确定</button>
                <button type="button" @click="showSkillForm = false" class="btn btn-outline">取消</button>
              </div>
            </div>

            <div v-if="formData.skills?.length" class="space-y-3">
              <div v-for="(skill, index) in formData.skills" :key="index" class="rounded-lg border transition-all">
                <div v-if="editingIndex === index" class="p-3 bg-primary-50 border-primary-200">
                  <textarea v-model="editingSkill.skill_prompt" rows="3" class="input-base w-full resize-none" />
                  <div class="flex gap-2 mt-3">
                    <button type="button" @click="saveSkill" class="btn btn-primary text-sm">保存</button>
                    <button type="button" @click="cancelEdit" class="btn btn-outline text-sm">取消</button>
                  </div>
                </div>
                <div v-else class="flex items-start justify-between p-3 bg-surface-50 border-surface-200">
                  <p class="text-sm text-surface-700 flex-1 line-clamp-3">{{ skill.skill_prompt }}</p>
                  <div class="flex items-center gap-1 ml-2">
                    <button type="button" @click="editSkill(index)" class="btn btn-ghost !p-1.5" title="编辑">
                      <Settings class="w-4 h-4 text-surface-500" />
                    </button>
                    <button type="button" @click="removeSkill(index)" class="btn btn-danger !p-1.5" title="删除">
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
            <p v-else class="text-sm text-surface-400 text-center py-8">暂无技能</p>
          </div>
        </template>
      </form>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-2 { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.line-clamp-3 { display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; }
</style>