<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Plus, User, Trash2, Edit2, Loader2, Mail, Phone, Building2, Shield, Calendar } from 'lucide-vue-next'
import { useAdminStore } from '@/stores'
import type { Personnel } from '@/types'

const router = useRouter()
const adminStore = useAdminStore()

const deletingId = ref<number | null>(null)
const showDetail = ref(false)
const selectedPersonnel = ref<Personnel | null>(null)

onMounted(async () => {
  await adminStore.loadPersonnel()
})

const deletePersonnel = async (id: number, event: Event) => {
  event.stopPropagation()
  if (confirm('确定要删除这个人吗?')) {
    deletingId.value = id
    try {
      await adminStore.deletePersonnel(id)
    } finally {
      deletingId.value = null
    }
  }
}

const openDetail = (personnel: Personnel) => {
  selectedPersonnel.value = personnel
  showDetail.value = true
}

const closeDetail = () => {
  showDetail.value = false
  selectedPersonnel.value = null
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return '从未登录'
  return new Date(dateStr).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <div>
        <h1 class="page-title">人员管理</h1>
        <p class="page-subtitle">personnel management</p>
      </div>
      <button @click="router.push('/admin/personnel/create')" class="btn btn-primary">
        <Plus class="w-4 h-4" />
        添加人员
      </button>
    </div>

    <div v-if="adminStore.loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else-if="adminStore.personnel.length === 0" class="card p-12 text-center">
      <div class="w-16 h-16 mx-auto rounded-2xl bg-primary-50 border border-primary-100 flex items-center justify-center mb-4">
        <User class="w-8 h-8 text-primary-400" />
      </div>
      <h3 class="text-lg font-medium text-surface-800 mb-2">暂无人员</h3>
      <p class="text-surface-400 mb-6">添加人员来开始管理</p>
      <button @click="router.push('/admin/personnel/create')" class="btn btn-primary">
        <Plus class="w-4 h-4" /> 添加人员
      </button>
    </div>

    <div v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <div
        v-for="personnel in adminStore.personnel"
        :key="personnel.id"
        @click="openDetail(personnel)"
        class="card p-5 cursor-pointer group"
      >
        <div class="flex items-start justify-between mb-3">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-primary-100 to-accent-100 flex items-center justify-center">
              <User class="w-6 h-6 text-primary-500" />
            </div>
            <div>
              <p class="text-xs text-surface-400 mb-0.5">姓名</p>
              <h3 class="font-semibold text-base text-surface-800 truncate" :title="personnel.name">{{ personnel.name }}</h3>
              <div class="flex items-center gap-1 text-xs text-surface-400 mt-0.5">
                <Calendar class="w-3 h-3" />
                {{ formatDate(personnel.last_login_date) }}
              </div>
            </div>
          </div>
          <div class="flex items-center gap-1">
            <button @click.stop="router.push(`/admin/personnel/${personnel.id}/edit`)" class="p-2 text-surface-400 hover:text-primary-500 hover:bg-surface-100 rounded-lg transition-colors">
              <Edit2 class="w-4 h-4" />
            </button>
            <button @click.stop="deletePersonnel(personnel.id, $event)" :disabled="deletingId === personnel.id" class="p-2 text-surface-400 hover:text-red-500 hover:bg-surface-100 rounded-lg transition-colors">
              <Loader2 v-if="deletingId === personnel.id" class="w-4 h-4 animate-spin" />
              <Trash2 v-else class="w-4 h-4" />
            </button>
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center gap-2 text-sm">
            <Mail class="w-4 h-4 text-surface-400" />
            <span class="text-surface-600 truncate">{{ personnel.email || '无邮箱' }}</span>
          </div>
          <div class="flex items-center gap-2 text-sm">
            <Phone class="w-4 h-4 text-surface-400" />
            <span class="text-surface-600">{{ personnel.phone || '无电话' }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Detail dialog -->
    <Teleport to="body">
      <div v-if="showDetail && selectedPersonnel" class="fixed inset-0 bg-black/30 flex items-center justify-center z-50 p-4" @click.self="closeDetail">
        <div class="card p-6 w-full max-w-lg animate-fade-in max-h-[80vh] overflow-y-auto">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-lg font-semibold text-surface-800">人员详情</h2>
            <button @click="closeDetail" class="p-2 hover:bg-surface-100 rounded-lg">
              <span class="text-surface-400 text-xl">&times;</span>
            </button>
          </div>
          
          <div class="flex items-center gap-4 mb-6">
            <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-primary-100 to-accent-100 flex items-center justify-center">
              <User class="w-8 h-8 text-primary-500" />
            </div>
            <div>
              <h3 class="text-xl font-semibold text-surface-800">{{ selectedPersonnel.name }}</h3>
              <p class="text-surface-400 text-sm">{{ selectedPersonnel.email }}</p>
            </div>
          </div>

          <div class="space-y-4">
            <div class="flex items-center justify-between p-3 bg-surface-50 rounded-lg">
              <div class="flex items-center gap-2">
                <User class="w-4 h-4 text-surface-400" />
                <span class="text-surface-600">性别</span>
              </div>
              <span class="font-medium text-surface-800">{{ selectedPersonnel.gender === 'male' ? '男' : selectedPersonnel.gender === 'female' ? '女' : '未知' }}</span>
            </div>
            
            <div class="flex items-center justify-between p-3 bg-surface-50 rounded-lg">
              <div class="flex items-center gap-2">
                <Mail class="w-4 h-4 text-surface-400" />
                <span class="text-surface-600">邮箱</span>
              </div>
              <span class="font-medium text-surface-800">{{ selectedPersonnel.email || '-' }}</span>
            </div>

            <div class="flex items-center justify-between p-3 bg-surface-50 rounded-lg">
              <div class="flex items-center gap-2">
                <Phone class="w-4 h-4 text-surface-400" />
                <span class="text-surface-600">手机号</span>
              </div>
              <span class="font-medium text-surface-800">{{ selectedPersonnel.phone || '-' }}</span>
            </div>

            <div class="flex items-center justify-between p-3 bg-surface-50 rounded-lg">
              <div class="flex items-center gap-2">
                <span class="w-4 h-4 text-surface-400">💬</span>
                <span class="text-surface-600">微信</span>
              </div>
              <span class="font-medium text-surface-800">{{ selectedPersonnel.wechat || '-' }}</span>
            </div>

            <div class="flex items-center justify-between p-3 bg-surface-50 rounded-lg">
              <div class="flex items-center gap-2">
                <Calendar class="w-4 h-4 text-surface-400" />
                <span class="text-surface-600">最后登录</span>
              </div>
              <span class="font-medium text-surface-800">{{ formatDate(selectedPersonnel.last_login_date) }}</span>
            </div>
          </div>

          <div class="mt-6 pt-4 border-t border-surface-100">
            <div class="flex items-center gap-2 mb-3">
              <Building2 class="w-4 h-4 text-surface-400" />
              <span class="font-medium text-surface-700">所属部门</span>
            </div>
            <div class="flex flex-wrap gap-2">
              <span class="tag tag-cyan text-xs">暂无部门</span>
            </div>
          </div>

          <div class="mt-4 pt-4 border-t border-surface-100">
            <div class="flex items-center gap-2 mb-3">
              <Shield class="w-4 h-4 text-surface-400" />
              <span class="font-medium text-surface-700">角色</span>
            </div>
            <div class="flex flex-wrap gap-2">
              <span class="tag tag-blue text-xs">暂无角色</span>
            </div>
          </div>

          <div class="flex gap-3 mt-6">
            <button @click="closeDetail" class="btn btn-outline flex-1 justify-center">关闭</button>
            <button @click="router.push(`/admin/personnel/${selectedPersonnel.id}/edit`)" class="btn btn-primary flex-1 justify-center">编辑</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
