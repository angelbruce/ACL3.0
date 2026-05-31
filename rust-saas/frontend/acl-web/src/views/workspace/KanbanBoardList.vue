<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Edit2, Trash2 } from 'lucide-vue-next'
import { workspaceService, type KanbanBoard, type CreateKanbanBoardRequest, type UpdateKanbanBoardRequest } from '@/api/workspace'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const boards = ref<KanbanBoard[]>([])
const loading = ref(true)
const error = ref('')

const showCreateModal = ref(false)
const newBoardName = ref('')
const newBoardDescription = ref('')

const showEditModal = ref(false)
const editingBoard = ref<KanbanBoard | null>(null)
const editBoardName = ref('')
const editBoardDescription = ref('')

const showDeleteConfirm = ref(false)
const deleteBoardId = ref<number | null>(null)

const fetchBoards = async () => {
  loading.value = true
  error.value = ''
  try {
    boards.value = await workspaceService.getPublicKanbanBoards()
  } catch (e) {
    error.value = 'Failed to load boards'
    console.error(e)
  } finally {
    loading.value = false
  }
}

const handleCreateBoard = async () => {
  if (!newBoardName.value.trim()) return
  
  try {
    const request: CreateKanbanBoardRequest = {
      name: newBoardName.value,
      description: newBoardDescription.value || undefined,
      is_public: true
    }
    
    await workspaceService.createKanbanBoard(request)
    showCreateModal.value = false
    newBoardName.value = ''
    newBoardDescription.value = ''
    await fetchBoards()
  } catch (e) {
    console.error('Failed to create board:', e)
  }
}

const handleEditBoard = (board: KanbanBoard) => {
  editingBoard.value = board
  editBoardName.value = board.name
  editBoardDescription.value = board.description || ''
  showEditModal.value = true
}

const handleSaveEdit = async () => {
  if (!editingBoard.value || !editBoardName.value.trim()) return
  
  try {
    const request: UpdateKanbanBoardRequest = {
      name: editBoardName.value,
      description: editBoardDescription.value,
      is_public: true
    }
    
    await workspaceService.updateKanbanBoard(editingBoard.value.id, request)
    showEditModal.value = false
    editingBoard.value = null
    await fetchBoards()
  } catch (e) {
    console.error('Failed to update board:', e)
  }
}

const handleDeleteBoard = (boardId: number) => {
  deleteBoardId.value = boardId
  showDeleteConfirm.value = true
}

const confirmDelete = async () => {
  if (!deleteBoardId.value) return
  
  try {
    await workspaceService.deleteKanbanBoard(deleteBoardId.value)
    await fetchBoards()
  } catch (e) {
    console.error('Failed to delete board:', e)
  }
  
  showDeleteConfirm.value = false
  deleteBoardId.value = null
}

const cancelDelete = () => {
  showDeleteConfirm.value = false
  deleteBoardId.value = null
}

const handleSubscribe = async (boardId: number) => {
  try {
    await workspaceService.subscribeBoard(boardId)
    await fetchBoards()
  } catch (e) {
    console.error('Failed to subscribe:', e)
  }
}

const formatDate = (dateStr: string): string => {
  return new Date(dateStr).toLocaleDateString()
}

onMounted(fetchBoards)
</script>

