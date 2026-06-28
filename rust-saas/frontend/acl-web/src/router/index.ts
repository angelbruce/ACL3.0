import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores'
import { authService } from '@/api'

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/auth/Login.vue'),
    meta: { requiresAuth: false },
  },
  {
    path: '/register',
    name: 'Register',
    component: () => import('@/views/auth/Register.vue'),
    meta: { requiresAuth: false },
  },
  {
    path: '/',
    component: () => import('@/views/layout/AppLayout.vue'),
    meta: { requiresAuth: true },
    children: [
      {
        path: '',
        redirect: '/sessions',
      },
      {
        path: 'sessions',
        name: 'Sessions',
        component: () => import('@/views/sessions/SessionList.vue'),
      },
      {
        path: 'sessions/:id',
        name: 'SessionDetail',
        component: () => import('@/views/sessions/SessionDetail.vue'),
      },
      {
        path: 'agents',
        name: 'Agents',
        component: () => import('@/views/agents/AgentList.vue'),
      },
      {
        path: 'agents/new',
        name: 'NewAgent',
        component: () => import('@/views/agents/AgentForm.vue'),
      },
      {
        path: 'agents/:id/edit',
        name: 'EditAgent',
        component: () => import('@/views/agents/AgentForm.vue'),
      },
      {
        path: 'flows',
        name: 'Flows',
        component: () => import('@/views/flows/FlowList.vue'),
      },
      {
        path: 'flows/new',
        name: 'NewFlow',
        component: () => import('@/views/flows/FlowEditor.vue'),
      },
      {
        path: 'flows/:id/edit',
        name: 'EditFlow',
        component: () => import('@/views/flows/FlowEditor.vue'),
      },
      {
        path: 'flows/:id/run',
        name: 'RunFlow',
        component: () => import('@/views/flows/FlowRunner.vue'),
      },
      {
        path: 'models',
        name: 'Models',
        component: () => import('@/views/models/ModelList.vue'),
      },
      {
        path: 'tools',
        name: 'Tools',
        component: () => import('@/views/tools/ToolList.vue'),
      },
      {
        path: 'mcp-servers',
        name: 'McpServers',
        component: () => import('@/views/tools/McpServerList.vue'),
      },
      {
        path: 'admin/init',
        name: 'AdminInit',
        component: () => import('@/views/admin/SystemInit.vue'),
      },
      {
        path: 'admin/personnel',
        name: 'AdminPersonnel',
        component: () => import('@/views/admin/PersonnelList.vue'),
      },
      {
        path: 'admin/personnel/create',
        name: 'AdminPersonnelCreate',
        component: () => import('@/views/admin/PersonnelForm.vue'),
      },
      {
        path: 'admin/personnel/:id/edit',
        name: 'AdminPersonnelEdit',
        component: () => import('@/views/admin/PersonnelForm.vue'),
      },
      {
        path: 'admin/departments',
        name: 'AdminDepartments',
        component: () => import('@/views/admin/DepartmentList.vue'),
      },
      {
        path: 'admin/roles',
        name: 'AdminRoles',
        component: () => import('@/views/admin/RoleList.vue'),
      },
      {
        path: 'admin/menus',
        name: 'AdminMenus',
        component: () => import('@/views/admin/MenuList.vue'),
      },
      {
        path: 'admin/permissions',
        name: 'AdminPermissions',
        component: () => import('@/views/admin/PermissionList.vue'),
      },
      {
        path: 'projects',
        name: 'Projects',
        component: () => import('@/views/workspace/ProjectList.vue'),
      },
      {
        path: 'projects/:id',
        name: 'ProjectDetail',
        component: () => import('@/views/workspace/ProjectDetail.vue'),
      },
      {
        path: 'workspace',
        name: 'Workspace',
        component: () => import('@/views/workspace/WorkspaceFiles.vue'),
      },
      {
        path: 'kanban',
        name: 'Kanban',
        component: () => import('@/views/workspace/KanbanBoardList.vue'),
      },
      {
        path: 'subscriptions',
        name: 'Subscriptions',
        component: () => import('@/views/workspace/SubscribedBoards.vue'),
      },
      {
        path: 'vec',
        name: 'VecDashboard',
        component: () => import('@/views/vec/Dashboard.vue'),
      },
      {
        path: 'vec/search',
        name: 'VecSearch',
        component: () => import('@/views/vec/Search.vue'),
      },
      {
        path: 'vec/documents',
        name: 'VecDocuments',
        component: () => import('@/views/vec/Documents/List.vue'),
      },
      {
        path: 'vec/documents/:id',
        name: 'VecDocumentDetail',
        component: () => import('@/views/vec/Documents/Detail.vue'),
      },
      {
        path: 'vec/graph',
        name: 'VecKnowledgeGraph',
        component: () => import('@/views/vec/KnowledgeGraph.vue'),
      },
      {
        path: 'vec/distillation',
        name: 'VecDistillation',
        component: () => import('@/views/vec/Distillation.vue'),
      },
      {
        path: 'vec/taxonomy',
        name: 'VecTaxonomy',
        component: () => import('@/views/vec/Taxonomy.vue'),
      },
      {
        path: 'vec/boundary',
        name: 'VecBoundary',
        component: () => import('@/views/vec/Boundary.vue'),
      },
      {
        path: 'vec/analytics',
        name: 'VecAnalytics',
        component: () => import('@/views/vec/Analytics.vue'),
      },
      {
        path: 'vec/version',
        name: 'VecVersion',
        component: () => import('@/views/vec/Version.vue'),
      },
      {
        path: 'vec/tasks',
        name: 'VecTasks',
        component: () => import('@/views/vec/Task.vue'),
      },
      {
        path: 'vec/import-export',
        name: 'VecImportExport',
        component: () => import('@/views/vec/ImportExport.vue'),
      },
    ],
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach(async (to, _from, next) => {
  // Temporarily bypass auth for VEC pages preview
  if (to.path.startsWith('/vec')) {
    next()
    return
  }

  const authStore = useAuthStore()

  if (to.meta.requiresAuth === false) {
    if (authStore.isAuthenticated) {
      next('/sessions')
    } else {
      next()
    }
  } else {
    if (!authStore.isAuthenticated) {
      next('/login')
    } else {
      const isValid = await authStore.checkAuth()
      if (!isValid) {
        next('/login')
      } else {
        if (!authStore.user && authStore.accessToken) {
          try {
            const tokenData = JSON.parse(atob(authStore.accessToken.split('.')[1]))
            const userInfo = await authService.getUser(tokenData.user_id)
            authStore.user = userInfo
          } catch {
            console.error('Failed to load user info in router guard')
          }
        }
        next()
      }
    }
  }
})

export default router
