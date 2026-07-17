<script setup lang="ts">
import { onMounted, ref, onUnmounted,  computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowLeft, LinkIcon, PlayIcon, Square
} from 'lucide-vue-next'
import { useFlowStore, useLlmStore } from '@/stores'
import type { FlowRuntimeNode,  FlowRuntime, Flow } from '@/types'
import FlowRunnerHuman from '@/views/flows/FlowRunnerHuman.vue'

const route = useRoute()
const router = useRouter()
const flowStore = useFlowStore()
const flowId = Number(route.params.id)
const loading = ref(true)
const nodes = ref<FlowRuntimeNode[]>([])
const pollingInterval = ref<number | null>(null)
const flowRuntime  = ref<FlowRuntime | null>(null)
const flow = ref<Flow | null>(null)

onMounted(async () => {
    flow.value = await flowStore.fetchFlow(flowId)
    fetchRuntime(),
    await Promise.all([
      fetchRuntime(),
    ])
    pollingInterval.value = window.setInterval(fetchRuntime, 5000)
    await flowStore.fetchRuntimes(flowId)
})


const runtimes = computed(() => {
    return  flowStore.runtimes || []
  }
)

onUnmounted(() => {
  if (pollingInterval.value) {
    clearInterval(pollingInterval.value)
  }
})

const fetchRuntime = async () => {
  try {
    flowRuntime.value = await flowStore.getFlowRuntimeByFlowId(flowId)
    if (flowRuntime.value !== null) {
      let runtime = await flowStore.fetchRuntime(flowRuntime.value.id)
      if (runtime) {
        nodes.value = runtime.nodes || []
      }
    }

  } finally {
    loading.value = false
  }
}

const startFlow = async () => {
  flowRuntime.value = await flowStore.startFlow(flowId)
  await fetchRuntime()
  await flowStore.fetchRuntimes(flowId)
  activeTab.value ='current'
}

const stopFlow = async () => {
  if (flowRuntime.value !== null) {
    await flowStore.stopFlow(flowRuntime.value.id)
    await fetchRuntime()
    await flowStore.fetchRuntimes(flowId)
  }
}

const currentRuntime = () => flowRuntime.value

const activeTab = ref('logs')
const activeCurrentTab = ref('running')


const completedNodes = computed(() =>  {
    return nodes.value?.filter(x=>x.status == 'Stop') || []
})

const runningNodes = computed(() =>  {
    return nodes.value?.filter(x=>x.status == 'Running' || x.status == 'RunningOver') || []
})


const formatHuman = (row: any) => {
  return row.human ? '是' : '否'
}

const selectedHistoryRuntimeId = ref(0);
const selectedHistoryRuntime = ref<FlowRuntime | null>(null);
const historyNodes = ref<FlowRuntimeNode[]>([])
const handleHistoryRowClick = async (row: any) => {
  selectedHistoryRuntimeId.value = row.id
  selectedHistoryRuntime.value = row
  let runtime = await flowStore.fetchRuntime(selectedHistoryRuntimeId.value)  
  if (runtime) {
    historyNodes.value = runtime.nodes || []
  } else {
    historyNodes.value = []
  }
}

// 根据当前选择的tab返回正确的runtimeId
const displayRuntimeId = computed(() => {
  if (activeTab.value === 'logs' && selectedHistoryRuntimeId.value > 0) {
    return selectedHistoryRuntimeId.value
  }
  return flowRuntime.value?.id || 0
})

// 根据当前选择的tab返回正确的flowId
const displayFlowId = computed(() => {
  if (activeTab.value === 'logs' && selectedHistoryRuntimeId.value > 0) {
    return selectedHistoryRuntime.value?.flow_id || flowId
  }
  return flowId
})

const selectedHumanRow = ref<any>({})
const selectedHumanRowPrompt = ref('')
const handleHumanRowClick = (row: any) => {
  selectedHumanRow.value = row;
}

const canInput = computed(() => {
  return runningNodes.value.some(n => n.human === 1)
})

const humanInputNode = computed(() => {
  return runningNodes.value.find(n => n.human === 1)
})

const sendHumanInput = async () => {
  if (!humanInputNode.value || !selectedHumanRowPrompt.value.trim()) return
  
  try {
    await flowStore.sendHumanInput(flowId, humanInputNode.value.id, selectedHumanRowPrompt.value.trim())
    selectedHumanRowPrompt.value = ''
    await fetchRuntime()
  } catch (error) {
    console.error('发送人工输入失败:', error)
  }
}

const formatOverName = (row: any) => {
  return row.is_over ? '是' : '进行中'
}

const formatterDate = (row:any) => {
  let d : number = Date.parse(row.created_at)
  let date = new Date(d);
  return date.getFullYear() + "-" 
                            + padLeft(date.getMonth()+"",2,"0")
                            + "-" 
                            + padLeft(date.getDay()+"",2,"0")
                            +" "
                            + padLeft(date.getHours()+"",2,"0")
                            + ":" 
                            + padLeft(date.getMinutes()+"",2,"0")
                            + ":" 
                            + padLeft(date.getSeconds()+"",2,"0")
}

const padLeft = (str:string,len: number,pad:string): string => {
  if (!str) return "";
  while (str.length < len) {
    str = pad + str;
  }

  return str;
}

