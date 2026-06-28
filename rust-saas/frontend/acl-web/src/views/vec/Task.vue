<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Clock, PauseCircle, CheckCircle, XCircle, AlertCircle, RefreshCw } from 'lucide-vue-next'
import { ElMessage, ElMessageBox } from 'element-plus'
import { taskApi, documentApi } from '@/vec/api'
import type { Task, Document } from '@/vec/types'
import { formatDate } from '@/vec/utils/date'

const tasks = ref<Task[]>([])
const documents = ref<Document[]>([])
const loading = ref(false)

const loadData = async () => {
  loading.value = true
  try {
    const [taskList, docList] = await Promise.all([
      taskApi.list(),
      documentApi.list({ page_size: 100 }),
    ])
    tasks.value = taskList.tasks || []
    documents.value = docList.documents || []
  } finally {
    loading.value = false
  }
}

const getStatusIcon = (status: string) => {
  switch (status) {
    case 'pending':
      return Clock
    case 'processing':
      return PauseCircle
    case 'completed':
      return CheckCircle
    case 'failed':
      return XCircle
    default:
      return AlertCircle
  }
}

const getStatusType = (status: string) => {
  switch (status) {
    case 'completed':
      return 'success'
    case 'processing':
      return 'warning'
    case 'failed':
      return 'danger'
    default:
      return 'info'
  }
}

const taskTypeMap: Record<string, string> = {
  document_process: '文档处理',
  document_reindex: '重新索引',
  document_distill: '知识蒸馏',
  batch_process: '批量处理',
}

const getTaskTypeLabel = (type?: string) => {
  return (type && taskTypeMap[type]) || type || '未知'
}

const getDocumentId = (row: Task) => {
  if (row.document_id) return row.document_id
  const payload = row.payload
  if (payload && typeof payload === 'object') {
    const id = payload.document_id
    if (id !== undefined && id !== null) return String(id)
  }
  return undefined
}

const getDocumentTitle = (documentId?: string) => {
  if (!documentId) return '-'
  const doc = documents.value.find((d) => String(d.id) === String(documentId))
  return doc?.topic || doc?.title || `文档 #${documentId}`
}

const handleCancel = async (row: Task) => {
  if (row.status === 'completed') return
  try {
    await ElMessageBox.confirm('确定要取消该任务吗？', '提示', { type: 'warning' })
    await taskApi.cancel(String(row.id))
    ElMessage.success('任务已取消')
    loadData()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error?.message || '取消任务失败')
    }
  }
}

onMounted(loadData)
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">任务管理</h1>
        <p class="page-subtitle">查看文档处理、重新索引等异步任务</p>
      </div>
      <button class="btn btn-outline" :disabled="loading" @click="loadData">
        <RefreshCw class="w-4 h-4" />
        刷新
      </button>
    </div>

    <el-card v-loading="loading">
      <template #header>
        <div class="flex items-center gap-2">
          <Clock class="w-4 h-4" />
          <span>任务列表</span>
        </div>
      </template>
      <el-table :data="tasks" row-key="id" style="width: 100%" empty-text="暂无任务">
        <el-table-column label="任务ID" width="80" show-overflow-tooltip>
          <template #default="{ row }">{{ row.id }}</template>
        </el-table-column>
        <el-table-column prop="task_type" label="任务类型" width="130">
          <template #default="{ row }">
            <el-tag type="primary">{{ getTaskTypeLabel(row.task_type) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="关联文档" min-width="160">
          <template #default="{ row }">{{ getDocumentTitle(getDocumentId(row)) }}</template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="140">
          <template #default="{ row }">
            <span class="flex items-center gap-2">
              <component :is="getStatusIcon(row.status)" class="w-4 h-4" />
              <el-tag :type="getStatusType(row.status)">{{ row.status }}</el-tag>
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="progress" label="进度" width="180">
          <template #default="{ row }">
            <el-progress :percentage="Math.round(row.progress || 0)" size="small" />
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="160">
          <template #default="{ row }">{{ formatDate(row.created_at, true) }}</template>
        </el-table-column>
        <el-table-column prop="message" label="消息" min-width="160" show-overflow-tooltip />
        <el-table-column label="操作" width="100">
          <template #default="{ row }">
            <el-button
              link
              type="danger"
              :disabled="row.status === 'completed' || row.status === 'failed'"
              @click="handleCancel(row)"
            >
              取消
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>
