<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { History, RotateCcw, GitCompare } from 'lucide-vue-next'
import { ElMessage, ElMessageBox } from 'element-plus'
import { documentApi, versionApi } from '@/vec/api'
import type { Document, Version } from '@/vec/types'
import { formatDate } from '@/vec/utils/date'

const selectedDoc = ref('')
const documents = ref<Document[]>([])
const versions = ref<Version[]>([])
const loadingDocs = ref(false)
const loadingVersions = ref(false)
const rollingBackId = ref<string | null>(null)

const loadDocuments = async () => {
  loadingDocs.value = true
  try {
    const res = await documentApi.list({ page_size: 100 })
    documents.value = res.documents || []
  } finally {
    loadingDocs.value = false
  }
}

const loadVersions = async () => {
  if (!selectedDoc.value) return
  loadingVersions.value = true
  try {
    versions.value = await versionApi.list(selectedDoc.value)
  } catch {
    versions.value = []
  } finally {
    loadingVersions.value = false
  }
}

const handleDocChange = (value: string) => {
  selectedDoc.value = value
  loadVersions()
}

const handleRollback = async (version: Version) => {
  try {
    await ElMessageBox.confirm(`确定要回滚到版本 ${version.version_number} 吗？`, '提示', { type: 'warning' })
    rollingBackId.value = version.id
    await versionApi.rollback(selectedDoc.value, Number(version.id))
    ElMessage.success('回滚成功')
    await loadVersions()
  } catch {
    // cancelled
  } finally {
    rollingBackId.value = null
  }
}

const getStatusType = (status: string) => {
  if (status === 'completed') return 'success'
  if (status === 'processing') return 'warning'
  return 'danger'
}

onMounted(loadDocuments)
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">版本管理</h1>
        <p class="page-subtitle">查看与回滚文档版本</p>
      </div>
    </div>

    <el-card class="mb-6">
      <el-select
        v-model="selectedDoc"
        placeholder="选择文档"
        style="width: 300px"
        :loading="loadingDocs"
        @change="handleDocChange"
      >
        <el-option
          v-for="doc in documents"
          :key="doc.id"
          :label="`${doc.topic || doc.title || '文档 #' + doc.id} (v${doc.version || 0})`"
          :value="doc.id"
        />
      </el-select>
    </el-card>

    <el-card v-if="selectedDoc" v-loading="loadingVersions">
      <template #header>
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <History class="w-4 h-4" />
            <span>版本列表</span>
          </div>
          <el-button :icon="GitCompare" size="small">对比版本</el-button>
        </div>
      </template>
      <el-table :data="versions" row-key="id" style="width: 100%">
        <el-table-column prop="version_number" label="版本号" width="100">
          <template #default="{ row }">
            <el-tag type="primary">v{{ row.version_number }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="title" label="标题" show-overflow-tooltip />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">{{ row.status }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="change_summary" label="变更摘要" show-overflow-tooltip />
        <el-table-column prop="created_at" label="创建时间" width="160">
          <template #default="{ row }">{{ formatDate(row.created_at, true) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button link type="primary" :icon="RotateCcw" :loading="rollingBackId === row.id" @click="handleRollback(row)">回滚</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>
