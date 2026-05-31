# ACL Web 前端产品需求文档

## 1. 项目概述

### 项目名称
ACL Web - Agent Control Layer Web Interface

### 项目类型
基于 Vue 3 + TypeScript 的 SaaS 管理平台前端

### 核心功能概述
为 Rust 微服务后端提供完整的 Web 管理界面，包括 Agent 管理、Flow 流程编排、会话管理、LLM 模型配置和 MCP 工具调用等功能。

### 目标用户
- AI 应用开发者
- AI 代理（Agent）管理员
- 流程编排工程师

---

## 2. 功能列表

### 2.1 认证模块
- [ ] 用户注册（邮箱 + 密码）
- [ ] 用户登录（JWT Token 认证）
- [ ] Token 刷新机制
- [ ] 登出功能
- [ ] 登录状态持久化

### 2.2 Agent 管理
- [ ] Agent 列表展示
- [ ] 创建新 Agent（名称、定义、工具、技能配置）
- [ ] 编辑 Agent 信息
- [ ] 删除 Agent
- [ ] Agent 详情查看

### 2.3 Flow 流程编排
- [ ] Flow 列表展示
- [ ] 创建新 Flow（可视化编辑器）
- [ ] 编辑 Flow 配置
- [ ] 删除 Flow
- [ ] Flow 执行（启动/停止）
- [ ] Flow 运行时状态查看
- [ ] Flow 输出日志

### 2.4 Session 会话管理
- [ ] 会话列表展示
- [ ] 创建新会话
- [ ] 会话详情查看
- [ ] 发送消息（支持流式响应）
- [ ] 会话消息历史
- [ ] 删除会话

### 2.5 LLM 模型配置
- [ ] 模型列表展示
- [ ] 添加新模型（名称、API URL、API Key）
- [ ] 设置默认模型
- [ ] 编辑模型信息
- [ ] 删除模型

### 2.6 MCP 工具
- [ ] 工具列表展示
- [ ] 工具详情查看（名称、描述、参数 Schema）
- [ ] 工具调用测试

### 2.7 导航和布局
- [ ] 左侧导航栏（会话、Agent、Flow、模型、MCP）
- [ ] 主内容区域
- [ ] 顶部状态栏（当前模型、Agent、会话信息）

---

## 3. 数据结构

### 3.1 API 服务地址
```
AUTH_SERVICE:  http://localhost:8080
AGENT_SERVICE: http://localhost:8081
FLOW_SERVICE:  http://localhost:8082
SESSION_SERVICE: http://localhost:8083
LLM_SERVICE:   http://localhost:8084
MCP_SERVICE:   http://localhost:8085
```

### 3.2 API 端点

#### 认证服务 (AUTH_SERVICE)
- `POST /auth/register` - 用户注册
- `POST /auth/login` - 用户登录
- `POST /auth/refresh` - 刷新 Token
- `POST /auth/logout` - 登出
- `GET /users` - 获取用户列表
- `GET /users/:id` - 获取用户详情

#### Agent 服务 (AGENT_SERVICE)
- `GET /agents` - 获取 Agent 列表
- `GET /agents/:id` - 获取 Agent 详情
- `POST /agents` - 创建 Agent
- `PUT /agents/:id` - 更新 Agent
- `DELETE /agents/:id` - 删除 Agent

#### Flow 服务 (FLOW_SERVICE)
- `GET /flows` - 获取 Flow 列表
- `GET /flows/:id` - 获取 Flow 详情
- `POST /flows` - 创建 Flow
- `PUT /flows/:id` - 更新 Flow
- `DELETE /flows/:id` - 删除 Flow
- `POST /flows/:id/start` - 启动 Flow
- `POST /flows/:id/stop` - 停止 Flow
- `GET /flows/:id/runtimes` - 获取 Flow 运行时列表

#### Session 服务 (SESSION_SERVICE)
- `GET /sessions` - 获取会话列表
- `GET /sessions/:id` - 获取会话详情
- `POST /sessions` - 创建会话
- `DELETE /sessions/:id` - 删除会话
- `GET /sessions/:id/messages` - 获取会话消息
- `POST /sessions/:id/messages` - 添加消息

#### LLM 服务 (LLM_SERVICE)
- `GET /models` - 获取模型列表
- `GET /models/:id` - 获取模型详情
- `POST /models` - 创建模型
- `PUT /models/:id` - 更新模型
- `DELETE /models/:id` - 删除模型
- `POST /chat` - 聊天（非流式）
- `POST /chat/stream` - 聊天（流式）

#### MCP 服务 (MCP_SERVICE)
- `GET /tools` - 获取工具列表
- `POST /tools/:name/call` - 调用工具

---

## 4. 页面结构

### 4.1 页面列表
1. **登录页** - `/login`
2. **注册页** - `/register`
3. **主布局** - 包含侧边栏和内容区
4. **会话列表页** - `/sessions`
5. **会话详情页** - `/sessions/:id`
6. **Agent 列表页** - `/agents`
7. **Agent 编辑页** - `/agents/new`, `/agents/:id/edit`
8. **Flow 列表页** - `/flows`
9. **Flow 编辑器页** - `/flows/new`, `/flows/:id/edit`
10. **Flow 执行页** - `/flows/:id/run`
11. **模型管理页** - `/models`
12. **MCP 工具页** - `/tools`

### 4.2 路由守卫
- 未登录用户只能访问 `/login` 和 `/register`
- 已登录用户访问其他页面需验证 Token
- Token 过期自动跳转登录页

---

## 5. UI/UX 设计方向

### 5.1 视觉风格
- **风格**: 现代简约科技风（Dark Mode 为主）
- **主色调**: 深色背景 + 亮色强调色
- **强调色**: 科技蓝 (#3B82F6) 或 紫色 (#8B5CF6)
- **字体**: Inter/Roboto Mono 或类似等宽字体

### 5.2 布局特点
- 左侧固定侧边栏导航
- 右侧可折叠面板（Flow 编辑器属性面板）
- 底部状态栏显示当前环境信息
- 支持深色/浅色主题切换

### 5.3 交互特点
- 实时流式响应展示（打字机效果）
- 拖拽式流程编排
- 快捷键支持
- 响应式布局

---

## 6. 技术栈

- **框架**: Vue 3 + Composition API
- **语言**: TypeScript
- **构建工具**: Vite
- **路由**: Vue Router 4
- **状态管理**: Pinia
- **HTTP 客户端**: Axios
- **CSS**: Tailwind CSS
- **图标**: Lucide Vue
- **流程图**: Vue Flow 或自定义 mxGraph 集成

---

## 7. 优先级

### P0 - 核心功能
1. 登录/注册 + 认证
2. 会话管理 + 聊天
3. Agent 管理

### P1 - 重要功能
4. Flow 流程编排
5. LLM 模型配置

### P2 - 辅助功能
6. MCP 工具调用
7. Flow 执行和监控

---

## 8. 注意事项

- 所有 API 调用需携带 JWT Token
- Token 存储在 localStorage 或 Cookie 中
- 敏感信息（如 API Key）不得明文显示
- 支持响应式布局（桌面端为主）
