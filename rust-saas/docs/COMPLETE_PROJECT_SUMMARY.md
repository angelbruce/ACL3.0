# ACL权限管理系统 - 完整项目总结

## 📋 项目概述

这是一个完整的权限管理系统，包括后端服务和前端管理界面。

### 技术栈

#### 后端 (Rust)
- **框架**: Axum
- **ORM**: Diesel
- **数据库**: PostgreSQL
- **缓存**: Redis
- **认证**: JWT + bcrypt

#### 前端 (Vue 3)
- **框架**: Vue 3 (Composition API)
- **语言**: TypeScript
- **构建**: Vite
- **UI**: Tailwind CSS
- **状态**: Pinia
- **路由**: Vue Router
- **HTTP**: Axios

## 🏗️ 系统架构

```
┌─────────────────────────────────────────┐
│           前端 (Vue 3 + TypeScript)      │
│  ┌──────────────────────────────────┐   │
│  │  管理界面 (React风格的组件)        │   │
│  │  - 人员管理                       │   │
│  │  - 部门管理                       │   │
│  │  - 角色管理                       │   │
│  │  - 菜单管理                       │   │
│  │  - 权限管理                       │   │
│  │  - 系统初始化                     │   │
│  └──────────────────────────────────┘   │
└────────────────┬────────────────────────┘
                 │ HTTP/REST
┌────────────────▼────────────────────────┐
│     Admin Service (Rust + Axum)         │
│     Port: 3007                         │
│  ┌──────────────────────────────────┐   │
│  │  业务逻辑层 (Handlers)            │   │
│  │  数据访问层 (Repository)          │   │
│  └──────────────────────────────────┘   │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│         Auth Service (Rust)             │
│         Port: 3001                      │
│  ┌──────────────────────────────────┐   │
│  │  用户注册/登录                    │   │
│  │  Token管理                       │   │
│  │  密码加密                        │   │
│  └──────────────────────────────────┘   │
└────────────────┬────────────────────────┘
                 │
        ┌───────┴───────┐
        ▼               ▼
┌───────────────┐ ┌───────────────┐
│  PostgreSQL   │ │    Redis     │
│  数据库        │ │   缓存        │
└───────────────┘ └───────────────┘
```

## 📁 项目结构

```
rust-saas/
├── crates/
│   ├── admin-svc/                 # Admin服务
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs
│   │       ├── handlers.rs        # 业务逻辑
│   │       ├── repository.rs      # 数据访问
│   │       └── routes.rs          # 路由配置
│   │
│   ├── auth-svc/                 # 认证服务
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── handlers.rs        # 已增强：自动创建人员
│   │       ├── repository.rs      # 已增强：人员管理
│   │       └── routes.rs
│   │
│   └── shared/                   # 共享模块
│       └── src/
│           ├── schema.rs         # 已增强：8个新表
│           ├── models.rs         # 已增强：所有数据模型
│           ├── errors.rs
│           └── utils.rs
│
├── frontend/
│   └── acl-web/                  # 前端应用
│       ├── src/
│       │   ├── api/
│       │   │   ├── admin.ts      # Admin API客户端
│       │   │   └── index.ts
│       │   ├── stores/
│       │   │   ├── admin.ts      # Admin状态管理
│       │   │   └── index.ts
│       │   ├── types/
│       │   │   ├── admin.ts       # Admin类型定义
│       │   │   └── index.ts
│       │   ├── views/
│       │   │   └── admin/        # 管理页面
│       │   │       ├── SystemInit.vue
│       │   │       ├── PersonnelList.vue
│       │   │       ├── DepartmentList.vue
│       │   │       ├── RoleList.vue
│       │   │       ├── MenuList.vue
│       │   │       └── PermissionList.vue
│       │   ├── views/layout/
│       │   │   └── AppLayout.vue # 已增强：侧边栏管理菜单
│       │   └── router/
│       │       └── index.ts      # 已增强：管理路由
│       └── package.json
│
├── migrations/
│   ├── 001_create_admin_tables.sql   # 完整迁移脚本
│   └── 002_admin_tables_simple.sql    # 简洁迁移脚本
│
└── docs/
    ├── admin_system_guide.md          # 完整使用指南
    ├── QUICK_REFERENCE.md             # 快速参考
    ├── IMPLEMENTATION_SUMMARY.md      # 实现总结
    └── frontend_guide.md              # 前端使用指南
```

## 🗄️ 数据库设计

