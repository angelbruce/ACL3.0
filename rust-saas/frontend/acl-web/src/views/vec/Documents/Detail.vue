<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, Sparkles, RefreshCw } from 'lucide-vue-next'
import { ElMessage, ElMessageBox } from 'element-plus'
import { documentApi } from '@/vec/api'
import type { Document, KnowledgePoint } from '@/vec/types'
import { formatDate } from '@/vec/utils/date'

const route = useRoute()
const router = useRouter()
const id = route.params.id as string

const document = ref<Document | null>(null)
const knowledgePoints = ref<KnowledgePoint[]>([])
const loading = ref(false)

const loadData = async () => {
  loading.value = true
  try {
    const [doc, points] = await Promise.all([
      documentApi.get(id),
      documentApi.getKnowledgePoints(id),
    ])
    document.value = doc
    knowledgePoints.value = points
  } finally {
    loading.value = false
  }
}

const getVisibilityType = (visibility: string) => {
  if (visibility === 'public') return 'success'
  if (visibility === 'private') return 'danger'
  return 'warning'
}

const handleReindex = async () => {
  try {
    await ElMessageBox.confirm('确定要重新索引该文档吗？', '提示', { type: 'warning' })
    await documentApi.reindex(id)
    ElMessage.success('已创建重新索引任务')
  } catch {
    // cancelled
  }
}

const handleDistill = async () => {
  try {
    await ElMessageBox.confirm('确定要对该文档进行知识蒸馏吗？', '提示', { type: 'warning' })
    await documentApi.distill(id)
    ElMessage.success('知识蒸馏已触发')
    await documentApi.getKnowledgePoints(id)
    await loadData()
  } catch {
    // cancelled
  }
}

onMounted(loadData)
</script>

<template>
  <div class="p-6" v-loading="loading">
    <div class="page-header">
      <div class="flex items-center gap-4">
        <button class="btn btn-ghost" @click="router.push('/vec/documents')">
          <ArrowLeft class="w-4 h-4" />
          返回列表
        </button>
        <h1 class="page-title">{{ document?.topic || document?.title || '文档详情' }}</h1>
      </div>
      <div v-if="document" class="flex items-center gap-3">
        <button class="btn btn-outline" @click="handleDistill">
          <Sparkles class="w-4 h-4" />
          触发蒸馏
        </button>
        <button class="btn btn-primary" @click="handleReindex">
          <RefreshCw class="w-4 h-4" />
          重新索引
        </button>
      </div>
    </div>

    <template v-if="document">
      <el-card class="mb-6">
        <el-descriptions :column="3" border>
          <el-descriptions-item label="类型">
            <el-tag type="info">{{ document.source_type || document.file_type || document.document_type || '-' }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="document.indexed_at ? 'success' : 'info'">{{ document.indexed_at ? '已索引' : '未索引' }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="可见性">
            <el-tag :type="getVisibilityType(document.visibility || 'public')">{{ document.visibility || 'public' }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="版本">{{ document.version || '-' }}</el-descriptions-item>
          <el-descriptions-item label="字数">{{ document.word_count || '-' }}</el-descriptions-item>
          <el-descriptions-item label="分块数">{{ document.chunk_count || 0 }}</el-descriptions-item>
          <el-descriptions-item label="创建时间" :span="3">{{ formatDate(document.created_at, true) }}</el-descriptions-item>
          <el-descriptions-item label="更新时间" :span="3">{{ formatDate(document.updated_at, true) }}</el-descriptions-item>
        </el-descriptions>
      </el-card>

      <el-card class="mb-6">
        <template #header>
          <span class="font-medium">文档内容</span>
        </template>
        <div class="whitespace-pre-wrap text-gray-800 leading-relaxed">{{ document.content }}</div>
      </el-card>

      <el-card v-if="knowledgePoints.length > 0" class="mb-6">
        <template #header>
          <div class="flex items-center gap-2">
            <Sparkles class="w-4 h-4" />
            <span class="font-medium">知识要点</span>
          </div>
        </template>
        <div class="space-y-4">
          <div v-for="point in knowledgePoints" :key="point.id" class="p-4 bg-gray-50 rounded-lg">
            <p class="text-gray-800">{{ point.content }}</p>
            <div class="flex items-center gap-4 mt-2 flex-wrap">
              <el-tag type="primary">置信度: {{ (point.confidence * 100).toFixed(1) }}%</el-tag>
              <el-tag v-for="(keyword, idx) in point.keywords" :key="idx">{{ keyword }}</el-tag>
            </div>
          </div>
        </div>
      </el-card>

    </template>
  </div>
</template>
