
# 工作区与看板系统实现计划

## 1. 需求分析

根据用户需求，系统需要实现以下核心功能：

### 1.1 工作区功能
- **文件下载**：用户可以下载自己工作区的文件
- **文件删除**：用户可以删除自己工作区的文件
- **文件列表**：用户可以查看自己工作区的文件列表
- **权限隔离**：用户只能访问和管理自己的文件，不能访问他人文件

### 1.2 看板功能
- **文件分享**：用户可以将自己的文件分享到公有看板
- **看板订阅**：用户可以订阅公有看板
- **订阅查看**：订阅后可以在自己工作区看到他人分享的文件（只读）
- **取消订阅**：用户可以取消订阅看板

### 1.3 数据模型设计

#### 新增数据库表

| 表名 | 用途 | 核心字段 |
|------|------|----------|
| `workspace_files` | 记录工作区文件元数据 | user_id, file_path, file_name, size, created_at |
| `kanban_boards` | 看板信息 | name, description, is_public, created_by |
| `kanban_items` | 看板文件项 | board_id, user_id, file_path, shared_at |
| `kanban_subscriptions` | 看板订阅关系 | board_id, user_id, subscribed_at |

### 1.4 API 接口设计

| 模块 | 接口 | 方法 | 说明 |
|------|------|------|------|
| 工作区 | `/api/workspace/files` | GET | 获取当前用户工作区文件列表 |
| 工作区 | `/api/workspace/files/{path}` | GET | 下载文件 |
| 工作区 | `/api/workspace/files/{path}` | DELETE | 删除自己的文件 |
| 看板 | `/api/kanban/boards` | GET | 获取所有公有看板列表 |
| 看板 | `/api/kanban/boards` | POST | 创建看板 |
| 看板 | `/api/kanban/boards/{id}` | GET | 获取看板详情和文件列表 |
| 看板 | `/api/kanban/boards/{id}/subscribe` | POST | 订阅看板 |
| 看板 | `/api/kanban/boards/{id}/unsubscribe` | POST | 取消订阅 |
| 看板 | `/api/kanban/boards/{id}/files` | POST | 分享文件到看板 |
| 看板 | `/api/kanban/subscriptions` | GET | 获取当前用户订阅的看板 |

---

## 2. 实现方案

### 2.1 新增服务模块

创建新的 `workspace-svc` 服务来处理工作区和看板相关业务：

```
rust-saas/crates/workspace-svc/
├── src/
│   ├── main.rs          # 服务入口
│   ├── routes.rs        # 路由定义
│   ├── handlers.rs      # 业务处理
│   ├── repository.rs    # 数据库操作
│   └── middleware.rs    # 权限中间件
├── Cargo.toml
└── Dockerfile
```

### 2.2 修改共享模块

1. **shared/src/schema.rs**：添加新表定义
2. **shared/src/models.rs**：添加新数据模型

### 2.3 文件存储结构

```
/workspace_storage/
├── 1/                  # user_id=1 的工作区
│   ├── file1.txt
│   └── subfolder/
│       └── file2.txt
├── 2/                  # user_id=2 的工作区
└── shared/             # 共享文件临时存储（可选）
```

---

## 3. 实施步骤

### 步骤 1：更新共享模块

**文件**: `shared/src/schema.rs`
- 添加 `workspace_files`, `kanban_boards`, `kanban_items`, `kanban_subscriptions` 表定义

**文件**: `shared/src/models.rs`
- 添加对应的结构体定义：`WorkspaceFile`, `KanbanBoard`, `KanbanItem`, `KanbanSubscription`
- 添加请求/响应 DTO

### 步骤 2：创建工作区服务

**文件**: `crates/workspace-svc/Cargo.toml`
- 添加依赖：axum, diesel, shared 等

**文件**: `crates/workspace-svc/src/main.rs`
- 初始化服务，配置路由

**文件**: `crates/workspace-svc/src/routes.rs`
- 定义工作区和看板的 API 路由

**文件**: `crates/workspace-svc/src/handlers.rs`
- 实现文件列表、下载、删除逻辑
- 实现看板 CRUD 和订阅逻辑

**文件**: `crates/workspace-svc/src/repository.rs`
- 实现数据库操作方法

**文件**: `crates/workspace-svc/src/middleware.rs`
- 实现工作区访问权限检查中间件

**文件**: `crates/workspace-svc/Dockerfile`
- Docker 构建配置

### 步骤 3：更新 Docker Compose

**文件**: `docker-compose.yml`
- 添加 workspace-svc 服务定义

### 步骤 4：数据库迁移

- 生成数据库迁移脚本
- 运行迁移命令创建新表

---

## 4. 安全考虑

1. **路径遍历防护**：对用户输入的路径进行规范化处理，检查是否包含 `..`
2. **访问权限校验**：确保用户只能访问自己的工作区文件
3. **订阅权限**：订阅后只能查看，不能修改他人文件
4. **JWT 认证**：所有接口必须携带有效 Token
5. **文件类型过滤**：禁止分享危险文件类型

---

## 5. 依赖与风险

### 5.1 依赖项

| 依赖 | 版本 | 用途 |
|------|------|------|
| axum | 0.7+ | Web 框架 |
| diesel | 2.0+ | ORM |
| shared | - | 共享模块 |
| tokio | 1.0+ | 异步运行时 |

### 5.2 风险点

| 风险 | 描述 | 应对方案 |
|------|------|----------|
| 路径遍历攻击 | 用户可能通过 `../` 访问其他目录 | 路径规范化 + 白名单校验 |
| 文件过大 | 下载大文件可能影响性能 | 限制单个文件大小 |
| 并发访问 | 多用户同时操作可能冲突 | 使用数据库事务和锁 |
| 存储空间 | 工作区文件过多占用空间 | 定期清理 + 容量限制 |

---

## 6. 测试计划

### 6.1 单元测试

- 文件操作测试（列表、下载、删除）
- 看板 CRUD 测试
- 订阅功能测试
- 权限校验测试

### 6.2 集成测试

- API 端点测试
- 数据库操作测试
- 认证授权测试

---

## 7. 部署计划

1. 构建 Docker 镜像：`docker-compose build workspace-svc`
2. 启动服务：`docker-compose up -d workspace-svc`
3. 运行数据库迁移：`docker-compose exec workspace-svc diesel migration run`
