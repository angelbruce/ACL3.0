<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Plus, Server, Trash2, RefreshCw, Eye, EyeOff, Edit3, Power, Settings, Wrench } from 'lucide-vue-next'
import { useMcpStore } from '@/stores'
import type { McpServer, CreateMcpServerRequest, McpServerWithTools } from '@/types'

const mcpStore = useMcpStore()

const showForm = ref(false)
const editingServer = ref<McpServer | null>(null)
const showHeaders = ref<number | null>(null)
const showTools = ref<number | null>(null)
const serverTools = ref<Map<number, McpServerWithTools>>(new Map())
const loadingTools = ref<Set<number>>(new Set())

const formData = ref<Partial<CreateMcpServerRequest>>({
  name: '', description: '', server_type: 'sse', url: '', headers: {}, enabled: true, stateless: false,
})

onMounted(async () => { await Promise.all([mcpStore.fetchServers(), mcpStore.fetchTools()]) })

const openForm = (server?: McpServer) => {
  if (server) {
    editingServer.value = server
    formData.value = { name: server.name, description: server.description || '', server_type: server.server_type, url: server.url, headers: server.headers || {}, enabled: server.enabled, stateless: server.stateless || false }
  } else {
    editingServer.value = null
    formData.value = { name: '', description: '', server_type: 'sse', url: '', headers: {}, enabled: true, stateless: false }
  }
  showForm.value = true
}

const closeForm = () => { showForm.value = false; editingServer.value = null }

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.url || !formData.value.server_type) { alert('请填写所有必填字段'); return }
  try {
    if (editingServer.value) await mcpStore.updateServer(editingServer.value.id, formData.value as CreateMcpServerRequest)
    else await mcpStore.createServer(formData.value as CreateMcpServerRequest)
    closeForm()
  } catch {}
}

const deleteServer = async (id: number) => { if (confirm('确定要删除这个 MCP 服务器吗?')) { await mcpStore.deleteServer(id); serverTools.value.delete(id) } }
const toggleServer = async (server: McpServer) => { await mcpStore.toggleServer(server.id, !server.enabled); serverTools.value.delete(server.id) }
const toggleHeaders = (id: number) => { showHeaders.value = showHeaders.value === id ? null : id }
const toggleTools = async (id: number) => {
  if (showTools.value === id) { showTools.value = null; return }
  showTools.value = id
  if (!serverTools.value.has(id)) {
    loadingTools.value.add(id)
    try { const data = await mcpStore.getServerTools(id); serverTools.value.set(id, data) } catch (e) { console.error(e) }
    finally { loadingTools.value.delete(id) }
  }
}

