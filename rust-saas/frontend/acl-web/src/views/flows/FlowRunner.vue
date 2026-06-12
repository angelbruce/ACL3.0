<script setup lang="ts">
import { onMounted, ref, onUnmounted,  computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowLeft, Play, Square, 
} from 'lucide-vue-next'
import { useFlowStore, useLlmStore } from '@/stores'
import type { FlowRuntimeNode,  FlowRuntime, Flow } from '@/types'

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
}

const stopFlow = async () => {
  if (flowRuntime.value !== null) {
    await flowStore.stopFlow(flowRuntime.value.id)
    await fetchRuntime()
    await flowStore.fetchRuntimes(flowId)
  }
}

const currentRuntime = () => flowRuntime.value

const activeTab = ref('current')
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

const selectedHistoryRuntime = ref(0);
const historyNodes = ref<FlowRuntimeNode[]>([])
const handleHistoryRowClick = async (row: any) => {
  selectedHistoryRuntime.value = row.id
    let runtime = await flowStore.fetchRuntime(selectedHistoryRuntime.value)  
    if (runtime) {
      historyNodes.value = runtime.nodes || []
    } else {
      historyNodes.value = []
    }
}

const selectedHumanRow = ref<any>({})
const selectedHumanRowPrompt = ref('')
const handleHumanRowClick = (row: any) => {
  selectedHumanRow.value =row;
}

const formatOverName = (row: any) => {
  return row.is_over ? '是' : '进行中'
}



</script>

<template>
  <div class="flex h-full">
    <div class="w-full h-full min-w-[380px] border-r border-surface-200 flex flex-col">
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
            <!-- <p class="text-surface-500 text-sm mt-0.5">
              运行时 ID: {{ currentRuntime()?.id || '无' }}
            </p> -->
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

      <el-tabs v-model="activeTab" type="border-card" class="h-full flex flex-col">

        <el-tab-pane label="当前" name="current" class="h-full w-full" >
          <el-tabs v-model="activeCurrentTab" type="border-card" class="h-full flex flex-col">
              <el-tab-pane  name="running" class="h-full w-full" >
                <template #label>
                  <span class="text-blue-600 font-bold">进行中</span>
                </template>
                <div class="flex flex-1 flex-col h-full">
                  <el-table
                    :data="runningNodes"
                    style="width: 100%"
                    border
                    stripe
                    class="h-full w-full"
                    @row-click="handleHumanRowClick"
                  >
                    <el-table-column prop="action" label="节点"  />
                    <el-table-column prop="human" label="是否需要人工参与" :formatter="formatHuman"  />
                    <el-table-column prop="created_at" label="创建时间" width="300" />
                  </el-table>
                </div>
                <div class="w-1/2 h-full" v-show="selectedHumanRow.human == 1">
                    <textarea v-model="selectedHumanRowPrompt" class="w-full h-full" placeholder="请输入提示"></textarea>
                    <el-button type="primary" @click="handleHumanRowClick">确认</el-button>
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
                  <el-table-column prop="action" label="节点" />
                  <el-table-column prop="human" label="是否需要人工参与" :formatter="formatHuman"  />
                  <el-table-column prop="created_at" label="创建时间" width="300" />
                </el-table>
              </el-tab-pane>
              



          </el-tabs>
        </el-tab-pane>  

  
        <el-tab-pane label="历史记录" name="logs" class="h-full w-full" >
          <div class="flex flex-1 flex-col h-full">
            <el-table
              :data="runtimes"
              style="width: 100%"
              border
              stripe
              class="h-full w-full"
              @row-click="handleHistoryRowClick"
            >
              <el-table-column prop="created_at" label="创建时间" width="300" />
              <el-table-column prop="over_name" label="已完成"    :formatter="formatOverName"  />
            </el-table>
          </div>
         <div class="w-1/2 h-full">
            <el-table
            :data="historyNodes"
            style="width: 100%"
            border
            stripe
            class="h-full w-full"
          >
            <el-table-column prop="action" label="节点"  />
            <el-table-column prop="created_at" label="创建时间" width="300" />
          </el-table>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

</div>
</template>

<style scoped>
</style>