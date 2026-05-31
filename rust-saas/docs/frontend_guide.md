# ACL权限管理系统 - 前端使用指南

## 概述

前端采用 Vue 3 + TypeScript + Vite + Tailwind CSS 构建，提供完整的权限管理界面。

## 技术栈

- **框架**: Vue 3 (Composition API)
- **语言**: TypeScript
- **构建工具**: Vite
- **UI框架**: Tailwind CSS
- **状态管理**: Pinia
- **路由**: Vue Router
- **HTTP客户端**: Axios
- **图标**: Lucide Vue

## 项目结构

```
acl-web/
├── src/
│   ├── api/
│   │   ├── admin.ts          # Admin API 客户端
│   │   ├── auth.ts           # 认证API
│   │   ├── client.ts         # HTTP客户端配置
│   │   └── index.ts          # API导出
│   ├── stores/
│   │   ├── admin.ts          # Admin状态管理
│   │   └── index.ts          # Store导出
│   ├── types/
│   │   ├── admin.ts          # Admin类型定义
│   │   └── index.ts          # 类型导出
│   ├── views/
│   │   └── admin/            # 管理页面
│   │       ├── SystemInit.vue     # 系统初始化
│   │       ├── PersonnelList.vue   # 人员管理
│   │       ├── DepartmentList.vue  # 部门管理
│   │       ├── RoleList.vue       # 角色管理
│   │       ├── MenuList.vue        # 菜单管理
│   │       └── PermissionList.vue  # 权限管理
│   └── router/
│       └── index.ts          # 路由配置
└── package.json
```

## 功能页面

### 1. 系统初始化页面

**路径**: `/admin/init`

**功能**:
- 按顺序初始化超级管理员角色
- 初始化默认菜单
- 初始化默认权限
- 分配所有权限给超级管理员

**使用流程**:
1. 点击"执行"按钮，按顺序执行4个初始化步骤
2. 所有步骤完成后，显示"系统初始化完成"提示
3. 点击"前往人员管理"开始使用

### 2. 人员管理页面

**路径**: `/admin/personnel`

**功能**:
- 查看所有人员列表
- 添加新人员
- 编辑人员信息
- 查看人员详情（包含部门、角色、权限）
- 分配部门
- 分配角色
- 设为超级管理员

**操作说明**:

#### 添加人员
1. 点击右上角"添加人员"按钮
2. 填写姓名、性别、邮箱、微信、手机号
3. 点击"保存"

#### 编辑人员
1. 在人员列表中点击"编辑"按钮
2. 修改信息
3. 点击"保存"

#### 查看详情
1. 点击人员列表中的"详情"按钮
2. 查看人员信息、所属部门、拥有角色、拥有权限
3. 进行分配部门、分配角色等操作

#### 设为超级管理员
1. 在详情页面点击"设为超级管理员"按钮
2. 确认操作
3. 人员将拥有所有权限

### 3. 部门管理页面

**路径**: `/admin/departments`

**功能**:
- 查看所有部门列表
- 添加部门（支持树形结构）
- 编辑部门信息
- 删除部门

**操作说明**:

#### 添加部门
1. 点击右上角"添加部门"按钮
2. 填写部门名称、选择上级部门、填写描述
3. 点击"保存"

#### 创建子部门
1. 在"上级部门"下拉框中选择父部门
2. 填写子部门信息
3. 点击"保存"

### 4. 角色管理页面

**路径**: `/admin/roles`

**功能**:
- 查看所有角色列表
- 添加角色
- 编辑角色信息
- 删除角色（非超级管理员角色）
- 管理角色权限

**操作说明**:

#### 添加角色
1. 点击右上角"添加角色"按钮
2. 填写角色名称、描述
3. 可选择"设为超级管理员"
4. 点击"保存"

#### 管理权限
1. 点击角色卡片中的"权限管理"按钮
2. 在权限管理弹窗中勾选需要的权限
3. 点击"保存"

### 5. 菜单管理页面

**路径**: `/admin/menus`

**功能**:
- 查看所有菜单列表
- 添加菜单（支持树形结构）
- 编辑菜单信息
- 删除菜单

**操作说明**:

#### 添加菜单
1. 点击右上角"添加菜单"按钮
2. 填写菜单名称、路径、上级菜单、图标、排序
3. 点击"保存"

### 6. 权限管理页面

**路径**: `/admin/permissions`

**功能**:
- 按菜单分组查看所有权限
- 添加权限
- 删除权限

**操作说明**:

#### 添加权限
1. 点击右上角"添加权限"按钮
2. 选择所属菜单
3. 填写权限名称和描述
4. 点击"保存"

#### 在菜单下快速添加
1. 在菜单卡片下方点击"+"按钮
2. 系统会自动选择该菜单
3. 填写权限信息
4. 点击"保存"

## 界面预览

### 侧边栏导航

