<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Sparkles, PlayCircle } from 'lucide-vue-next'
import { ElMessage } from 'element-plus'
import { documentApi } from '@/vec/api'
import type { Document, KnowledgePoint } from '@/vec/types'
import { formatDate } from '@/vec/utils/date'

const selectedDoc = ref<number | ''>('')
const documents = ref<Document[]>([])
const knowledgePoints = ref<KnowledgePoint[]>([])
const loadingDocs = ref(false)
const loadingPoints = ref(false)
const dialogVisible = ref(false)
const distilling = ref(false)

const loadDocuments = async () => {
  loadingDocs.value = true
  try {
    const res = await documentApi.list({ page_size: 100 })
    documents.value = res.documents || []
  } finally {
    loadingDocs.value = false
  }
}

const loadKnowledgePoints = async () => {
  if (!selectedDoc.value) return
  loadingPoints.value = true
  try {
    knowledgePoints.value = await documentApi.getKnowledgePoints(selectedDoc.value)
  } finally {
    loadingPoints.value = false
  }
}

const handleDocChange = (value: number) => {
  selectedDoc.value = value
  loadKnowledgePoints()
}

const handleDistill = async () => {
  if (!selectedDoc.value) return
  distilling.value = true
  try {
    await documentApi.distill(selectedDoc.value)
    ElMessage.success('知识蒸馏已触发')
    dialogVisible.value = false
    await loadKnowledgePoints()
  } finally {
    distilling.value = false
  }
}

const getConfidenceType = (confidence: number) => {
  if (confidence > 0.8) return 'success'
  if (confidence > 0.5) return 'warning'
  return 'danger'
}

onMounted(loadDocuments)
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">知识蒸馏</h1>
        <p class="page-subtitle">从文档中提取知识要点</p>
      </div>
      <button class="btn btn-primary" :disabled="!selectedDoc" @click="dialogVisible = true">
        <PlayCircle class="w-4 h-4" />
        触发蒸馏
      </button>
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
          :label="doc.topic || doc.title || `文档 #${doc.id}`"
          :value="doc.id"
        />
      </el-select>
    </el-card>

    <el-card v-if="selectedDoc" v-loading="loadingPoints">
      <template #header>
        <div class="flex items-center gap-2">
          <Sparkles class="w-4 h-4" />
          <span>知识要点列表</span>
        </div>
      </template>
      <el-table :data="knowledgePoints" row-key="id" style="width: 100%">
        <el-table-column prop="point_type" label="类型" width="120">
          <template #default="{ row }">
            <el-tag type="info">{{ row.point_type || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="point_content" label="内容" show-overflow-tooltip />
        <el-table-column prop="confidence" label="置信度" width="120">
          <template #default="{ row }">
            <el-tag :type="getConfidenceType(row.confidence)">{{ ((row.confidence || 0) * 100).toFixed(1) }}%</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="160">
          <template #default="{ row }">{{ formatDate(row.created_at, true) }}</template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="dialogVisible" title="确认蒸馏" width="400px">
      <p>确定要对该文档进行知识蒸馏吗？</p>
      <p class="text-gray-500 text-sm mt-2">蒸馏过程可能需要一些时间，请耐心等待。</p>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="distilling" @click="handleDistill">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>
