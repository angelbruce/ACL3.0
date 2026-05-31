<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { ArrowLeft, Save, Loader2, User as UserIcon, Mail, Phone, MessageCircle } from 'lucide-vue-next';
import { useAdminStore, useAuthStore } from '@/stores';
import type { User } from '@/types';

const router = useRouter();
const route = useRoute();
const adminStore = useAdminStore();
const authStore = useAuthStore();

const isEdit = !!route.params.id;
const loading = ref(false);
const saving = ref(false);
const users = ref<User[]>([]);

const formData = ref({
  name: '',
  user_id: 0,
  gender: '',
  email: '',
  wechat: '',
  phone: '',
});

onMounted(async () => {
  await loadUsers();
  if (isEdit) {
    loading.value = true;
    try {
      const personnelId = Number(route.params.id);
      await adminStore.loadPersonnel();
      const personnel = adminStore.personnel.find(p => p.id === personnelId);
      if (personnel) {
        formData.value = {
          name: personnel.name,
          user_id: personnel.user_id,
          gender: personnel.gender || '',
          email: personnel.email || '',
          wechat: personnel.wechat || '',
          phone: personnel.phone || '',
        };
      }
    } catch (error) {
      console.error('Failed to load personnel:', error);
    } finally {
      loading.value = false;
    }
  }
});

const loadUsers = async () => {
    authStore.fetchUsers().then(() => {
      users.value = authStore.users;
    });
};

const handleSubmit = async () => {
  if (!formData.value.name.trim()) {
    alert('请输入姓名');
    return;
  }
  saving.value = true;
  try {
    const request = {
      name: formData.value.name,
      user_id: formData.value.user_id || undefined,
      gender: formData.value.gender || undefined,
      email: formData.value.email || undefined,
      wechat: formData.value.wechat || undefined,
      phone: formData.value.phone || undefined,
    };
    if (isEdit) {
      const personnelId = Number(route.params.id);
      await adminStore.updatePersonnel(personnelId, request);
    } else {
      await adminStore.createPersonnel(request);
    }
    router.push('/admin/personnel');
  } catch (error) {
    console.error('Failed to save personnel:', error);
  } finally {
    saving.value = false;
  }
};
</script>

<template>
  <div class="p-6">
    <div class="page-header">
      <button @click="router.push('/admin/personnel')" class="btn btn-outline mr-4">
        <ArrowLeft class="w-4 h-4 mr-2" />
        返回
      </button>
      <div>
        <h1 class="page-title">{{ isEdit ? '编辑人员' : '添加人员' }}</h1>
        <p class="page-subtitle">personnel {{ isEdit ? 'edit' : 'create' }}</p>
      </div>
      <button @click="handleSubmit" :disabled="saving" class="btn btn-primary">
        <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />
        <Save v-else class="w-4 h-4" />
        {{ saving ? '保存中...' : '保存' }}
      </button>
    </div>

    <div v-if="loading" class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 animate-spin text-primary-500" />
    </div>

    <div v-else class="flex justify-center">
      <div class="card p-8 w-full max-w-xl">
        <form @submit.prevent="handleSubmit" class="space-y-6">
          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">关联用户</label>
            <select v-model="formData.user_id" class="input-base w-full">
              <option :value="0">未关联用户</option>
              <option v-for="user in users" :key="user.id" :value="user.id">
                {{ user.email }}
              </option>
            </select>
            <p class="text-xs text-surface-400 mt-1">选择一个已注册的用户账号进行关联</p>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">姓名 <span class="text-red-500">*</span></label>
            <div class="relative">
              <div class="absolute left-4 top-1/2 -translate-y-1/2 pointer-events-none">
                <UserIcon class="w-5 h-5 text-surface-400" />
              </div>
              <input v-model="formData.name" type="text" placeholder="输入姓名" class="input-base w-full pl-12" />
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">性别</label>
            <select v-model="formData.gender" class="input-base w-full">
              <option value="">请选择</option>
              <option value="male">男</option>
              <option value="female">女</option>
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">邮箱</label>
            <div class="relative">
              <div class="absolute left-4 top-1/2 -translate-y-1/2 pointer-events-none">
                <Mail class="w-5 h-5 text-surface-400" />
              </div>
              <input v-model="formData.email" type="email" placeholder="输入邮箱" class="input-base w-full pl-12" />
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">手机号</label>
            <div class="relative">
              <div class="absolute left-4 top-1/2 -translate-y-1/2 pointer-events-none">
                <Phone class="w-5 h-5 text-surface-400" />
              </div>
              <input v-model="formData.phone" type="tel" placeholder="输入手机号" class="input-base w-full pl-12" />
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-surface-700 mb-2">微信</label>
            <div class="relative">
              <div class="absolute left-4 top-1/2 -translate-y-1/2 pointer-events-none">
                <MessageCircle class="w-5 h-5 text-surface-400" />
              </div>
              <input v-model="formData.wechat" type="text" placeholder="输入微信号" class="input-base w-full pl-12" />
            </div>
          </div>

          <div class="flex gap-3 pt-4">
            <button type="button" @click="router.push('/admin/personnel')" class="btn btn-outline flex-1 justify-center">取消</button>
            <button type="submit" :disabled="saving" class="btn btn-primary flex-1 justify-center">
              <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />
              <Save v-else class="w-4 h-4" />
              {{ saving ? '保存中...' : '保存' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
