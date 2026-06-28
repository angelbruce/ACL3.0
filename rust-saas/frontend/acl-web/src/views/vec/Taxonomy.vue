<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { Tags, Plus, Edit, Trash2 } from 'lucide-vue-next'
import { ElMessage, ElMessageBox } from 'element-plus'
import { taxonomyApi } from '@/vec/api'
import type { Category } from '@/vec/types'

interface TreeNode {
  id: string
  label: string
  name: string
  document_count: number
  children?: TreeNode[]
}

const categories = ref<Category[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const dialogTitle = ref('新建分类')
const editingId = ref<string | null>(null)
const submitting = ref(false)
const form = reactive({ name: '' })

const buildTree = (list: Category[]): TreeNode[] => {
  return list.map((item) => ({
    id: item.id,
    label: item.category_name,
    name: item.category_name,
    document_count: item.document_count || 0,
    children: item.children ? buildTree(item.children) : undefined,
  }))
}

const treeData = ref<TreeNode[]>([])

const loadCategories = async () => {
  loading.value = true
  try {
    categories.value = await taxonomyApi.categories()
    treeData.value = buildTree(categories.value)
  } finally {
    loading.value = false
  }
}

const openCreate = () => {
  editingId.value = null
  dialogTitle.value = '新建分类'
  form.name = ''
  dialogVisible.value = true
}

const openEdit = (node: TreeNode) => {
  editingId.value = node.id
  dialogTitle.value = '编辑分类'
  form.name = node.name
  dialogVisible.value = true
}

const handleSubmit = async () => {
  if (!form.name.trim()) {
    ElMessage.warning('请输入分类名称')
    return
  }
  submitting.value = true
  try {
    if (editingId.value) {
      await taxonomyApi.updateCategory(editingId.value, { name: form.name })
    } else {
      await taxonomyApi.createCategory({ name: form.name })
    }
    ElMessage.success(editingId.value ? '更新成功' : '创建成功')
    dialogVisible.value = false
    await loadCategories()
  } finally {
    submitting.value = false
  }
}

const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定删除该分类吗？', '提示', { type: 'warning' })
    await taxonomyApi.deleteCategory(id)
    ElMessage.success('删除成功')
    await loadCategories()
  } catch {
    // cancelled
  }
}

onMounted(loadCategories)
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">知识分类</h1>
        <p class="page-subtitle">管理文档分类体系</p>
      </div>
      <button class="btn btn-primary" @click="openCreate">
        <Plus class="w-4 h-4" />
        新建分类
      </button>
    </div>

    <el-card v-loading="loading">
      <template #header>
        <div class="flex items-center gap-2">
          <Tags class="w-4 h-4" />
          <span>分类树</span>
        </div>
      </template>
      <el-tree :data="treeData" node-key="id" default-expand-all>
        <template #default="{ node, data }">
          <div class="flex items-center gap-2 py-1">
            <span>{{ node.label }}</span>
            <el-tag type="info" size="small">{{ data.document_count }} 文档</el-tag>
            <el-button link type="primary" :icon="Edit" @click.stop="openEdit(data)">编辑</el-button>
            <el-button link type="danger" :icon="Trash2" @click.stop="handleDelete(data.id)">删除</el-button>
          </div>
        </template>
      </el-tree>
    </el-card>

    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="500px">
      <el-form label-position="top">
        <el-form-item label="名称" required>
          <el-input v-model="form.name" placeholder="请输入分类名称" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>