const refreshAll = async () => { await mcpStore.refreshServers(); serverTools.value.clear() }
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">MCP 服务器管理</h1>
        <p class="page-subtitle">mcp server management</p>
      </div>
      <div class="flex items-center gap-2">
        <button @click="refreshAll" class="btn btn-outline"><RefreshCw class="w-4 h-4" :class="{ 'animate-spin': mcpStore.loading }" /> 刷新</button>
        <button @click="openForm()" class="btn btn-primary"><Plus class="w-4 h-4" /> 添加服务器</button>
      </div>
    </div>

    <div class="card p-4 mb-6 flex items-center gap-4">
      <div class="w-12 h-12 rounded-xl bg-cyan-50 flex items-center justify-center">
        <Settings class="w-6 h-6 text-cyan-500" />
      </div>
      <div>
        <h3 class="font-medium text-surface-800">可用工具</h3>
        <p class="text-sm text-surface-400">{{ mcpStore.tools.length }} 个工具 ({{ mcpStore.servers.filter(s => s.enabled).length }} 个服务器)</p>
      </div>
    </div>

    <div v-if="mcpStore.loading" class="flex items-center justify-center py-12">
      <RefreshCw class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="mcpStore.servers.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-cyan-50 border border-cyan-100 flex items-center justify-center mb-4">
        <Server class="w-8 h-8 text-cyan-500" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无 MCP 服务器</h3>
      <p class="text-surface-400 mb-6">添加外部 MCP 服务器来扩展功能</p>
      <button @click="openForm()" class="btn btn-primary"><Plus class="w-4 h-4" /> 添加服务器</button>
    </div>

    <div v-else class="space-y-4">
      <div v-for="server in mcpStore.servers" :key="server.id" class="card p-5">
        <div class="flex items-start justify-between mb-4">
          <div class="flex items-center gap-4">
            <div class="w-12 h-12 rounded-xl bg-cyan-50 flex items-center justify-center">
              <Server class="w-6 h-6 text-cyan-500" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3 class="font-semibold text-lg text-surface-800">{{ server.name }}</h3>
                <span v-if="server.enabled" class="tag tag-green">已启用</span>
                <span v-else class="tag bg-surface-100 text-surface-400 border-surface-200">已禁用</span>
              </div>
              <p v-if="server.description" class="text-sm text-surface-500 mt-1">{{ server.description }}</p>
              <p class="text-sm text-surface-400 mt-1 truncate max-w-md">{{ server.url }}</p>
            </div>
          </div>
        </div>

        <div v-if="server.headers && Object.keys(server.headers).length > 0" class="mb-4 p-3 bg-surface-50 rounded-lg">
          <div class="flex items-center justify-between">
            <span class="text-sm text-surface-500">Headers:</span>
            <button @click="toggleHeaders(server.id)" class="p-1.5 text-surface-400 hover:text-surface-700 rounded-lg hover:bg-surface-100 transition-colors">
              <EyeOff v-if="showHeaders === server.id" class="w-4 h-4" />
              <Eye v-else class="w-4 h-4" />
            </button>
          </div>
          <div v-if="showHeaders === server.id" class="mt-2 space-y-1">
            <div v-for="(value, key) in server.headers" :key="key" class="text-xs text-surface-500 flex items-center gap-2">
              <span class="text-cyan-600 font-medium">{{ key }}:</span>
              <span class="font-mono">{{ value }}</span>
            </div>
          </div>
        </div>

        <div class="mb-4">
          <div class="flex items-center justify-between">
            <span class="text-sm text-surface-500 flex items-center gap-1.5"><Wrench class="w-4 h-4" /> 工具列表</span>
            <button @click="toggleTools(server.id)" :disabled="!server.enabled" class="p-1.5 text-surface-400 hover:text-surface-700 rounded-lg hover:bg-surface-100 transition-colors disabled:opacity-50">
              <EyeOff v-if="showTools === server.id" class="w-4 h-4" />
              <Eye v-else class="w-4 h-4" />
            </button>
          </div>
          <div v-if="showTools === server.id" class="mt-3">
            <div v-if="loadingTools.has(server.id)" class="flex items-center justify-center py-4"><RefreshCw class="w-5 h-5 animate-spin text-primary-500" /></div>
            <div v-else-if="!server.enabled" class="text-sm text-surface-400 py-4 text-center">请先启用服务器以查看工具</div>
            <div v-else-if="serverTools.get(server.id)?.tools.length === 0" class="text-sm text-surface-400 py-4 text-center">该服务器暂无可用工具</div>
            <div v-else class="space-y-2">
              <div v-for="tool in serverTools.get(server.id)?.tools" :key="tool.name" class="p-3 bg-surface-50 rounded-lg">
                <div class="flex items-center justify-between mb-1"><span class="font-medium text-sm text-surface-800">{{ tool.name }}</span></div>
                <p class="text-xs text-surface-500">{{ tool.description }}</p>
                <div class="mt-2 pt-2 border-t border-surface-200">
                  <div class="text-xs text-surface-500"><span class="text-cyan-600">输入参数:</span><pre class="mt-1 text-surface-500">{{ JSON.stringify(tool.inputSchema, null, 2) }}</pre></div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button @click="openForm(server)" class="btn btn-outline flex-1 justify-center text-sm"><Edit3 class="w-3.5 h-3.5" /> 编辑</button>
          <button @click="toggleServer(server)" class="btn btn-outline text-sm justify-center" :class="{ 'text-green-600 border-green-200 hover:bg-green-50': server.enabled }"><Power class="w-3.5 h-3.5" /></button>
          <button @click="deleteServer(server.id)" class="btn btn-danger text-sm justify-center"><Trash2 class="w-3.5 h-3.5" /></button>
        </div>
      </div>
    </div>

    <!-- Form dialog -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 bg-black/30 flex items-center justify-center z-50 p-4" @click.self="closeForm">
        <div class="card p-6 w-full max-w-lg animate-fade-in max-h-[90vh] overflow-y-auto">
          <h2 class="text-lg font-semibold mb-4 text-surface-800">{{ editingServer ? '编辑服务器' : '添加服务器' }}</h2>
          <form @submit.prevent="handleSubmit" class="space-y-4">
            <div><label class="block text-sm font-medium text-surface-700 mb-2">名称 <span class="text-red-500">*</span></label><input v-model="formData.name" type="text" placeholder="例如: GitHub MCP" class="input-base w-full" /></div>
            <div><label class="block text-sm font-medium text-surface-700 mb-2">描述</label><textarea v-model="formData.description" placeholder="描述这个服务器的功能" rows="2" class="input-base w-full resize-none" /></div>
            <div><label class="block text-sm font-medium text-surface-700 mb-2">类型 <span class="text-red-500">*</span></label><select v-model="formData.server_type" class="input-base w-full"><option value="sse">SSE 服务器</option></select></div>
            <div><label class="block text-sm font-medium text-surface-700 mb-2">URL <span class="text-red-500">*</span></label><input v-model="formData.url" type="url" placeholder="http://localhost:8000" class="input-base w-full" /></div>
            <div><label class="block text-sm font-medium text-surface-700 mb-2">Headers (JSON)</label><textarea :value="JSON.stringify(formData.headers, null, 2)" @input="(e: Event) => { try { formData.headers = JSON.parse((e.target as HTMLTextAreaElement).value) } catch {} }" placeholder='{"Authorization": "Bearer token"}' rows="4" class="input-base w-full font-mono text-sm" /></div>
            <div class="flex items-center gap-2"><input v-model="formData.enabled" type="checkbox" id="enabled" class="w-4 h-4 rounded border-surface-300 text-primary-500 focus:ring-primary-500" /><label for="enabled" class="text-sm text-surface-700">启用服务器</label></div>
            <div class="flex items-center gap-2"><input v-model="formData.stateless" type="checkbox" id="stateless" class="w-4 h-4 rounded border-surface-300 text-primary-500 focus:ring-primary-500" /><label for="stateless" class="text-sm text-surface-700">Stateless 模式</label><span class="text-xs text-surface-400">（使用 REST API）</span></div>
            <div class="flex gap-3 pt-2"><button type="button" @click="closeForm" class="btn btn-outline flex-1 justify-center">取消</button><button type="submit" class="btn btn-primary flex-1 justify-center">{{ editingServer ? '保存' : '添加' }}</button></div>
          </form>
        </div>
      </div>
    </Teleport>
  </div>
</template>