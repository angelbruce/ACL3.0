<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { workspaceService, type SubscribedBoard, type SharedFileInfo } from '@/api/workspace'

const subscribedBoards = ref<SubscribedBoard[]>([])
const loading = ref(true)
const error = ref('')

const fetchSubscriptions = async () => {
  loading.value = true
  error.value = ''
  try {
    subscribedBoards.value = await workspaceService.getSubscribedBoards()
  } catch (e) {
    error.value = 'Failed to load subscriptions'
    console.error(e)
  } finally {
    loading.value = false
  }
}

const handleUnsubscribe = async (boardId: number) => {
  if (!confirm('Are you sure you want to unsubscribe from this board?')) return
  
  try {
    await workspaceService.unsubscribeBoard(boardId)
    subscribedBoards.value = subscribedBoards.value.filter(b => b.board.id !== boardId)
  } catch (e) {
    console.error('Failed to unsubscribe:', e)
  }
}

const handleDownloadSharedFile = async (boardId: number, file: SharedFileInfo) => {
  try {
    const response = await workspaceService.downloadSharedFile(boardId, file.file_path)
    const blob = response.data
    const url = window.URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = file.file_name
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    window.URL.revokeObjectURL(url)
  } catch (e) {
    console.error('Failed to download file:', e)
  }
}

const formatDate = (dateStr: string): string => {
  return new Date(dateStr).toLocaleDateString()
}

onMounted(fetchSubscriptions)
</script>

<template>
  <div class="subscribed-container">
    <div class="header">
      <h2>我的订阅</h2>
      <p class="text-gray-500">查看已订阅看板中的共享文件（只读）</p>
    </div>

    <div v-if="loading" class="loading">
      <div class="loader"></div>
    </div>

    <div v-else-if="error" class="error">
      {{ error }}
    </div>

    <div v-else class="subscriptions-list">
      <div v-if="subscribedBoards.length === 0" class="empty-state">
        <div class="empty-icon">📭</div>
        <p>暂无订阅的看板</p>
        <p class="text-sm">订阅公开看板后可以在这里查看共享文件</p>
      </div>

      <div
        v-for="subscribed in subscribedBoards"
        :key="subscribed.board.id"
        class="subscription-card"
      >
        <div class="subscription-header">
          <div>
            <h3>{{ subscribed.board.name }}</h3>
            <p class="text-sm text-gray-500">{{ subscribed.board.description || '暂无描述' }}</p>
          </div>
          <button class="btn btn-sm btn-danger" @click="handleUnsubscribe(subscribed.board.id)">
            取消订阅
          </button>
        </div>

        <div v-if="subscribed.items.length === 0" class="empty-files">
          <p class="text-gray-400 text-sm">此看板暂无共享文件</p>
        </div>

        <div v-else class="shared-files">
          <div
            v-for="item in subscribed.items"
            :key="item.id"
            class="shared-file-item"
          >
            <div class="file-icon">📄</div>
            <div class="file-info">
              <span class="file-name">{{ item.file_name }}</span>
              <span class="file-meta">共享时间：{{ formatDate(item.shared_at) }}</span>
            </div>
            <button 
              class="btn btn-sm btn-primary"
              @click="handleDownloadSharedFile(subscribed.board.id, item)"
            >
              下载
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.subscribed-container {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

.header {
  margin-bottom: 20px;
}

.header h2 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 4px;
}

.loading {
  display: flex;
  justify-content: center;
  padding: 40px;
}

.loader {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error {
  color: #ef4444;
  padding: 20px;
  text-align: center;
}

.subscriptions-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.empty-state {
  padding: 60px 20px;
  text-align: center;
  color: #9ca3af;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.subscription-card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.subscription-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.subscription-header h3 {
  margin: 0 0 4px 0;
  font-size: 18px;
}

.subscription-header p {
  margin: 0;
}

.empty-files {
  padding: 20px;
}

.shared-files {
  padding: 8px;
}

.shared-file-item {
  display: flex;
  align-items: center;
  padding: 12px;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.shared-file-item:hover {
  background-color: #f9fafb;
}

.file-icon {
  font-size: 24px;
  margin-right: 12px;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  display: block;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  display: block;
  font-size: 12px;
  color: #6b7280;
  margin-top: 2px;
}

.btn {
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: background-color 0.2s;
}

.btn-sm {
  padding: 4px 8px;
  font-size: 12px;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
}

.btn-primary:hover {
  background-color: #2563eb;
}

.btn-danger {
  background-color: #ef4444;
  color: white;
}

.btn-danger:hover {
  background-color: #dc2626;
}
</style>
