# ACL Web 技术架构文档

## 1. 项目结构

```
acl-web/
├── public/                  # 静态资源
├── src/
│   ├── api/                 # API 请求模块
│   │   ├── client.ts        # Axios 实例配置
│   │   ├── auth.ts          # 认证 API
│   │   ├── agent.ts         # Agent API
│   │   ├── flow.ts          # Flow API
│   │   ├── session.ts       # Session API
│   │   ├── llm.ts           # LLM API
│   │   └── mcp.ts           # MCP API
│   ├── components/          # 通用组件
│   │   ├── common/          # 通用 UI 组件
│   │   ├── layout/          # 布局组件
│   │   ├── agent/           # Agent 相关组件
│   │   ├── flow/            # Flow 相关组件
│   │   └── chat/            # 聊天相关组件
│   ├── composables/         # Vue Composables
│   │   ├── useAuth.ts       # 认证状态管理
│   │   ├── useApi.ts        # API 请求封装
│   │   └── useStream.ts     # 流式响应处理
│   ├── stores/              # Pinia 状态管理
│   │   ├── auth.ts          # 认证状态
│   │   ├── agent.ts         # Agent 状态
│   │   ├── flow.ts          # Flow 状态
│   │   ├── session.ts       # Session 状态
│   │   └── llm.ts           # LLM 状态
│   ├── views/               # 页面视图
│   │   ├── auth/            # 认证页面
│   │   ├── agents/          # Agent 页面
│   │   ├── flows/           # Flow 页面
│   │   ├── sessions/        # Session 页面
│   │   ├── models/          # 模型页面
│   │   └── tools/           # 工具页面
│   ├── router/              # 路由配置
│   │   └── index.ts
│   ├── types/               # TypeScript 类型定义
│   │   ├── api.ts           # API 类型
│   │   ├── agent.ts         # Agent 类型
│   │   ├── flow.ts          # Flow 类型
│   │   └── session.ts       # Session 类型
│   ├── utils/               # 工具函数
│   │   ├── storage.ts       # 本地存储
│   │   └── format.ts        # 格式化工具
│   ├── App.vue
│   ├── main.ts
│   └── style.css            # 全局样式
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
└── postcss.config.js
```

## 2. API 服务配置

### 2.1 服务地址
```typescript
const API_BASE = {
  AUTH:    'http://localhost:8080',
  AGENT:   'http://localhost:8081',
  FLOW:    'http://localhost:8082',
  SESSION: 'http://localhost:8083',
  LLM:     'http://localhost:8084',
  MCP:     'http://localhost:8085',
}
```

### 2.2 Axios 配置
```typescript
// 请求拦截器：自动添加 Token
// 响应拦截器：处理 Token 过期
// 错误处理：统一错误提示
```

## 3. 认证流程

### 3.1 Token 管理
- Access Token：有效期 1 小时，用于 API 请求
- Refresh Token：有效期 7 天，用于刷新 Access Token
- Token 存储在 localStorage

### 3.2 认证状态
```typescript
interface AuthState {
  user: User | null
  accessToken: string | null
  refreshToken: string | null
  isAuthenticated: boolean
}
```

## 4. 核心组件设计

### 4.1 布局组件
- `AppLayout.vue` - 主布局（侧边栏 + 内容区）
- `SideNav.vue` - 左侧导航栏
- `TopBar.vue` - 顶部状态栏
- `ContentArea.vue` - 内容区域

### 4.2 通用组件
- `Button.vue` - 按钮组件
- `Input.vue` - 输入框组件
- `Card.vue` - 卡片组件
- `Modal.vue` - 模态框
- `Table.vue` - 表格组件
- `Loading.vue` - 加载状态
- `Empty.vue` - 空状态

### 4.3 业务组件
- `ChatWindow.vue` - 聊天窗口
- `ChatMessage.vue` - 聊天消息
- `FlowEditor.vue` - 流程编辑器
- `FlowNode.vue` - 流程节点
- `AgentForm.vue` - Agent 表单
- `ModelForm.vue` - 模型表单

## 5. 状态管理 (Pinia)

### 5.1 Store 结构
```typescript
// authStore - 认证状态
// agentStore - Agent 数据
// flowStore - Flow 数据
// sessionStore - 会话数据
// llmStore - LLM 模型数据
```

### 5.2 持久化
- 使用 pinia-plugin-persistedstate
- 敏感信息加密存储

## 6. 路由设计

### 6.1 路由配置
```typescript
const routes = [
  { path: '/login', name: 'Login', component: Login },
  { path: '/register', name: 'Register', component: Register },
  {
    path: '/',
    component: AppLayout,
    meta: { requiresAuth: true },
    children: [
      { path: '', redirect: '/sessions' },
      { path: 'sessions', name: 'Sessions', component: SessionList },
      { path: 'sessions/:id', name: 'SessionDetail', component: SessionDetail },
      { path: 'agents', name: 'Agents', component: AgentList },
      { path: 'agents/new', name: 'NewAgent', component: AgentForm },
      { path: 'agents/:id/edit', name: 'EditAgent', component: AgentForm },
      { path: 'flows', name: 'Flows', component: FlowList },
      { path: 'flows/new', name: 'NewFlow', component: FlowEditor },
      { path: 'flows/:id/edit', name: 'EditFlow', component: FlowEditor },
      { path: 'flows/:id/run', name: 'RunFlow', component: FlowRunner },
      { path: 'models', name: 'Models', component: ModelList },
      { path: 'tools', name: 'Tools', component: ToolList },
    ]
  }
]
```

### 6.2 路由守卫
```typescript
// 全局前置守卫
// 1. 检查是否已认证
// 2. Token 有效性检查
// 3. 自动刷新过期 Token
```

## 7. 流式响应处理

### 7.1 SSE 实现
```typescript
// 使用 EventSource 或 Fetch + ReadableStream
// 实时显示 LLM 响应
// 支持中断生成
```

### 7.2 Chat 组件
```vue
<!-- 支持流式输出 -->
<!-- 打字机效果 -->
<!-- 消息复制 -->
```

## 8. Flow 可视化编辑器

### 8.1 技术选型
- 使用 Vue Flow (@vue-flow/core)
- 自定义节点类型
- 支持拖拽、缩放、连线和

### 8.2 节点类型
- Start 节点
- Agent 节点
- Condition 节点
- End 节点

### 8.3 节点配置面板
- 右侧可折叠面板
- 显示选中节点属性
- 支持编辑节点配置

## 9. 环境变量

```env
VITE_API_AUTH_URL=http://localhost:8080
VITE_API_AGENT_URL=http://localhost:8081
VITE_API_FLOW_URL=http://localhost:8082
VITE_API_SESSION_URL=http://localhost:8083
VITE_API_LLM_URL=http://localhost:8084
VITE_API_MCP_URL=http://localhost:8085
```

## 10. 部署配置

### 10.1 开发环境
- Vite Dev Server
- 热模块替换 (HMR)

### 10.2 生产环境
- `pnpm build` 构建
- 静态资源部署到 CDN
- Nginx 配置反向代理

### 10.3 Nginx 配置
```nginx
location /api/auth {
  proxy_pass http://localhost:8080;
}
location /api/agent {
  proxy_pass http://localhost:8081;
}
# ... 其他服务配置
```
