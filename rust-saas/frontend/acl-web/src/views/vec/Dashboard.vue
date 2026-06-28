<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { FileText, Search, GitFork, BarChart3 } from 'lucide-vue-next'
import { analyticsApi } from '@/vec/api'
import type { AnalyticsSummary } from '@/vec/types'

const summary = ref<AnalyticsSummary | null>(null)
const loading = ref(false)

const stats = [
  { title: '文档总数', key: 'total_documents', icon: FileText, suffix: '份', color: 'text-blue-500' },
  { title: '搜索查询', key: 'total_searches', icon: Search, suffix: '次', color: 'text-green-500' },
  { title: '实体数量', key: 'total_entities', icon: GitFork, suffix: '个', color: 'text-purple-500' },
  { title: '访问次数', key: 'total_views', icon: BarChart3, suffix: '次', color: 'text-orange-500' },
]

const loadSummary = async () => {
  loading.value = true
  try {
    summary.value = await analyticsApi.summary()
  } finally {
    loading.value = false
  }
}

onMounted(loadSummary)
</script>

<template>
  <div class="p-6" v-loading="loading">
    <div class="page-header">
      <div>
        <h1 class="page-title">仪表盘</h1>
        <p class="page-subtitle">知识库核心指标概览</p>
      </div>
    </div>
    <el-row :gutter="16">
      <el-col :span="6" :xs="24" :sm="12" :md="12" :lg="6" v-for="(stat, index) in stats" :key="index" class="mb-4">
        <el-card>
          <div class="flex items-center gap-4">
            <component :is="stat.icon" :class="['w-8 h-8', stat.color]" />
            <div>
              <div class="text-sm text-gray-500">{{ stat.title }}</div>
              <div class="text-2xl font-semibold">
                {{ summary ? (summary as any)[stat.key] : 0 }}
                <span class="text-sm text-gray-400">{{ stat.suffix }}</span>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>
