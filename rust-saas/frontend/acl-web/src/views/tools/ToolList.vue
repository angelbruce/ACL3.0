<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Wrench, Loader2, Play, ChevronDown, ChevronRight, Terminal } from 'lucide-vue-next'
import { useMcpStore } from '@/stores'
import type { MCPToolCallResult } from '@/types'

const mcpStore = useMcpStore()

const expandedTool = ref<string | null>(null)
const callResult = ref<Record<string, MCPToolCallResult>>({})
const callLoading = ref<string | null>(null)
const callArgs = ref<Record<string, string>>({})

onMounted(async () => { await mcpStore.fetchTools() })

const toggleTool = (name: string) => { expandedTool.value = expandedTool.value === name ? null : name }

const callTool = async (name: string) => {
  callLoading.value = name
  try {
    const args = callArgs.value[name] ? JSON.parse(callArgs.value[name]) : {}
    const result = await mcpStore.callTool(name, args)
    callResult.value[name] = result
  } catch (error) {
    callResult.value[name] = { success: false, content: '', error: error instanceof Error ? error.message : '未知错误' }
  } finally { callLoading.value = null }
}

const formatSchema = (schema: Record<string, unknown>) => JSON.stringify(schema, null, 2)
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">MCP 工具</h1>
        <p class="page-subtitle">mcp tools</p>
      </div>
    </div>

    <div v-if="mcpStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="mcpStore.tools.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-green-50 border border-green-100 flex items-center justify-center mb-4">
        <Wrench class="w-8 h-8 text-green-500" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无工具</h3>
      <p class="text-surface-400">配置 MCP 服务器以启用工具</p>
    </div>

    <div v-else class="space-y-4">
      <div v-for="tool in mcpStore.tools" :key="tool.name" class="card overflow-hidden">
        <div @click="toggleTool(tool.name)" class="flex items-center gap-4 p-4 cursor-pointer hover:bg-surface-50 transition-colors">
          <div class="w-10 h-10 rounded-lg bg-green-50 flex items-center justify-center">
            <Wrench class="w-5 h-5 text-green-500" />
          </div>
          <div class="flex-1">
            <h3 class="font-semibold text-surface-800">{{ tool.name }}</h3>
            <p class="text-sm text-surface-400 mt-1">{{ tool.description }}</p>
          </div>
          <component :is="expandedTool === tool.name ? ChevronDown : ChevronRight" class="w-5 h-5 text-surface-400" />
        </div>

        <div v-if="expandedTool === tool.name" class="p-4 pt-0 space-y-4">
          <div>
            <h4 class="text-sm font-medium text-surface-500 mb-2">输入参数</h4>
            <pre class="p-3 bg-surface-50 rounded-lg text-xs font-mono overflow-x-auto text-surface-600">{{ formatSchema(tool.inputSchema) }}</pre>
          </div>

          <div>
            <h4 class="text-sm font-medium text-surface-500 mb-2">输出格式</h4>
            <pre class="p-3 bg-surface-50 rounded-lg text-xs font-mono overflow-x-auto text-surface-600">{{ formatSchema(tool.outputSchema) }}</pre>
          </div>

          <div class="pt-4 border-t border-surface-200">
            <h4 class="text-sm font-medium text-surface-500 mb-2">测试调用</h4>
            <div class="flex gap-2">
              <input v-model="callArgs[tool.name]" type="text" placeholder='{"param": "value"}' class="flex-1 input-base font-mono text-sm" />
              <button @click="callTool(tool.name)" :disabled="callLoading === tool.name" class="btn btn-primary">
                <Loader2 v-if="callLoading === tool.name" class="w-4 h-4 animate-spin" />
                <Play v-else class="w-4 h-4" /> 执行
              </button>
            </div>
          </div>

          <div v-if="callResult[tool.name]" class="pt-4 border-t border-surface-200">
            <h4 class="text-sm font-medium text-surface-500 mb-2">执行结果</h4>
            <div :class="['p-3 rounded-lg text-sm font-mono overflow-x-auto', callResult[tool.name].success ? 'bg-green-50 border border-green-100' : 'bg-red-50 border border-red-100']">
              <div v-if="callResult[tool.name].success" class="flex items-start gap-2">
                <Terminal class="w-4 h-4 text-green-500 mt-0.5 flex-shrink-0" />
                <pre class="whitespace-pre-wrap text-surface-700">{{ callResult[tool.name].content }}</pre>
              </div>
              <div v-else class="flex items-start gap-2 text-red-500"><span class="text-sm">{{ callResult[tool.name].error }}</span></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>pre { max-height: 300px; }</style>