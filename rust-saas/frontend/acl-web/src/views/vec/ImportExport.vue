<script setup lang="ts">
import { ref } from 'vue'
import { Upload, FileText, GitFork } from 'lucide-vue-next'
import { ElMessage } from 'element-plus'
import { importExportApi } from '@/vec/api'
import type { UploadFile } from 'element-plus'
import type { ImportResult } from '@/vec/types'

const importing = ref(false)
const uploadProgress = ref(0)
const fileList = ref<UploadFile[]>([])
const importResult = ref<ImportResult | null>(null)

const handleChange = async () => {
  const files = fileList.value.map((f) => f.raw).filter(Boolean) as File[]
  if (files.length === 0) return
  importing.value = true
  uploadProgress.value = 0
  importResult.value = null
  try {
    const result = await importExportApi.importDocuments(files)
    importResult.value = result
    uploadProgress.value = 100
    if (result && result.failed_count > 0) {
      ElMessage.warning(`导入完成：成功 ${result.success_count} 个，失败 ${result.failed_count} 个`)
    } else {
      ElMessage.success(`导入成功 ${result?.success_count || 0} 个文档`)
    }
    fileList.value = []
  } catch {
    uploadProgress.value = 0
  } finally {
    importing.value = false
  }
}

const handleExportDocuments = async () => {
  try {
    const blob = await importExportApi.exportDocuments()
    downloadBlob(blob as Blob, 'documents.json')
  } catch {
    ElMessage.error('导出失败')
  }
}

const handleExportGraph = async () => {
  try {
    const data = await importExportApi.exportKnowledgeGraph()
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    downloadBlob(blob, 'knowledge-graph.json')
  } catch {
    ElMessage.error('导出失败')
  }
}

const downloadBlob = (blob: Blob, filename: string) => {
  const url = window.URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  window.URL.revokeObjectURL(url)
}
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">导入导出</h1>
        <p class="page-subtitle">批量导入文档或导出知识库数据</p>
      </div>
      <div class="flex items-center gap-3">
        <button class="btn btn-outline" @click="handleExportDocuments">
          <FileText class="w-4 h-4" />
          导出文档
        </button>
        <button class="btn btn-outline" @click="handleExportGraph">
          <GitFork class="w-4 h-4" />
          导出图谱
        </button>
      </div>
    </div>

    <el-card v-loading="importing">
        <template #header>
          <div class="flex items-center gap-2">
            <Upload class="w-4 h-4" />
            <span>导入文档</span>
          </div>
        </template>
        <el-upload
          v-model:file-list="fileList"
          drag
          multiple
          accept=".txt,.md,.json,.xml,.csv"
          :auto-upload="false"
          :on-change="handleChange"
        >
          <el-icon class="el-icon--upload"><Upload class="w-12 h-12 text-surface-400" /></el-icon>
          <div class="el-upload__text">
            拖拽文件到此处或 <em>点击上传</em>
          </div>
          <template #tip>
            <div class="el-upload__tip">支持 txt, md, json, xml, csv 等文本格式</div>
          </template>
        </el-upload>
        <div v-if="importResult" class="mt-4">
          <el-alert
            :type="importResult.failed_count > 0 ? 'warning' : 'success'"
            :closable="false"
            show-icon
          >
            <template #title>
              导入完成：成功 {{ importResult.success_count }} 个，失败 {{ importResult.failed_count }} 个，共 {{ importResult.total_count }} 个
            </template>
          </el-alert>
          <el-table v-if="importResult.errors.length > 0" :data="importResult.errors" class="mt-2" size="small">
            <el-table-column prop="index" label="序号" width="60" />
            <el-table-column prop="title" label="文件名" />
            <el-table-column prop="error" label="错误" />
          </el-table>
        </div>
      </el-card>
  </div>
</template>