<template>
  <div class="kanban-container w-full h-full md:px-1">
    <div class="header">
      <h2>公示中心</h2>
      <p class="text-gray-500">发现并订阅共享看板</p>
      <button class="btn btn-primary" @click="showCreateModal = true">
        创建看板
      </button>
    </div>

    <div v-if="loading" class="loading">
      <div class="loader"></div>
    </div>

    <div v-else-if="error" class="error">
      {{ error }}
    </div>

    <div v-else class="boards-grid">
      <div v-if="boards.length === 0" class="empty-state">
        <div class="empty-icon">📋</div>
        <p>暂无公开看板</p>
      </div>

      <div
        v-for="board in boards"
        :key="board.id"
        class="board-card"
      >
        <div class="board-header">
          <h3>{{ board.name }}</h3>
          <div class="board-actions">
            <button 
              class="action-btn edit-btn"
              title="编辑"
              @click="handleEditBoard(board)"
            >
              <Edit2 class="w-4 h-4" />
            </button>
            <button 
              class="action-btn delete-btn"
              title="删除"
              @click="handleDeleteBoard(board.id)"
            >
              <Trash2 class="w-4 h-4" />
            </button>
            <span class="badge public-badge">公开</span>
          </div>
        </div>
        <p class="board-description">{{ board.description || '暂无描述' }}</p>
        <div class="board-meta">
          <span>创建时间：{{ formatDate(board.created_at) }}</span>
        </div>
        <button class="btn btn-outline btn-block" @click="handleSubscribe(board.id)">
          订阅
        </button>
      </div>
    </div>

    <ConfirmDialog
      :visible="showDeleteConfirm"
      title="删除看板"
      message="确定要删除此看板吗？此操作无法撤销。"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
    />

    <div v-if="showCreateModal" class="modal-overlay" @click.self="showCreateModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3>创建新看板</h3>
          <button class="modal-close" @click="showCreateModal = false">×</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>看板名称</label>
            <input
              v-model="newBoardName"
              type="text"
              placeholder="请输入看板名称"
              class="form-control"
            />
          </div>
          <div class="form-group">
            <label>描述（可选）</label>
            <textarea
              v-model="newBoardDescription"
              placeholder="请输入描述"
              class="form-control"
              rows="3"
            ></textarea>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="showCreateModal = false">取消</button>
          <button class="btn btn-primary" @click="handleCreateBoard">创建</button>
        </div>
      </div>
    </div>

    <div v-if="showEditModal" class="modal-overlay" @click.self="showEditModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3>编辑看板</h3>
          <button class="modal-close" @click="showEditModal = false">×</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>看板名称</label>
            <input
              v-model="editBoardName"
              type="text"
              placeholder="请输入看板名称"
              class="form-control"
            />
          </div>
          <div class="form-group">
            <label>描述（可选）</label>
            <textarea
              v-model="editBoardDescription"
              placeholder="请输入描述"
              class="form-control"
              rows="3"
            ></textarea>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="showEditModal = false">取消</button>
          <button class="btn btn-primary" @click="handleSaveEdit">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.kanban-container {
  margin: 0 auto;
  padding: 20px;
}

.header {
  display: flex;
  align-items: center;
  gap: 20px;
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

.boards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.empty-state {
  grid-column: 1 / -1;
  padding: 60px 20px;
  text-align: center;
  color: #9ca3af;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.board-card {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.board-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.board-header h3 {
  font-size: 18px;
  font-weight: 600;
}

.board-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.action-btn {
  padding: 4px 8px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.edit-btn {
  background-color: #fef3c7;
  color: #d97706;
}

.edit-btn:hover {
  background-color: #fde68a;
}

.delete-btn {
  background-color: #fee2e2;
  color: #dc2626;
}

.delete-btn:hover {
  background-color: #fecaca;
}

.badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.public-badge {
  background-color: #dcfce7;
  color: #16a34a;
}

.board-description {
  color: #6b7280;
  margin-bottom: 12px;
}

.board-meta {
  font-size: 12px;
  color: #9ca3af;
  margin-bottom: 16px;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: background-color 0.2s;
}

.btn-block {
  width: 100%;
}

.btn-primary {
  background-color: #3b82f6;
  color: white;
}

.btn-primary:hover {
  background-color: #2563eb;
}

.btn-outline {
  background-color: transparent;
  border: 1px solid #3b82f6;
  color: #3b82f6;
}

.btn-outline:hover {
  background-color: #eff6ff;
}

.btn-secondary {
  background-color: #f3f4f6;
  color: #374151;
}

.btn-secondary:hover {
  background-color: #e5e7eb;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 8px;
  width: 90%;
  max-width: 480px;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.modal-header h3 {
  margin: 0;
}

.modal-close {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #9ca3af;
}

.modal-body {
  padding: 20px;
}

.modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 16px 20px;
  border-top: 1px solid #e5e7eb;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 4px;
}

.form-control {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  box-sizing: border-box;
}

.form-control:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}
</style>
