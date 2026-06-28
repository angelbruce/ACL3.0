<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { FileText, Plus, Upload, Eye, RefreshCw, Trash2 } from 'lucide-vue-next'
import { ElMessage, ElMessageBox } from 'element-plus'
import { documentApi } from '@/vec/api'
import type { Document } from '@/vec/types'
import { formatDate } from '@/vec/utils/date'

const router = useRouter()

const documents = ref<Document[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const submitting = ref(false)
const form = reactive({
  topic: '',
  content: '',
})
const uploadFileInput = ref<HTMLInputElement | null>(null)

const loadDocuments = async () => {
  loading.value = true
  try {
    const res = await documentApi.list({ page_size: 50 })
    documents.value = res.documents || []
  } finally {
    loading.value = false
  }
}

const openCreate = () => {
  form.topic = ''
  form.content = ''
  dialogVisible.value = true
}

const handleSubmit = async () => {
  if (!form.topic.trim() || !form.content.trim()) {
    ElMessage.warning('请填写标题和内容')
    return
  }
  submitting.value = true
  try {
    await documentApi.createText({
      topic: form.topic,
      content: form.content,
    })
    ElMessage.success('创建成功')
    dialogVisible.value = false
    await loadDocuments()
  } finally {
    submitting.value = false
  }
}

const handleUploadClick = () => {
  uploadFileInput.value?.click()
}

const handleFileChange = async (e: Event) => {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  try {
    loading.value = true
    await documentApi.uploadFile(file, file.name)
    ElMessage.success('上传成功')
    await loadDocuments()
  } catch (err: any) {
    ElMessage.error(err?.message || '上传失败')
  } finally {
    loading.value = false
    input.value = ''
  }
}

const handleReindex = async (row: Document) => {
  try {
    await ElMessageBox.confirm('确定要重新索引该文档吗？', '提示', { type: 'warning' })
    await documentApi.reindex(String(row.id))
    ElMessage.success('已创建重新索引任务，请在任务管理查看进度')
  } catch {
    // cancelled
  }
}

const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定删除该文档吗？', '提示', { type: 'warning' })
    await documentApi.delete(id)
    ElMessage.success('删除成功')
    await loadDocuments()
  } catch {
    // cancelled
  }
}

onMounted(loadDocuments)
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">文档管理</h1>
        <p class="page-subtitle">管理知识库文档，支持文本创建和文件上传</p>
      </div>
      <div class="flex items-center gap-3">
        <button class="btn btn-outline" @click="handleUploadClick">
          <Upload class="w-4 h-4" />
          上传文件
        </button>
        <input ref="uploadFileInput" type="file" accept=".txt,.md,.json,.xml,.csv" class="hidden" @change="handleFileChange" />
        <button class="btn btn-primary" @click="openCreate">
          <Plus class="w-4 h-4" />
          新建文档
        </button>
      </div>
    </div>

    <el-card v-loading="loading">
        <template #header>
          <div class="flex items-center gap-2">
            <FileText class="w-4 h-4" />
            <span>文档列表</span>
          </div>
        </template>
        <el-table :data="documents" row-key="id" style="width: 100%" empty-text="暂无文档">
          <el-table-column label="标题" min-width="180" show-overflow-tooltip>
            <template #default="{ row }">{{ row.topic || row.title || `文档 #${row.id}` }}</template>
          </el-table-column>
          <el-table-column label="类型" width="100">
            <template #default="{ row }">
              <el-tag type="info">{{ row.source_type || row.file_type || row.document_type || '-' }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="100">
            <template #default="{ row }">
              <el-tag :type="row.indexed_at ? 'success' : 'info'">{{ row.indexed_at ? '已索引' : '未索引' }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="chunk_count" label="分块数" width="100" />
          <el-table-column prop="created_at" label="创建时间" width="160">
            <template #default="{ row }">{{ formatDate(row.created_at, true) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="220" fixed="right">
            <template #default="{ row }">
              <el-button link type="primary" :icon="Eye" @click="router.push(`/vec/documents/${row.id}`)">查看</el-button>
              <el-button link type="primary" :icon="RefreshCw" @click="handleReindex(row)">重新索引</el-button>
              <el-button link type="danger" :icon="Trash2" @click="handleDelete(row.id)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <el-dialog v-model="dialogVisible" title="新建文档" width="600px">
        <el-form label-position="top">
          <el-form-item label="标题" required>
            <el-input v-model="form.topic" placeholder="请输入标题" />
          </el-form-item>
          <el-form-item label="内容" required>
            <el-input v-model="form.content" type="textarea" :rows="6" placeholder="请输入内容" />
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" :loading="submitting" @click="handleSubmit">确定</el-button>
        </template>
      </el-dialog>
  </div>
</template>