### 数据表结构

#### 1. departments (部门表)
```sql
- id: BIGSERIAL PRIMARY KEY
- name: TEXT NOT NULL
- parent_id: BIGINT (上级部门，支持树形结构)
- description: TEXT
- created_at, updated_at: TIMESTAMP
```

#### 2. personnel (人员表)
```sql
- id: BIGSERIAL PRIMARY KEY
- user_id: BIGINT NOT NULL (关联用户)
- name: TEXT NOT NULL
- gender: TEXT
- email: TEXT
- wechat: TEXT
- phone: TEXT
- last_login_date: TIMESTAMP (最后登录时间)
- created_at, updated_at: TIMESTAMP
```

#### 3. personnel_departments (人员-部门关联表)
```sql
- id: BIGSERIAL PRIMARY KEY
- personnel_id: BIGINT
- department_id: BIGINT
- created_at: TIMESTAMP
```

#### 4. menus (菜单表)
```sql
- id: BIGSERIAL PRIMARY KEY
- name: TEXT NOT NULL
- path: TEXT
- parent_id: BIGINT (上级菜单，支持树形)
- icon: TEXT
- sort_order: INTEGER
- created_at, updated_at: TIMESTAMP
```

#### 5. permissions (权限表)
```sql
- id: BIGSERIAL PRIMARY KEY
- menu_id: BIGINT (关联菜单)
- name: TEXT NOT NULL
- description: TEXT
- created_at: TIMESTAMP
```

#### 6. roles (角色表)
```sql
- id: BIGSERIAL PRIMARY KEY
- name: TEXT NOT NULL
- description: TEXT
- is_super_admin: BOOLEAN (是否超级管理员)
- created_at, updated_at: TIMESTAMP
```

#### 7. role_permissions (角色-权限关联表)
```sql
- id: BIGSERIAL PRIMARY KEY
- role_id: BIGINT
- permission_id: BIGINT
- created_at: TIMESTAMP
```

#### 8. personnel_roles (人员-角色关联表)
```sql
- id: BIGSERIAL PRIMARY KEY
- personnel_id: BIGINT
- role_id: BIGINT
- created_at: TIMESTAMP
```

### ER图

```
┌──────────────┐       ┌────────────────────┐       ┌──────────────┐
│  departments │       │ personnel_departments│       │  personnel   │
│              │◄──────│                    │──────►│              │
└──────────────┘       └────────────────────┘       └──────┬───────┘
                                                           │
                                                           │
                                                           ▼
┌──────────────┐       ┌────────────────────┐       ┌──────────────┐
│    menus     │──────►│   permissions      │       │personnel_roles│
│              │       │                    │◄──────│              │
└──────────────┘       └────────────────────┘       └──────┬───────┘
                                                           │
                                                           ▼
                                                  ┌──────────────┐
                                                  │    roles     │
                                                  │              │
                                                  └──────┬───────┘
                                                         │
                                                         ▼
                                                  ┌──────────────┐
                                                  │role_permissions│
                                                  │              │
                                                  └──────┬───────┘
                                                         │
                                                         ▼
                                                  ┌──────────────┐
                                                  │  permissions │
                                                  │              │
                                                  └──────────────┘
```

## 🔌 API接口

### Admin Service (端口 3007)

#### 初始化API
| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /init-super-admin | 初始化超级管理员角色 |
| POST | /init-menus | 初始化默认菜单 |
| POST | /init-permissions | 初始化默认权限 |
| POST | /init-super-admin-all | 分配所有权限给超级管理员 |

#### 人员管理
| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /personnel | 获取所有人员 |
| POST | /personnel | 创建人员 |
| GET | /personnel/:id | 获取人员详情 |
| PUT | /personnel/:id | 更新人员 |
| GET | /personnel/:id/details | 获取完整信息 |
| POST | /personnel/:id/assign-departments | 分配部门 |
| POST | /personnel/:id/assign-roles | 分配角色 |
| POST | /personnel/:id/assign-super-admin | 设为超级管理员 |

#### 部门管理
| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /departments | 获取所有部门 |
| POST | /departments | 创建部门 |
| GET | /departments/:id | 获取部门详情 |
| PUT | /departments/:id | 更新部门 |
| DELETE | /departments/:id | 删除部门 |