const formatterName = (row:any) => {
  let name = row.action;
  let human = row.human == '0' ? 'AI' : 'Human'
  return `<${human}>${name}`;
}

watch(()=> flowRuntime.value?.is_over, async(newVal) => {
  if(newVal) {
    activeTab.value='current';
  } else {
    handleHistoryRowClick({flowId:flowId,id:runtimes.value[0].id})
    activeTab.value = 'logs';
  }
})

</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex-shrink-0">
      <div class="p-4 border-b border-surface-200 justify-between items-center flex flex-row">
        <div class="flex items-center gap-2">
          <button
            @click="router.push('/flows')"
            class="p-2 rounded-lg hover:bg-surface-100 transition-colors text-surface-500"
          >
            <ArrowLeft class="w-5 h-5" />
          </button>
          <div class="flex-1">
            <h1 class="text-xl font-bold text-surface-900">{{ flowStore.currentFlow?.name || '工作流执行' }}</h1>          
          </div>
        </div>
        <div class="flex gap-2">
          <button
            v-if="!currentRuntime()"
            @click="startFlow"
            class="btn btn-primary flex items-center gap-2 !bg-emerald-500 text-sm"
          >
            <PlayIcon class="w-4 h-4" /> 启动工作流
          </button>
          <button
            v-else
            @click="stopFlow"
            class="btn btn-danger flex items-center gap-2 text-sm"
          >
            <Square class="w-4 h-4" /> 停止工作流
          </button>
          <button
            @click="router.push(`/flows/${flowId}/edit`)"
            class="btn btn-primary flex items-center gap-2 text-sm"
          >
            <LinkIcon class="w-4 h-4" /> 工作流
          </button>
        </div>
      </div>
    </div>
    <div class="flex-1 flex flex-row h-full">
     
      
      <div class="flex-1 h-full">
        <FlowRunnerHuman :flowId="displayFlowId" :flowRuntimeId="displayRuntimeId" />
      </div>

      <div class="h-full" style="width:400px;">
        <el-tabs v-model="activeTab" type="border-card" class="h-full flex flex-col">
          <el-tab-pane label="过往执行记录" name="logs" class="h-full w-full" >
            <div class="flex-1 flex flex-row h-full">
              <div class="flex-1 h-full">
                <el-table
                  :data="runtimes"
                  border
                  stripe
                  class="h-full w-full"
                  @row-click="handleHistoryRowClick"
                >
                  <el-table-column prop="created_at" label="创建时间"  :formatter="formatterDate" />
                  <!-- <el-table-column prop="over_name" label="已完成" :formatter="formatOverName" /> -->
                </el-table>
              </div>
              <div class="flex-1 h-full">
                <el-table
                  :data="historyNodes"
                  style="width: 100%"
                  border
                  stripe
                  class="h-full w-full"
                >
                  <el-table-column prop="action" label="节点" />
                  <!-- <el-table-column prop="created_at" label="创建时间" width="180"  :formatter="formatterDate"  /> -->
                </el-table>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="当前" name="current" class="h-full w-full" v-if="flowRuntime && !flowRuntime.is_over" >
            <el-tabs v-model="activeCurrentTab" type="border-card" class="h-full flex flex-col">
                <el-tab-pane  name="running" class="h-full w-full" >
                  <template #label>
                    <span class="text-blue-600 font-bold">进行中</span>
                  </template>
                  <div class="flex flex-col h-full">
                    <el-table
                      :data="runningNodes"
                      style="width: 100%"
                      border
                      stripe
                      class="flex-1"
                      @row-click="handleHumanRowClick"
                    >
                      <el-table-column prop="action" label="节点"   :formatter="formatterName"></el-table-column>
                      <el-table-column prop="created_at" label="创建时间" width="180" :formatter="formatterDate"  />
                    </el-table>
                    <div v-if="humanInputNode" class="mt-4 p-4 border border-surface-200 rounded-lg">
                      <div class="mb-2">
                        <label class="text-sm font-medium text-surface-700">人工输入 - {{ humanInputNode.action }}</label>
                      </div>
                      <textarea 
                        v-model="selectedHumanRowPrompt" 
                        :disabled="!canInput"
                        class="w-full h-32 p-3 border border-surface-300 rounded-lg resize-none" 
                        :placeholder="canInput ? '请输入人工输入内容...' : '当前没有需要人工输入的节点'"
                      ></textarea>
                      <div class="mt-2 flex justify-end">
                        <el-button type="primary" @click="sendHumanInput" :disabled="!canInput || !selectedHumanRowPrompt.trim()">
                          {{ canInput ? '发送人工输入' : '等待人工输入节点' }}
                        </el-button>
                      </div>
                    </div>
                  </div>
                </el-tab-pane>
                <el-tab-pane  name="completed" class="h-full w-full" >
                  <template #label>
                    <span class="text-gray-500 font-bold">已完成</span>
                  </template>
                  <el-table 
                    :data="completedNodes"
                    border
                    stripe
                    class="h-full w-full"
                  >
                    <el-table-column prop="action" label="节点"  :formatter="formatterName" />
                    <el-table-column prop="created_at" label="创建时间" width="180"  :formatter="formatterDate"  />
                  </el-table>
                </el-tab-pane>

            </el-tabs>
          </el-tab-pane>  

        </el-tabs>
      </div>

    
    </div>

  </div>
</template>

<style scoped>
</style>