# Vec-SVC 前端开发计划

> 最后更新：2026-06-26
> 状态：规划阶段，等待执行

---

## 目录

1. [项目背景与目标](#1-项目背景与目标)
2. [技术选型](#2-技术选型)
3. [项目结构设计](#3-项目结构设计)
4. [里程碑规划](#4-里程碑规划)
5. [页面清单与 API 对接](#5-页面清单与-api-对接)
6. [状态管理与数据流](#6-状态管理与数据流)
7. [开发环境配置](#7-开发环境配置)
8. [部署方案](#8-部署方案)
9. [风险与应对](#9-风险与应对)

---

## 1. 项目背景与目标

### 1.1 背景

Vec-SVC 后端服务（M1-M5）已全部开发完成，包含 50+ API 接口、21 张数据库表、完整的 RAG 知识管理能力。但目前**前端界面零实现**，用户无法通过可视化界面使用系统功能。

### 1.2 目标

创建一个企业级、现代化的前端应用，让用户能够：
- 方便地上传、管理、搜索文档
- 可视化查看知识图谱
- 使用知识蒸馏、分类分级等高级功能
- 进行权限管理和知识边界设置
- 查看统计分析和操作日志

### 1.3 前端核心价值

| 价值点 | 说明 |
|--------|------|
| **直观操作** | 文档上传、搜索、管理全部可视化 |
| **知识可视化** | 知识图谱以图形式展示，实体关系一目了然 |
| **权限可控** | 可视化设置文档可见性和共享规则 |
| **智能搜索** | 搜索建议、自动补全、结果高亮 |
| **数据分析** | 统计看板展示文档访问量、热门搜索等 |

---

## 2. 技术选型

### 2.1 核心技术栈

| 技术 | 选型 | 版本 | 决策原因 |
|------|------|------|----------|
| **框架** | React | 18+ | 生态成熟，组件化开发，适合复杂企业应用 |
| **语言** | TypeScript | 5+ | 类型安全，提升开发效率和代码质量 |
| **UI 组件库** | Ant Design | 5+ | 企业级 UI 组件，风格统一，开箱即用 |
| **状态管理** | React Query | 5+ | 服务端状态管理，缓存、重试、分页一站式解决 |
| **路由** | React Router | 6+ | 声明式路由，支持嵌套路由和代码分割 |
| **构建工具** | Vite | 6+ | 极速开发服务器，高效构建 |
| **HTTP 客户端** | Axios | 1+ | 拦截器、请求取消、自动转换 JSON |
| **图表库** | ECharts | 5+ | 强大的图表能力，支持知识图谱可视化 |
| **代码规范** | ESLint + Prettier | - | 统一代码风格，提升团队协作效率 |

### 2.2 辅助工具

| 工具 | 用途 |
|------|------|
| `@ant-design/icons` | Ant Design 图标库 |
| `dayjs` | 日期时间处理 |
| `lodash` | 工具函数库 |
| `clsx` | CSS 类名组合 |
| `tailwindcss` | 原子化 CSS（可选） |

---

## 3. 项目结构设计

```
frontend/
├── public/                      # 静态资源
│   ├── index.html
│   └── favicon.ico
├── src/
│   ├── main.tsx                 # 入口文件
│   ├── App.tsx                 # 根组件
│   ├── index.css               # 全局样式
│   ├── types/                  # TypeScript 类型定义
│   │   ├── api.ts              # API 响应类型
│   │   ├── document.ts         # 文档相关类型
│   │   ├── graph.ts            # 知识图谱类型
│   │   └── common.ts           # 通用类型
│   ├── api/                    # API 请求封装
│   │   ├── client.ts           # Axios 实例配置
│   │   ├── document.ts         # 文档接口
│   │   ├── search.ts           # 搜索接口
│   │   ├── graph.ts            # 知识图谱接口
│   │   ├── taxonomy.ts         # 分类分级接口
│   │   ├── boundary.ts         # 知识边界接口
│   │   ├── distillation.ts     # 知识蒸馏接口
│   │   ├── verification.ts     # 校验接口
│   │   ├── version.ts          # 版本管理接口
│   │   ├── analytics.ts        # 统计分析接口
│   │   ├── import_export.ts    # 导入导出接口
│   │   └── task.ts             # 任务队列接口
│   ├── components/             # 通用组件
│   │   ├── Layout/             # 布局组件
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── index.tsx
│   │   ├── Search/             # 搜索组件
│   │   │   ├── SearchInput.tsx
│   │   │   └── SearchSuggestions.tsx
│   │   ├── Document/           # 文档组件
│   │   │   ├── DocumentCard.tsx
│   │   │   ├── DocumentList.tsx
│   │   │   └── UploadModal.tsx
│   │   ├── Graph/              # 图谱组件
│   │   │   ├── EntityNode.tsx
│   │   │   ├── RelationEdge.tsx
│   │   │   └── GraphCanvas.tsx
│   │   ├── Common/             # 通用组件
│   │   │   ├── StatusBadge.tsx
│   │   │   ├── EmptyState.tsx
│   │   │   └── LoadingSpinner.tsx
│   ├── pages/                  # 页面组件
│   │   ├── Dashboard/          # 首页仪表盘
│   │   │   └── index.tsx
│   │   ├── Search/             # 搜索页面
│   │   │   └── index.tsx
│   │   ├── Documents/          # 文档管理页面
│   │   │   ├── List.tsx
│   │   │   └── Detail.tsx
│   │   ├── KnowledgeGraph/     # 知识图谱页面
│   │   │   └── index.tsx
│   │   ├── Distillation/       # 知识蒸馏页面
│   │   │   └── index.tsx
│   │   ├── Taxonomy/           # 分类分级页面
│   │   │   └── index.tsx
│   │   ├── Boundary/           # 知识边界页面
│   │   │   └── index.tsx
│   │   ├── Analytics/          # 统计分析页面
│   │   │   └── index.tsx
│   │   ├── Version/            # 版本管理页面
│   │   │   └── index.tsx
│   │   ├── Task/               # 任务管理页面
│   │   │   └── index.tsx
│   │   └── ImportExport/       # 导入导出页面
│   │       └── index.tsx
│   ├── hooks/                  # 自定义 Hooks
│   │   ├── useDocument.ts      # 文档相关 hooks
│   │   ├── useSearch.ts        # 搜索相关 hooks
│   │   ├── useGraph.ts         # 图谱相关 hooks
│   │   └── useAuth.ts          # 认证相关 hooks
│   ├── stores/                 # 状态管理
│   │   └── appStore.ts         # 应用级状态
│   ├── utils/                  # 工具函数
│   │   ├── format.ts           # 格式化工具
│   │   ├── validation.ts       # 表单校验
│   │   └── constants.ts        # 常量定义
│   └── routes/                 # 路由配置
│       └── index.tsx           # 路由定义
├── .env                        # 环境变量
├── .env.development            # 开发环境变量
├── .env.production             # 生产环境变量
├── vite.config.ts              # Vite 配置
├── tsconfig.json               # TypeScript 配置
├── package.json                # 依赖配置
└── README.md                   # 项目说明
```

---

## 4. 里程碑规划

### 4.1 总览

| 里程碑 | 名称 | 任务数 | 预计工期 | 核心交付物 |
|--------|------|--------|----------|------------|
| **FM1** | 项目骨架 | 4 | 3 天 | 项目初始化、布局、路由、API 层 |
| **FM2** | 文档管理与搜索 | 4 | 4 天 | 文档列表、详情、搜索页面、上传功能 |
| **FM3** | 知识图谱与高级功能 | 4 | 5 天 | 图谱可视化、蒸馏、分类分级、边界设置 |
| **FM4** | 统计分析与管理 | 4 | 3 天 | 统计看板、版本管理、导入导出、任务管理 |
| **FM5** | 生产优化 | 4 | 3 天 | 响应式、性能优化、国际化、测试 |
| **总计** | - | 20 | **18 天** | 完整前端应用 |

### 4.2 FM1 - 项目骨架（第 1-3 天）

#### 任务列表

| 任务 | 描述 | 工期 |
|------|------|------|
| **FT-001** | 创建 React + TypeScript + Vite 项目 | 0.5 天 |
| **FT-002** | 安装依赖（Ant Design、React Router、React Query、Axios） | 0.5 天 |
| **FT-003** | 配置布局组件（Header + Sidebar）和路由系统 | 1 天 |
| **FT-004** | 封装 API 层和类型定义 | 1 天 |

#### 交付物

- 项目脚手架
- 基础布局（左侧导航 + 右侧内容）
- API 请求封装（统一拦截器、错误处理）
- 完整的 TypeScript 类型定义

#### 验收标准

- 项目能正常启动和构建
- 路由跳转正常
- API 请求能正确发送和处理响应
- 布局适配不同屏幕尺寸

### 4.3 FM2 - 文档管理与搜索（第 4-7 天）

#### 任务列表

| 任务 | 描述 | 工期 |
|------|------|------|
| **FT-005** | 文档列表页面（上传、删除、筛选、分页） | 1 天 |
| **FT-006** | 文档详情页面（查看内容、知识点、版本历史） | 1 天 |
| **FT-007** | 搜索页面（搜索框、结果列表、高亮、搜索建议） | 1.5 天 |
| **FT-008** | 文件上传组件（拖拽上传、进度显示、格式验证） | 0.5 天 |

#### 交付物

- 文档列表页面
- 文档详情页面
- 搜索页面（含自动补全）
- 文件上传功能

#### API 对接

| 接口 | 用途 |
|------|------|
| `GET /api/documents` | 文档列表 |
| `GET /api/documents/{id}` | 文档详情 |
| `POST /api/documents/text` | 添加文本文档 |
| `POST /api/documents/file` | 上传文件 |
| `DELETE /api/documents/{id}` | 删除文档 |
| `POST /api/search` | 搜索 |
| `GET /api/search/autocomplete` | 自动补全 |
| `GET /api/search/suggest` | 搜索建议 |

#### 验收标准

- 文档上传成功后能在列表中显示
- 搜索能返回相关结果并高亮匹配内容
- 搜索建议和自动补全功能正常
- 删除文档有确认提示

### 4.4 FM3 - 知识图谱与高级功能（第 8-12 天）

#### 任务列表

| 任务 | 描述 | 工期 |
|------|------|------|
| **FT-009** | 知识图谱可视化页面（实体节点、关系边、力导向布局） | 1.5 天 |
| **FT-010** | 知识蒸馏页面（触发蒸馏、查看知识点、预览效果） | 1 天 |
| **FT-011** | 分类分级页面（分类树管理、文档打标签、分级评分） | 1.5 天 |
| **FT-012** | 知识边界页面（可见性设置、共享管理、权限检查） | 1 天 |

#### 交付物

- 知识图谱可视化页面（ECharts 力导向图）
- 知识蒸馏页面
- 分类分级管理页面
- 知识边界设置页面

#### API 对接

| 接口 | 用途 |
|------|------|
| `POST /api/graph/extract` | 提取实体和关系 |
| `GET /api/graph/entities` | 搜索实体 |
| `GET /api/graph/entities/{id}` | 实体详情 |
| `GET /api/graph/entities/{id}/relations` | 实体关系 |
| `POST /api/documents/{id}/distill` | 触发蒸馏 |
| `GET /api/documents/{id}/knowledge-points` | 获取知识点 |
| `POST /api/categories` | 创建分类 |
| `GET /api/categories` | 分类列表 |
| `POST /api/documents/{id}/categories` | 文档打标签 |
| `POST /api/documents/{id}/visibility` | 设置可见性 |
| `POST /api/shares` | 创建共享 |

#### 验收标准

- 知识图谱能展示实体和关系，支持缩放和拖拽
- 知识蒸馏能触发并显示结果
- 分类树能正常管理（增删改查）
- 文档能设置不同层级的可见性

### 4.5 FM4 - 统计分析与管理（第 13-15 天）

#### 任务列表

| 任务 | 描述 | 工期 |
|------|------|------|
| **FT-013** | 统计分析页面（仪表盘、文档统计、搜索统计） | 1 天 |
| **FT-014** | 版本管理页面（版本列表、版本对比、版本回滚） | 1 天 |
| **FT-015** | 导入导出页面（批量导入、多种格式导出） | 0.5 天 |
| **FT-016** | 任务管理页面（任务列表、进度查看、取消任务） | 0.5 天 |

#### 交付物

- 统计分析仪表盘
- 版本管理页面
- 导入导出功能
- 任务管理页面

#### API 对接

| 接口 | 用途 |
|------|------|
| `GET /api/analytics/summary` | 统计概览 |
| `GET /api/analytics/document` | 单文档统计 |
| `GET /api/documents/{id}/versions` | 版本列表 |
| `GET /api/versions/{id}` | 版本详情 |
| `GET /api/versions/compare` | 版本对比 |
| `POST /api/documents/{id}/rollback` | 版本回滚 |
| `POST /api/import/documents` | 批量导入 |
| `GET /api/export/documents` | 导出文档 |
| `GET /api/tasks` | 任务列表 |
| `GET /api/tasks/{id}` | 任务详情 |
| `DELETE /api/tasks/{id}` | 取消任务 |

#### 验收标准

- 统计仪表盘能展示关键指标
- 版本对比能显示差异
- 导入导出功能正常
- 任务进度能实时更新

### 4.6 FM5 - 生产优化（第 16-18 天）

#### 任务列表

| 任务 | 描述 | 工期 |
|------|------|------|
| **FT-017** | 响应式设计（移动端适配、平板适配） | 1 天 |
| **FT-018** | 性能优化（懒加载、代码分割、缓存优化） | 1 天 |
| **FT-019** | 国际化支持（中英文切换） | 0.5 天 |
| **FT-020** | 端到端测试和 Bug 修复 | 0.5 天 |

#### 交付物

- 响应式布局
- 性能优化后的应用
- 国际化支持
- 完整的测试用例

#### 验收标准

- 在手机、平板、桌面端都能正常显示
- 页面加载时间 < 2 秒
- 中英文切换正常
- 核心功能测试通过

---

## 5. 页面清单与 API 对接

### 5.1 页面路由清单

| 路径 | 页面 | 组件 | 说明 |
|------|------|------|------|
| `/` | 仪表盘 | Dashboard | 统计概览、快捷入口 |
| `/search` | 搜索 | Search | 搜索框、结果列表 |
| `/documents` | 文档列表 | Documents/List | 文档管理 |
| `/documents/{id}` | 文档详情 | Documents/Detail | 查看文档内容 |
| `/graph` | 知识图谱 | KnowledgeGraph | 图谱可视化 |
| `/distillation` | 知识蒸馏 | Distillation | 蒸馏管理 |
| `/taxonomy` | 分类分级 | Taxonomy | 分类和评分管理 |
| `/boundary` | 知识边界 | Boundary | 权限和共享管理 |
| `/analytics` | 统计分析 | Analytics | 数据看板 |
| `/version` | 版本管理 | Version | 文档版本管理 |
| `/tasks` | 任务管理 | Task | 后台任务管理 |
| `/import-export` | 导入导出 | ImportExport | 数据迁移 |

### 5.2 核心 API 对接表

#### 搜索相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/search` | POST | 主搜索功能 |
| `/api/search/suggest` | GET | 搜索建议下拉 |
| `/api/search/autocomplete` | GET | 输入框自动补全 |
| `/api/projects/{id}/search` | GET | 项目内搜索 |

#### 文档相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/documents` | GET | 文档列表 |
| `/api/documents/{id}` | GET | 文档详情 |
| `/api/documents/text` | POST | 新建文本文档 |
| `/api/documents/file` | POST | 上传文件 |
| `/api/documents/{id}` | DELETE | 删除文档 |
| `/api/documents/{id}/reindex` | POST | 重新索引 |

#### 知识图谱相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/graph/extract` | POST | 提取实体关系 |
| `/api/graph/entities` | GET | 搜索实体 |
| `/api/graph/entities/{id}` | GET | 实体详情 |
| `/api/graph/entities/{id}/relations` | GET | 实体关系图 |

#### 知识蒸馏相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/documents/{id}/distill` | POST | 触发蒸馏 |
| `/api/documents/{id}/knowledge-points` | GET | 查看知识点 |

#### 分类分级相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/categories` | GET/POST | 分类管理 |
| `/api/categories/{id}` | GET/PUT/DELETE | 分类 CRUD |
| `/api/documents/{id}/categories` | GET/POST | 文档打标签 |
| `/api/levels` | GET/POST | 分级管理 |
| `/api/documents/{id}/levels` | GET/POST | 文档评分 |

#### 知识边界相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/documents/{id}/visibility` | POST | 设置可见性 |
| `/api/shares` | POST | 创建共享 |
| `/api/shares/{id}` | DELETE | 删除共享 |

#### 版本管理相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/documents/{id}/versions` | GET/POST | 版本列表/创建 |
| `/api/versions/{id}` | GET | 版本详情 |
| `/api/versions/compare` | GET | 版本对比 |
| `/api/documents/{id}/rollback` | POST | 版本回滚 |

#### 统计分析相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/analytics/summary` | GET | 仪表盘数据 |
| `/api/analytics/document` | GET | 单文档统计 |

#### 任务队列相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/tasks` | GET/POST | 任务列表/创建 |
| `/api/tasks/{id}` | GET/DELETE | 任务详情/取消 |

#### 导入导出相关

| API | 方法 | 前端使用场景 |
|-----|------|-------------|
| `/api/import/documents` | POST | 批量导入 |
| `/api/export/documents` | GET | 导出文档 |
| `/api/export/knowledge-graph` | GET | 导出图谱 |

---

## 6. 状态管理与数据流

### 6.1 状态分层

```
┌─────────────────────────────────────────────────────────┐
│                    UI 层 (Components)                    │
│   展示数据、响应用户操作、触发状态更新                       │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  Hooks 层 (Custom Hooks)                 │
│   useDocument, useSearch, useGraph 等                    │
│   封装业务逻辑、调用 API、管理组件状态                      │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                React Query (服务端状态)                   │
│   缓存、重试、分页、乐观更新、SWR                         │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   API 层 (Axios)                         │
│   请求封装、拦截器、错误处理、统一响应格式                   │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   后端服务 (Vec-SVC)                      │
│   50+ API 接口、21 张数据库表                              │
└─────────────────────────────────────────────────────────┘
```

### 6.2 数据流示例（文档搜索）

```
用户输入搜索词
   │
   ▼
SearchInput 组件 onChange
   │
   ▼
useSearch hook (React Query)
   │
   ▼
search API 调用 (/api/search)
   │
   ▼
后端返回搜索结果
   │
   ▼
React Query 缓存结果
   │
   ▼
SearchResult 组件渲染
   │
   ▼
用户点击结果项
   │
   ▼
跳转到文档详情页面 (/documents/{id})
   │
   ▼
DocumentDetail 组件加载文档详情
```

### 6.3 状态管理策略

| 状态类型 | 管理方式 | 说明 |
|----------|----------|------|
| 服务端数据 | React Query | 自动缓存、重试、失效 |
| 表单状态 | React useState + Form.Item | Ant Design 表单 |
| 应用状态 | Zustand/React Context | 用户信息、全局配置 |
| UI 状态 | React useState | 弹窗、下拉、加载状态 |

---

## 7. 开发环境配置

### 7.1 环境变量

```bash
# .env.development
VITE_API_URL=http://localhost:8080
VITE_APP_NAME=Vec-SVC
VITE_APP_DESCRIPTION=企业级 RAG 知识管理平台
```

```bash
# .env.production
VITE_API_URL=https://api.vec-svc.example.com
VITE_APP_NAME=Vec-SVC
VITE_APP_DESCRIPTION=企业级 RAG 知识管理平台
```

### 7.2 Vite 配置

```typescript
// vite.config.ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
})
```

### 7.3 脚本命令

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "format": "prettier --write ."
  }
}
```

---

## 8. 部署方案

### 8.1 构建优化

- 使用 Vite 构建生产版本
- 配置路径别名和代码分割
- 启用 gzip 压缩
- 优化静态资源（图片、字体）

### 8.2 Docker 部署

```dockerfile
# Dockerfile
FROM node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html

COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### 8.3 Nginx 配置

```nginx
server {
    listen 80;
    server_name _;

    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://vec-svc:8080/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 8.4 部署架构

```
┌─────────────────────────────────────────────────────────┐
│                    Nginx (反向代理)                      │
│   ┌─────────────────┐  ┌─────────────────┐              │
│   │   前端静态资源    │  │   /api -> 后端    │              │
│   │   (dist/)        │  │   vec-svc:8080   │              │
│   └─────────────────┘  └─────────────────┘              │
└─────────────────────────┬───────────────────────────────┘
                          │
    ┌─────────────────────┼─────────────────────┐
    ▼                     ▼                     ▼
┌──────────┐       ┌──────────┐       ┌──────────┐
│ PostgreSQL │       │  Milvus   │       │  MinIO   │
│ (元数据)   │       │ (向量库)   │       │ (文件存储)│
└──────────┘       └──────────┘       └──────────┘
```

---

## 9. 风险与应对

### 9.1 技术风险

| 风险 | 等级 | 应对措施 |
|------|------|----------|
| React Query 缓存策略不当导致数据不一致 | 🟡 中 | 合理设置 cacheTime 和 staleTime，关键数据手动失效 |
| 知识图谱可视化性能问题（大量节点） | 🟡 中 | 采用分页加载、虚拟滚动、按需渲染 |
| 大文件上传超时 | 🟡 中 | 使用分片上传、显示上传进度、异步任务处理 |
| 搜索结果高亮性能 | 🟢 低 | 使用虚拟列表、延迟高亮计算 |

### 9.2 依赖风险

| 风险 | 等级 | 应对措施 |
|------|------|----------|
| Ant Design 版本升级导致组件 API 变更 | 🟡 中 | 锁定版本号，升级前做兼容性测试 |
| ECharts 图表配置复杂 | 🟢 低 | 封装通用图表组件，参考官方示例 |

### 9.3 进度风险

| 风险 | 等级 | 应对措施 |
|------|------|----------|
| API 接口定义变更 | 🟡 中 | 与后端团队保持同步，使用 OpenAPI/Swagger 文档 |
| 页面开发工作量预估偏差 | 🟢 低 | 预留 20% 缓冲时间，优先完成核心功能 |

---

**文档结束**

> 本文档是 Vec-SVC 前端开发的完整规划，包含技术选型、项目结构、里程碑计划和 API 对接方案。
> 每次重大变更后应更新此文档。