左侧边栏包含：
- **主导航**: 会话、Agent、工作流、模型、工具、MCP服务器
- **系统管理**（可折叠）:
  - 系统初始化
  - 人员管理
  - 部门管理
  - 角色管理
  - 菜单管理
  - 权限管理

### 页面布局

所有管理页面采用统一布局：
- **标题栏**: 页面标题 + 主要操作按钮
- **数据表格/卡片**: 显示数据列表
- **模态框**: 用于创建、编辑、详情查看等操作

### 响应式设计

- 桌面端: 完整侧边栏
- 移动端: 可折叠侧边栏，汉堡菜单

## 状态管理

使用 Pinia 进行状态管理，主要状态：

```typescript
interface AdminState {
  personnel: Personnel[]           // 人员列表
  departments: Department[]          // 部门列表
  menus: Menu[]                    // 菜单列表
  permissions: Permission[]         // 权限列表
  roles: Role[]                    // 角色列表
  currentPersonnel: PersonnelWithDetails | null  // 当前查看的人员详情
  loading: boolean                 // 加载状态
  error: string | null             // 错误信息
}
```

## API调用

所有API调用通过 `adminService` 进行：

```typescript
import { adminService } from '@/api'

// 获取人员列表
const personnel = await adminService.getPersonnelList()

// 创建人员
const newPerson = await adminService.createPersonnel(data)

// 更新人员
const updated = await adminService.updatePersonnel(id, data)

// 删除部门
await adminService.deleteDepartment(id)

// 分配角色
await adminService.assignRoles({ personnel_id: 1, role_ids: [1, 2] })
```

## 错误处理

所有API调用包含错误处理：

```typescript
try {
  await adminStore.loadPersonnel()
} catch (error) {
  console.error('Failed to load personnel:', error)
  // 错误信息会自动存储在 adminStore.error 中
}
```

## 组件说明

### 通用组件

- **Modal**: 用于创建、编辑、详情等操作
- **Table**: 数据显示表格
- **Form**: 表单输入
- **Button**: 操作按钮
- **Badge**: 状态标签

### 页面组件

每个管理页面都是独立组件，包含：
- 数据加载
- 数据展示
- 创建/编辑表单
- 删除确认
- 错误提示

## 开发指南

### 添加新的管理页面

1. 在 `src/views/admin/` 下创建新页面组件
2. 在 `src/router/index.ts` 添加路由
3. 在 `src/api/admin.ts` 添加API方法（如需要）
4. 在 `src/stores/admin.ts` 添加状态管理方法（如需要）
5. 在 `AppLayout.vue` 侧边栏添加导航项

### 示例：添加新管理页面

```vue
<!-- src/views/admin/NewFeatureList.vue -->
<template>
  <div class="min-h-screen bg-gray-50 p-8">
    <div class="max-w-7xl mx-auto">
      <div class="bg-white rounded-lg shadow-md p-6">
        <!-- 页面内容 -->
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useAdminStore } from '@/stores'

const adminStore = useAdminStore()

onMounted(async () => {
  // 加载数据
})
</script>
```

```typescript
// src/router/index.ts
{
  path: 'admin/new-feature',
  name: 'AdminNewFeature',
  component: () => import('@/views/admin/NewFeatureList.vue'),
}
```

## 样式指南

### Tailwind CSS 类名

- **布局**: `min-h-screen`, `flex`, `grid`
- **间距**: `p-4`, `m-4`, `space-y-4`
- **颜色**: `bg-white`, `text-gray-800`, `border-gray-300`
- **响应式**: `md:w-64`, `lg:grid-cols-3`

### 自定义样式

在 `src/style.css` 中定义全局样式。

## 性能优化

1. **路由懒加载**: 使用 `() => import()` 动态导入页面组件
2. **状态缓存**: 使用 Pinia 管理状态，避免重复请求
3. **虚拟滚动**: 大数据列表使用虚拟滚动（可选）

## 浏览器兼容性

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## 部署说明

### 开发环境

```bash
cd acl-web
npm install
npm run dev
```

### 生产构建

```bash
npm run build
```

构建产物在 `dist/` 目录。

### 环境变量

创建 `.env` 文件：

```env
VITE_API_BASE_URL=http://localhost:3000
```

## 常见问题

### 1. 页面空白

检查：
- API服务是否启动
- 浏览器控制台是否有错误
- 网络请求是否正常

### 2. 数据不更新

解决：
- 刷新页面
- 清除浏览器缓存
- 检查网络请求

### 3. 样式异常

解决：
- 清除浏览器缓存
- 重新安装依赖: `rm -rf node_modules && npm install`

## 相关文档

- [后端API文档](../docs/admin_system_guide.md)
- [项目总结](../docs/IMPLEMENTATION_SUMMARY.md)
- [快速参考](../docs/QUICK_REFERENCE.md)