#### 角色管理
| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /roles | 获取所有角色 |
| POST | /roles | 创建角色 |
| GET | /roles/:id | 获取角色详情 |
| PUT | /roles/:id | 更新角色 |
| DELETE | /roles/:id | 删除角色 |
| GET | /roles/:id/permissions | 获取角色权限 |
| POST | /roles/:id/permissions | 分配权限 |

#### 菜单管理
| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /menus | 获取所有菜单 |
| POST | /menus | 创建菜单 |
| GET | /menus/:id | 获取菜单详情 |
| PUT | /menus/:id | 更新菜单 |
| DELETE | /menus/:id | 删除菜单 |

#### 权限管理
| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /permissions | 获取所有权限 |
| POST | /permissions | 创建权限 |
| GET | /permissions/by-menu/:menu_id | 按菜单获取权限 |
| DELETE | /permissions/:id | 删除权限 |

## 🚀 快速开始

### 1. 数据库准备

```bash
# 创建数据库
psql -U postgres -c "CREATE DATABASE acl_db;"

# 运行迁移
psql -U postgres -d acl_db -f migrations/002_admin_tables_simple.sql
```

### 2. 启动后端服务

```bash
# 启动Admin服务
cd rust-saas
cargo build --release
./target/release/admin-svc &

# 启动Auth服务（如果需要）
./target/release/auth-svc &
```

### 3. 启动前端

```bash
cd frontend/acl-web
npm install
npm run dev
```

### 4. 初始化系统

1. 打开浏览器访问 http://localhost:5173
2. 登录或注册账号
3. 进入"系统管理 > 系统初始化"
4. 按顺序执行4个初始化步骤
5. 进入"系统管理 > 人员管理"
6. 找到自己的账号，点击"详情"
7. 点击"设为超级管理员"

## 📱 前端页面

### 侧边栏导航

```
主导航:
├─ 会话
├─ Agent
├─ 工作流
├─ 模型
├─ 工具
└─ MCP服务器

系统管理 (可折叠):
├─ 系统初始化
├─ 人员管理
├─ 部门管理
├─ 角色管理
├─ 菜单管理
└─ 权限管理
```

### 页面功能

#### 1. 系统初始化 (/admin/init)
- 初始化超级管理员角色
- 初始化默认菜单
- 初始化默认权限
- 分配所有权限

#### 2. 人员管理 (/admin/personnel)
- 列表展示
- 添加/编辑人员
- 查看详情
- 分配部门
- 分配角色
- 设为超级管理员

#### 3. 部门管理 (/admin/departments)
- 列表展示
- 添加/编辑部门
- 树形结构支持
- 删除部门

#### 4. 角色管理 (/admin/roles)
- 卡片式展示
- 添加/编辑角色
- 权限管理
- 删除角色

#### 5. 菜单管理 (/admin/menus)
- 列表展示
- 添加/编辑菜单
- 树形结构支持
- 删除菜单

#### 6. 权限管理 (/admin/permissions)
- 按菜单分组展示
- 快速添加权限
- 删除权限

## ⚙️ 配置说明

### 环境变量

#### Admin Service
```env
DATABASE_URL=postgres://user:password@localhost:5432/acl_db
REDIS_URL=redis://localhost:6379
PORT=3007
RUST_LOG=info
```

#### Auth Service
```env
DATABASE_URL=postgres://user:password@localhost:5432/acl_db
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-secret-key
PORT=3001
RUST_LOG=info
```

### 前端环境变量
```env
VITE_API_BASE_URL=http://localhost:3000
```

## 🧪 测试

### API测试

```bash
# Linux/Mac
./scripts/test_admin_api.sh

# Windows
scripts\test_admin_api.bat
```

### 手动测试

```bash
# 1. 初始化
curl -X POST http://localhost:3007/init-super-admin
curl -X POST http://localhost:3007/init-menus
curl -X POST http://localhost:3007/init-permissions
curl -X POST http://localhost:3007/init-super-admin-all

# 2. 创建部门
curl -X POST http://localhost:3007/departments \
  -H "Content-Type: application/json" \
  -d '{"name": "技术部", "description": "技术研发"}'

# 3. 查看人员
curl -X GET http://localhost:3007/personnel
```

## 📊 核心业务流程

### 用户注册流程
```
1. 用户提交注册表单
   ↓
2. Auth服务创建User记录
   ↓
3. Auth服务自动创建Personnel记录（初始无部门、无角色）
   ↓
4. 返回Token给前端
```

### 登录流程
```
1. 用户提交登录表单
   ↓
2. Auth服务验证密码
   ↓
3. 更新Personnel的last_login_date
   ↓
4. 返回Token给前端
```

