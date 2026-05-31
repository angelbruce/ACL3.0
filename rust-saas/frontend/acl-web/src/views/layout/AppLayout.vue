<script setup lang="ts">
import { ref } from 'vue'
import { RouterView, useRouter, useRoute } from 'vue-router'
import {
  MessageSquare, Bot, Workflow, Settings, Wrench, LogOut, Menu, X, Server, Cpu,
  Users, Building, Shield, Key, FolderTree, FolderOpen, ClipboardList, Bell,
} from 'lucide-vue-next'
import { useAuthStore } from '@/stores'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const sidebarOpen = ref(true)
const sidebarCollapsed = ref(false)
const adminExpanded = ref(false)

const navItems = [
  { path: '/sessions', name: '会话', icon: MessageSquare },
  { path: '/agents', name: 'Agent', icon: Bot },
  { path: '/flows', name: '工作流', icon: Workflow },
  { path: '/models', name: '模型', icon: Settings },
  { path: '/tools', name: '工具', icon: Wrench },
  { path: '/mcp-servers', name: 'MCP服务器', icon: Server },
  { path: '/workspace', name: '工作区', icon: FolderOpen },
  { path: '/kanban', name: '看板', icon: ClipboardList },
  { path: '/subscriptions', name: '订阅', icon: Bell },
]

const adminNavItems = [
  { path: '/admin/init', name: '系统初始化', icon: Settings },
  { path: '/admin/personnel', name: '人员管理', icon: Users },
  { path: '/admin/departments', name: '部门管理', icon: Building },
  { path: '/admin/roles', name: '角色管理', icon: Shield },
  { path: '/admin/menus', name: '菜单管理', icon: FolderTree },
  { path: '/admin/permissions', name: '权限管理', icon: Key },
]

const isActive = (path: string) => route.path.startsWith(path)

const handleLogout = async () => {
  await authStore.logout()
  router.push('/login')
}

const toggleSidebar = () => {
  if (window.innerWidth < 768) {
    sidebarOpen.value = !sidebarOpen.value
  } else {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }
}
</script>

<template>
  <div class="flex h-screen bg-surface-50 text-surface-800 overflow-hidden">
    <!-- Mobile overlay -->
    <div
      v-if="sidebarOpen && !sidebarCollapsed"
      class="fixed inset-0 bg-black/20 backdrop-blur-sm z-40 md:hidden"
      @click="sidebarOpen = false"
    />

    <!-- Sidebar -->
    <aside
      :class="[
        'fixed md:relative z-50 h-full bg-white border-r border-surface-200 transition-all duration-300 flex-shrink-0',
        sidebarOpen ? 'w-64' : 'w-0 md:w-16 -translate-x-full md:translate-x-0 overflow-hidden',
        sidebarCollapsed ? 'md:w-16' : 'md:w-64',
      ]"
    >
      <div class="flex flex-col h-full">
        <!-- Logo -->
        <div class="flex items-center h-16 px-4 border-b border-surface-100">
          <div class="flex items-center gap-3">
            <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-primary-500 to-accent-500 flex items-center justify-center font-bold text-sm shadow-[0_2px_8px_rgba(12,142,233,0.2)]">
              <Cpu class="w-5 h-5 text-white" />
            </div>
            <span v-if="!sidebarCollapsed" class="font-semibold text-lg gradient-text tracking-wide">ACL</span>
          </div>
          <button class="ml-auto md:hidden text-surface-400 hover:text-surface-700" @click="sidebarOpen = false">
            <X class="w-5 h-5" />
          </button>
        </div>

        <!-- Navigation -->
        <nav class="flex-1 py-4 px-2 space-y-0.5">
          <router-link
            v-for="item in navItems"
            :key="item.path"
            :to="item.path"
            :class="[
              'flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-150',
              isActive(item.path)
                ? 'bg-primary-50 text-primary-600 font-medium'
                : 'text-surface-500 hover:text-surface-700 hover:bg-surface-50',
              sidebarCollapsed ? 'justify-center' : '',
            ]"
            :title="sidebarCollapsed ? item.name : undefined"
          >
            <component :is="item.icon" class="w-5 h-5 flex-shrink-0" />
            <span v-if="!sidebarCollapsed" class="text-sm">{{ item.name }}</span>
          </router-link>

          <!-- Admin Section -->
          <div v-if="!sidebarCollapsed" class="pt-4 mt-4 border-t border-surface-200">
            <button
              @click="adminExpanded = !adminExpanded"
              class="w-full flex items-center gap-3 px-3 py-2.5 text-surface-500 hover:text-surface-700 hover:bg-surface-50 rounded-lg transition-all duration-150"
            >
              <Settings class="w-5 h-5 flex-shrink-0" />
              <span class="text-sm flex-1 text-left">系统管理</span>
              <span :class="['transform transition-transform', adminExpanded ? 'rotate-180' : '']">▼</span>
            </button>
            <div v-show="adminExpanded" class="mt-1 space-y-0.5 pl-4">
              <router-link
                v-for="item in adminNavItems"
                :key="item.path"
                :to="item.path"
                :class="[
                  'flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-150 text-sm',
                  isActive(item.path)
                    ? 'bg-primary-50 text-primary-600 font-medium'
                    : 'text-surface-500 hover:text-surface-700 hover:bg-surface-50',
                ]"
              >
                <component :is="item.icon" class="w-4 h-4 flex-shrink-0" />
                <span>{{ item.name }}</span>
              </router-link>
            </div>
          </div>
        </nav>

        <!-- User section -->
        <div class="p-4 border-t border-surface-100">
          <div :class="['flex items-center', sidebarCollapsed ? 'justify-center' : 'gap-3']">
            <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-primary-100 to-accent-100 border border-primary-200 flex items-center justify-center text-sm font-medium text-primary-600">
              {{ authStore.user?.email?.[0]?.toUpperCase() || 'U' }}
            </div>
            <div v-if="!sidebarCollapsed" class="flex-1 min-w-0">
              <p class="text-xs font-medium truncate text-surface-600">{{ authStore.user?.email || 'User' }}</p>
            </div>
            <button
              :class="['text-surface-300 hover:text-red-500 transition-colors p-1.5 rounded-lg hover:bg-red-50', sidebarCollapsed ? '' : 'ml-auto']"
              :title="sidebarCollapsed ? '登出' : undefined"
              @click="handleLogout"
            >
              <LogOut class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </aside>

    <!-- Main content -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- Top bar -->
      <header class="h-14 flex items-center px-4 bg-white border-b border-surface-200 flex-shrink-0">
        <button class="p-2 rounded-lg text-surface-400 hover:text-surface-700 hover:bg-surface-50 transition-colors md:hidden" @click="sidebarOpen = true">
          <Menu class="w-5 h-5" />
        </button>
        <button class="hidden md:block p-2 rounded-lg text-surface-400 hover:text-surface-700 hover:bg-surface-50 transition-colors mr-2" @click="toggleSidebar">
          <Menu class="w-5 h-5" />
        </button>
        <div class="flex-1" />
      </header>

      <!-- Page content -->
      <main class="flex-1 overflow-auto">
        <RouterView />
      </main>
    </div>
  </div>
</template>