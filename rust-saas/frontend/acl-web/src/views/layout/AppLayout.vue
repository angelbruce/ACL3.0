<script setup lang="ts">
import { ref } from 'vue'
import { RouterView, useRouter, useRoute } from 'vue-router'
import {
  MessageSquare, Bot, Workflow, Settings, Wrench, LogOut, Menu, X, Server, Cpu,
  Users, Building, Shield, Key, FolderTree, FolderOpen, ClipboardList, Bell,
  BookOpen, Search, FileText, GitFork, FlaskConical, Tags, ShieldCheck, BarChart3,
  History, ListTodo, ArrowLeftRight,
} from 'lucide-vue-next'
import { useAuthStore } from '@/stores'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const sidebarOpen = ref(true)
const sidebarCollapsed = ref(false)
const adminExpanded = ref(false)
const configExpanded = ref(false)
const vecExpanded = ref(false)
const aiExpanded = ref(false)

const vecNavItems = [
  { path: '/vec', name: '仪表盘', icon: BookOpen },
  { path: '/vec/search', name: '搜索', icon: Search },
  { path: '/vec/documents', name: '文档管理', icon: FileText },
  { path: '/vec/graph', name: '知识图谱', icon: GitFork },
  { path: '/vec/distillation', name: '知识蒸馏', icon: FlaskConical },
  { path: '/vec/taxonomy', name: '知识分类', icon: Tags },
  { path: '/vec/boundary', name: '知识边界', icon: ShieldCheck },
  { path: '/vec/analytics', name: '数据分析', icon: BarChart3 },
  { path: '/vec/version', name: '版本管理', icon: History },
  { path: '/vec/tasks', name: '任务管理', icon: ListTodo },
  { path: '/vec/import-export', name: '导入导出', icon: ArrowLeftRight },
]

const navItems = [
  { path: '/sessions', name: '探索', icon: MessageSquare },
  { path: '/workspace', name: '工作区', icon: FolderOpen },
  { path: '/flows', name: '自动化', icon: Workflow },
  { path: '/agents', name: '智能仓库', icon: Bot },
  { path: '/kanban', name: '公示中心', icon: ClipboardList },
  { path: '/subscriptions', name: '我的订阅', icon: Bell },
]

const adminNavItems = [
  { path: '/admin/init', name: '系统初始化', icon: Settings },
  { path: '/admin/personnel', name: '人员管理', icon: Users },
  { path: '/admin/departments', name: '部门管理', icon: Building },
  { path: '/admin/roles', name: '角色管理', icon: Shield },
  { path: '/admin/menus', name: '菜单管理', icon: FolderTree },
  { path: '/admin/permissions', name: '权限管理', icon: Key },
]

const configNavItems = [
  { path: '/models', name: '模型管理', icon: Settings },
  { path: '/mcp-servers', name: 'MCP注册', icon: Server },
  { path: '/tools', name: '工具列表', icon: Wrench },
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
        <nav class="flex-1 px-2 space-y-0.5 overflow-auto">
          <!-- <router-link
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
        </router-link> -->

        <!-- AI  Base Section -->
        <div v-if="!sidebarCollapsed" class="border-surface-200">
          <button
            @click="aiExpanded = !aiExpanded"
            class="w-full flex items-center gap-3 px-3 py-2.5 text-surface-500 hover:text-surface-700 hover:bg-surface-50 transition-all duration-150"
          >
            <BookOpen class="w-5 h-5 flex-shrink-0" />
            <span class="text-sm flex-1 text-left">智能中心</span>
            <span :class="['transform transition-transform', aiExpanded ? 'rotate-180' : '']">▼</span>
          </button>
          <div v-show="aiExpanded" class="mt-1 space-y-0.5 pl-4  border-surface-100  border-t">
            <router-link
              v-for="item in navItems"
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

        <!-- VEC Knowledge Base Section -->
        <div v-if="!sidebarCollapsed" class="mt-4 border-t border-surface-200">
          <button
            @click="vecExpanded = !vecExpanded"
            class="w-full flex items-center gap-3 px-3 py-2.5 text-surface-500 hover:text-surface-700 hover:bg-surface-50 rounded-lg transition-all duration-150"
          >
            <BookOpen class="w-5 h-5 flex-shrink-0" />
            <span class="text-sm flex-1 text-left">知识库</span>
            <span :class="['transform transition-transform', vecExpanded ? 'rotate-180' : '']">▼</span>
          </button>
          <div v-show="vecExpanded" class="mt-1 space-y-0.5 pl-4  border-surface-100  border-t">
            <router-link
              v-for="item in vecNavItems"
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

          
            <!-- Config Section -->
          <div v-if="!sidebarCollapsed" class="mt-4 border-t border-surface-200">
            <button
              @click="configExpanded = !configExpanded"
              class="w-full flex items-center gap-3 px-3 py-2.5 text-surface-500 hover:text-surface-700 hover:bg-surface-50 rounded-lg transition-all duration-150"
            >
              <Settings class="w-5 h-5 flex-shrink-0" />
              <span class="text-sm flex-1 text-left">配置管理</span>
              <span :class="['transform transition-transform', configExpanded ? 'rotate-180' : '']">▼</span>
            </button>
            <div v-show="configExpanded" class="mt-1 space-y-0.5 pl-4  border-surface-100  border-t">
              <router-link
                v-for="item in configNavItems"
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

          <!-- Admin Section -->
          <div v-if="!sidebarCollapsed" class="mt-4 border-t border-surface-200">
            <button
              @click="adminExpanded = !adminExpanded"
              class="w-full flex items-center gap-3 px-3 py-2.5 text-surface-500 hover:text-surface-700 hover:bg-surface-50 rounded-lg transition-all duration-150"
            >
              <Settings class="w-5 h-5 flex-shrink-0" />
              <span class="text-sm flex-1 text-left">系统管理</span>
              <span :class="['transform transition-transform', adminExpanded ? 'rotate-180' : '']">▼</span>
            </button>
            <div v-show="adminExpanded" class="mt-1 space-y-0.5 pl-4  border-surface-100  border-t">
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