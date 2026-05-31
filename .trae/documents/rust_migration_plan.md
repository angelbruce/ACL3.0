# ACL 项目 Rust 微服务 + SaaS 迁移计划

## 一、项目现状分析

### 1.1 当前技术栈

| 层次 | 技术 | 说明 |
|------|------|------|
| 语言 | C# .NET | Windows 桌面应用 |
| UI | WinForms | 传统桌面界面 |
| 数据库 | SQLite | 本地文件数据库 |
| LLM 集成 | OpenAI API | 使用 System.ClientModel |
| 核心能力 | Agent + Flow | AI 代理编排引擎 |

### 1.2 核心业务模块

```
┌─────────────────────────────────────────────────────────────┐
│                      ACL (WinForms)                        │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│    Agent    │    Flow     │    LLM      │      MCP          │
│  代理管理   │  流程编排   │  模型调用   │   工具调用        │
├─────────────┼─────────────┼─────────────┼───────────────────┤
│                   SQLiteDataStore                          │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 关键数据模型

| 模型 | 用途 | 关键字段 |
|------|------|----------|
| AgentInfo | 代理定义 | Id, Name, Defination, Tools, Skills |
| FlowInfo | 流程配置 | Id, Name, Config(JSON) |
| FlowRuntime | 流程运行时 | FlowId, IsOver, Nodes |
| Session | 会话 | Id, Description, Items |
| LLMModelInfo | 模型配置 | Name, AccessUrl, ApiKey |

---

## 二、迁移目标架构

### 2.1 微服务架构设计

```
┌──────────────────────────────────────────────────────────────────────┐
│                          SaaS 架构                                   │
├─────────────────────┬─────────────────────┬─────────────────────────┤
│     API Gateway     │    Auth Service     │      Frontend (React)   │
│   (反向代理/路由)    │   (认证/授权)       │    (Web UI)             │
├─────────────────────┴─────────────────────┴─────────────────────────┤
├─────────────┬─────────────┬─────────────┬─────────────┬─────────────┤
│  agent-svc  │  flow-svc   │  llm-svc    │  mcp-svc    │  session-svc│
│  代理服务   │  流程服务   │  模型服务   │  工具服务   │   会话服务  │
├─────────────┴─────────────┴─────────────┴─────────────┴─────────────┤
│                         Message Queue (Redis/RabbitMQ)              │
├──────────────────────────────────────────────────────────────────────┤
│              PostgreSQL          │           Redis Cache            │
│           (持久化存储)           │         (缓存/会话)              │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.2 技术选型

| 分类 | 技术 | 选型理由 |
|------|------|----------|
| 语言 | Rust | 高性能、内存安全、并发友好 |
| 框架 | Axum | 异步 Web 框架，生态成熟 |
| 数据库 | PostgreSQL | 支持 JSONB、全文搜索、事务 |
| ORM | Diesel | Rust 成熟 ORM，类型安全 |
| 缓存 | Redis | 会话管理、热点数据缓存 |
| 消息队列 | Redis Pub/Sub | 轻量级，与缓存共用 |
| API 文档 | OpenAPI/Swagger | 标准接口文档 |
| 配置 | Config + dotenv | 环境变量管理 |
| 日志 | tracing | Rust 生态标准日志 |

### 2.3 微服务职责划分

| 服务 | 职责 | 核心功能 |
|------|------|----------|
| **agent-svc** | 代理管理 | CRUD、技能配置、工具绑定 |
| **flow-svc** | 流程编排 | 流程定义、运行时管理、状态机 |
| **llm-svc** | 模型调用 | LLM 请求转发、流式响应 |
| **mcp-svc** | 工具调用 | MCP 服务管理、工具执行 |
| **session-svc** | 会话管理 | 会话 CRUD、消息历史 |
| **auth-svc** | 认证授权 | 用户管理、JWT、RBAC |

---

## 三、数据模型迁移

### 3.1 实体映射表

| 原 C# 实体 | Rust 实体 | 说明 |
|------------|-----------|------|
| AgentInfo | agent | 代理基本信息 |
| AgentBody | agent_detail | 代理完整配置（含工具/技能） |
| AgentSkillInfo | agent_skill | 代理技能 |
| AgentMcpToolInfo | agent_tool | 代理工具绑定 |
| FlowInfo | flow | 流程定义 |
| FlowConfig | flow_config | 流程配置(JSONB) |
| FlowItem | flow_item | 流程节点 |
| FlowRuntime | flow_runtime | 流程运行时 |
| FlowRuntimeNode | flow_runtime_node | 运行时节点 |
| Session | session | 会话 |
| SessionItem | session_item | 会话消息 |
| LLMModelInfo | llm_model | 模型配置 |

### 3.2 数据库 Schema 设计

```sql
-- agents 表
CREATE TABLE agents (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    defination TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- flows 表
CREATE TABLE flows (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    config JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- flow_runtimes 表
CREATE TABLE flow_runtimes (
    id BIGSERIAL PRIMARY KEY,
    flow_id BIGINT REFERENCES flows(id),
    is_over BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- sessions 表
CREATE TABLE sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- llm_models 表
CREATE TABLE llm_models (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    access_url VARCHAR(512) NOT NULL,
    api_key VARCHAR(512) NOT NULL,
    is_default BOOLEAN DEFAULT FALSE
);
```

