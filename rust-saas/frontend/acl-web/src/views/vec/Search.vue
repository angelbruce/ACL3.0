<script setup lang="ts">
import { ref } from 'vue'
import { Search as SearchIcon, FileText } from 'lucide-vue-next'
import { searchApi } from '@/vec/api'
import type { SearchResult } from '@/vec/types'

const query = ref('')
const searchKey = ref('')
const results = ref<SearchResult[]>([])
const loading = ref(false)

const handleSearch = async () => {
  if (!query.value.trim()) return
  searchKey.value = query.value
  loading.value = true
  try {
    const res = await searchApi.query(searchKey.value, { limit: 20 })
    results.value = res.results || []
  } finally {
    loading.value = false
  }
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    handleSearch()
  }
}

// 高亮关键词（支持中英文）
const highlightText = (text: string, keyword: string): string => {
  if (!keyword.trim()) return escapeHtml(text)

  // 先转义 HTML 防止 XSS
  const escaped = escapeHtml(text)

  // 将关键词按空格分词，支持多关键词
  const keywords = keyword.trim().split(/\s+/).filter(k => k.length > 0)

  let result = escaped
  for (const kw of keywords) {
    // 转义关键词中的特殊字符
    const escapedKw = kw.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    const regex = new RegExp(`(${escapedKw})`, 'gi')
    result = result.replace(regex, '<mark class="highlight">$1</mark>')
  }

  return result
}

// HTML 转义
const escapeHtml = (text: string): string => {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">搜索</h1>
        <p class="page-subtitle">向量语义检索知识库内容</p>
      </div>
    </div>
    <el-card class="mb-6">
      <el-input
        v-model="query"
        placeholder="输入搜索关键词..."
        size="large"
        @keydown="handleKeyDown"
      >
        <template #prefix>
          <SearchIcon class="w-4 h-4 text-gray-400" />
        </template>
        <template #append>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
        </template>
      </el-input>
    </el-card>

    <el-card v-loading="loading" v-if="searchKey" :title="`搜索结果: &quot;${searchKey}&quot;`">
      <template #header>
        <div class="font-medium">搜索结果: "{{ searchKey }}"</div>
      </template>
      <div v-if="results.length > 0">
        <div
          v-for="item in results"
          :key="item.id"
          class="py-4 border-b border-gray-100 last:border-b-0"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex items-start gap-3 flex-1">
              <FileText class="w-5 h-5 text-gray-400 mt-0.5" />
              <div class="flex-1">
                <router-link
                  v-if="item.document_id"
                  :to="`/vec/documents/${item.document_id}`"
                  class="text-blue-600 hover:underline font-medium"
                >
                  {{ item.document_topic || item.document_title || `文档 #${item.document_id}` }}
                </router-link>
                <span v-else class="font-medium">{{ item.document_topic || item.document_title || '搜索结果' }}</span>
                <div class="mt-1 text-sm text-gray-600" v-html="highlightText(item.content, searchKey)"></div>
              </div>
            </div>
            <el-tag type="primary">{{ (item.score * 100).toFixed(1) }}%</el-tag>
          </div>
        </div>
      </div>
      <el-empty v-else description="暂无搜索结果" />
    </el-card>
  </div>
</template>

<style scoped>
:deep(.highlight) {
  background-color: #fef08a;
  color: #854d0e;
  padding: 0 2px;
  border-radius: 2px;
  font-weight: 500;
}
</style>