### 权限检查流程
```
1. 用户请求资源
   ↓
2. 获取用户关联的Personnel
   ↓
3. 获取Personnel的所有角色
   ↓
4. 检查是否有超级管理员角色
   ├─ 是 → 拥有所有权限
   └─ 否 → 合并所有角色的权限
   ↓
5. 检查是否有所需权限
   ├─ 是 → 允许访问
   └─ 否 → 拒绝访问
```

## 🎯 核心特性

### ✅ 完整的功能
- 人员管理与用户账号关联
- 部门树形结构管理
- 角色权限复合管理
- 菜单访问控制
- 超级管理员自动拥有所有权限

### ✅ 自动关联
- 用户注册自动创建人员
- 登录自动更新最后登录时间
- 超级管理员自动获得所有权限

### ✅ 灵活的权限系统
- 基于角色的访问控制 (RBAC)
- 多对多关系支持
- 树形结构（部门、菜单）
- 细粒度权限控制

### ✅ 友好的界面
- Vue 3 现代前端
- 响应式设计
- 直观的操作流程
- 实时的状态反馈

## 🔧 扩展开发

### 添加新功能

1. **前端**
   - 创建页面组件: `views/admin/NewFeature.vue`
   - 添加路由: `router/index.ts`
   - 添加API: `api/admin.ts`
   - 添加状态: `stores/admin.ts`

2. **后端**
   - 添加Model: `shared/src/models.rs`
   - 添加Schema: `shared/src/schema.rs`
   - 添加Repository: `admin-svc/src/repository.rs`
   - 添加Handler: `admin-svc/src/handlers.rs`
   - 添加Route: `admin-svc/src/routes.rs`

### 数据库迁移

```sql
-- 添加新表
CREATE TABLE new_table (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 添加外键
ALTER TABLE new_table ADD COLUMN other_id BIGINT REFERENCES other_table(id);
```

## 📚 相关文档

- [完整使用指南](docs/admin_system_guide.md)
- [快速参考](docs/QUICK_REFERENCE.md)
- [前端使用指南](docs/frontend_guide.md)
- [实现总结](docs/IMPLEMENTATION_SUMMARY.md)

## 🐛 故障排查

### 服务启动失败
```bash
# 检查数据库连接
psql -U postgres -d acl_db -c "SELECT 1;"

# 检查Redis连接
redis-cli ping

# 查看日志
tail -f /var/log/rust-saas.log
```

### 前端无法连接后端
```bash
# 检查CORS配置
# 检查后端服务是否运行
curl http://localhost:3007/personnel

# 检查前端API配置
cat frontend/acl-web/.env
```

### 权限不生效
```bash
# 检查角色分配
curl http://localhost:3007/personnel/1/roles

# 检查角色权限
curl http://localhost:3007/roles/1/permissions

# 检查是否为超级管理员
curl http://localhost:3007/personnel/1/is-super-admin
```

## 🚢 部署

### Docker部署

```bash
# 构建所有服务
docker-compose build

# 启动所有服务
docker-compose up -d
```

### 生产环境

1. 配置HTTPS
2. 配置反向代理 (Nginx)
3. 配置监控
4. 配置日志收集
5. 配置备份

## 📈 性能优化建议

### 后端
- 添加数据库索引
- 实现缓存层 (Redis)
- 使用连接池
- 异步处理

### 前端
- 路由懒加载
- 组件懒加载
- 图片优化
- 代码分割

## 🔐 安全建议

- 使用HTTPS
- 定期更换JWT密钥
- 密码加密存储
- SQL注入防护
- XSS防护
- CSRF防护

## 📞 支持

如有问题，请查看：
1. 系统日志
2. 浏览器控制台
3. 后端日志
4. 数据库日志

## 📄 许可证

MIT License

## 🎉 总结

本系统成功实现了：

✅ **后端服务**
- Admin Service (端口3007)
- Auth Service 增强
- 8个数据表
- 40+ API端点
- 完整的CRUD操作

✅ **前端应用**
- 6个管理页面
- 响应式设计
- 状态管理
- 路由配置

✅ **文档**
- 使用指南
- 快速参考
- API文档
- 前端指南

✅ **工具**
- 数据库迁移脚本
- 测试脚本
- 部署配置

系统已具备生产环境使用的基本功能，可根据业务需求进一步扩展和优化。