---

## 四、API 设计

### 4.1 Agent Service API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/api/agents` | 获取代理列表 |
| GET | `/api/agents/{id}` | 获取单个代理 |
| POST | `/api/agents` | 创建代理 |
| PUT | `/api/agents/{id}` | 更新代理 |
| DELETE | `/api/agents/{id}` | 删除代理 |

### 4.2 Flow Service API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/api/flows` | 获取流程列表 |
| GET | `/api/flows/{id}` | 获取流程详情 |
| POST | `/api/flows` | 创建流程 |
| POST | `/api/flows/{id}/start` | 启动流程 |
| GET | `/api/flows/{id}/runtimes` | 获取流程运行时列表 |
| GET | `/api/flow-runtimes/{id}` | 获取运行时详情 |

### 4.3 Session Service API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/api/sessions` | 获取会话列表 |
| GET | `/api/sessions/{id}` | 获取会话详情 |
| POST | `/api/sessions` | 创建会话 |
| POST | `/api/sessions/{id}/messages` | 添加消息 |

---

## 五、迁移实施计划

### 5.1 阶段划分

| 阶段 | 时间 | 目标 | 关键任务 |
|------|------|------|----------|
| **Phase 1** | 2-3 周 | 基础架构搭建 | Rust 项目初始化、依赖配置、基础框架 |
| **Phase 2** | 3-4 周 | 核心服务开发 | Agent、Flow、Session 服务 |
| **Phase 3** | 2-3 周 | LLM/MCP 集成 | 模型调用、工具执行、流式响应 |
| **Phase 4** | 2 周 | 数据迁移 | SQLite → PostgreSQL 数据迁移 |
| **Phase 5** | 2 周 | 测试与验证 | 单元测试、集成测试、性能测试 |

### 5.2 Phase 1: 基础架构

```
任务清单：
1. 创建 Rust 工作空间 (workspace)
2. 配置 Cargo.toml 依赖
3. 搭建 Axum 基础框架
4. 配置 PostgreSQL + Redis 连接
5. 实现认证中间件 (JWT)
6. 配置日志和监控
```

### 5.3 Phase 2: 核心服务

```
任务清单：
1. 定义数据库 Schema (Diesel migration)
2. 实现 Agent CRUD
3. 实现 Flow CRUD
4. 实现 Flow Runtime 状态管理
5. 实现 Session 管理
6. API 接口开发
```

### 5.4 Phase 3: LLM/MCP 集成

```
任务清单：
1. LLM 客户端封装
2. 流式响应支持
3. MCP 工具调用框架
4. Tool Call 解析与执行
5. 错误处理与重试机制
```

### 5.5 Phase 4: 数据迁移

```
任务清单：
1. 编写数据迁移脚本
2. SQLite 数据导出
3. PostgreSQL 数据导入
4. 数据验证
```

---

## 六、关键技术挑战

### 6.1 异步流式响应

Rust Axum 天然支持异步流式响应，可直接使用 `Stream` 处理 LLM 流式输出。

### 6.2 状态机实现

Flow 运行时需要实现复杂的状态机逻辑，推荐使用 `smol_str` + 枚举实现状态转换。

### 6.3 JSON 配置解析

使用 `serde_json` 处理 Flow 的 JSON 配置，结合 `serde` 实现类型安全的序列化/反序列化。

### 6.4 并发安全

Rust 的所有权模型天然保证并发安全，配合 `tokio` 实现高效的并发处理。

---

## 七、代码结构示例

```
rust-saas/
├── crates/
│   ├── agent-svc/          # 代理服务
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── routes/
│   │   │   ├── handlers/
│   │   │   ├── models/
│   │   │   └── repository/
│   ├── flow-svc/           # 流程服务
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── state_machine/
│   │   │   ├── routes/
│   │   │   └── repository/
│   ├── llm-svc/            # 模型服务
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── client/
│   │   │   └── routes/
│   ├── mcp-svc/            # 工具服务
│   ├── session-svc/        # 会话服务
│   └── shared/             # 共享库
│       ├── src/
│       │   ├── models/
│       │   ├── errors/
│       │   └── utils/
├── docker-compose.yml
└── Cargo.toml
```

---

## 八、风险评估

| 风险 | 等级 | 应对措施 |
|------|------|----------|
| Rust 学习曲线 | 中 | 预留学习时间，参考成熟项目 |
| 数据迁移一致性 | 高 | 分批迁移，验证后切换 |
| LLM API 兼容性 | 中 | 使用标准 OpenAI 接口 |
| 性能优化 | 中 | 压测后针对性优化 |

---

## 九、下一步行动

1. **确认技术选型** - 确认 PostgreSQL、Redis、Axum 等技术栈
2. **环境准备** - 搭建开发环境和 CI/CD
3. **开始 Phase 1** - 初始化 Rust 项目和基础框架

如需进一步讨论某个具体环节，请告诉我！