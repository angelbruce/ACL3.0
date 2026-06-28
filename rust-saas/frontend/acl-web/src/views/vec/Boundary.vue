<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Globe, Plus, Trash2 } from 'lucide-vue-next'
import { ElMessage, ElMessageBox } from 'element-plus'
import { documentApi, boundaryApi } from '@/vec/api'
import type { Document, Share } from '@/vec/types'
import { formatDate } from '@/vec/utils/date'

const selectedDoc = ref('')
const documents = ref<Document[]>([])
const shares = ref<Share[]>([])
const loadingDocs = ref(false)
const loadingShares = ref(false)
const dialogVisible = ref(false)
const visibility = ref<'public' | 'private' | 'restricted'>('public')
const settingVisibility = ref(false)

const loadDocuments = async () => {
  loadingDocs.value = true
  try {
    const res = await documentApi.list({ page_size: 100 })
    documents.value = res.documents || []
  } finally {
    loadingDocs.value = false
  }
}

const loadShares = async () => {
  if (!selectedDoc.value) return
  loadingShares.value = true
  try {
    shares.value = await boundaryApi.shares(selectedDoc.value)
  } finally {
    loadingShares.value = false
  }
}

const handleDocChange = (value: string) => {
  selectedDoc.value = value
  const doc = documents.value.find((d) => d.id === value)
  visibility.value = doc?.visibility || 'public'
  loadShares()
}

const handleSetVisibility = async () => {
  if (!selectedDoc.value) return
  settingVisibility.value = true
  try {
    await boundaryApi.setVisibility(selectedDoc.value, { visibility: visibility.value })
    ElMessage.success('可见性设置成功')
    dialogVisible.value = false
    await loadDocuments()
  } finally {
    settingVisibility.value = false
  }
}

const handleDeleteShare = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定删除该共享吗？', '提示', { type: 'warning' })
    await boundaryApi.deleteShare(id)
    ElMessage.success('删除成功')
    await loadShares()
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
        <h1 class="page-title">知识边界</h1>
        <p class="page-subtitle">管理文档可见性与共享</p>
      </div>
      <button class="btn btn-primary" :disabled="!selectedDoc" @click="dialogVisible = true">
        <Globe class="w-4 h-4" />
        设置可见性
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
          :label="`${doc.topic || doc.title || `文档 #${doc.id}`} (${doc.visibility || 'public'})`"
          :value="doc.id"
        />
      </el-select>
    </el-card>

    <el-card v-if="selectedDoc" v-loading="loadingShares">
      <template #header>
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Globe class="w-4 h-4" />
            <span>共享列表</span>
          </div>
          <el-button :icon="Plus" size="small">添加共享</el-button>
        </div>
      </template>
      <el-table :data="shares" row-key="id" style="width: 100%">
        <el-table-column label="目标类型" width="120">
          <template #default="{ row }">
            <el-tag type="info">{{ row.target_type || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="目标ID" width="120">
          <template #default="{ row }">{{ row.target_id || '-' }}</template>
        </el-table-column>
        <el-table-column label="共享类型" width="120">
          <template #default="{ row }">
            <el-tag type="primary">{{ row.share_type || '-' }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="创建时间" width="180">
          <template #default="{ row }">{{ formatDate(row.created_at, true) }}</template>
        </el-table-column>
        <el-table-column label="过期时间" width="180">
          <template #default="{ row }">{{ row.expire_at ? formatDate(row.expire_at, true) : '永不过期' }}</template>
        </el-table-column>
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button link type="danger" :icon="Trash2" @click="handleDeleteShare(row.id)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="dialogVisible" title="设置可见性" width="400px">
      <el-radio-group v-model="visibility" class="flex flex-col gap-3">
        <el-radio value="public">公开 - 所有人可见</el-radio>
        <el-radio value="private">私有 - 仅自己可见</el-radio>
        <el-radio value="restricted">受限 - 指定用户可见</el-radio>
      </el-radio-group>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="settingVisibility" @click="handleSetVisibility">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>
