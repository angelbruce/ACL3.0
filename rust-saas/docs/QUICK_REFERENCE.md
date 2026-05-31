# ACL权限管理系统 - 快速参考

## 服务地址
- Admin Service: http://localhost:3007

## 快速启动命令

### 1. 数据库迁移
```bash
psql -U postgres -d your_database -f migrations/002_admin_tables_simple.sql
```

### 2. 启动服务
```bash
cd rust-saas
cargo build --release
./target/release/admin-svc
```

### 3. 初始化系统（按顺序执行）
```bash
# 3.1 初始化超级管理员角色
curl -X POST http://localhost:3007/init-super-admin

# 3.2 初始化菜单
curl -X POST http://localhost:3007/init-menus

# 3.3 初始化权限
curl -X POST http://localhost:3007/init-permissions

# 3.4 分配所有权限给超级管理员
curl -X POST http://localhost:3007/init-super-admin-all
```

### 4. 设置超级管理员
```bash
# 注册用户（自动创建人员）
curl -X POST http://localhost:3007/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@example.com", "password": "password123"}'

# 获取人员ID（通过GET /personnel查找）
# 假设人员ID为1

# 设为超级管理员
curl -X POST http://localhost:3007/personnel/1/assign-super-admin
```

## 常用API速查

### 人员管理
```bash
GET    /personnel                          # 列表
GET    /personnel/:id                       # 详情
PUT    /personnel/:id                       # 更新
GET    /personnel/:id/details               # 完整信息（含部门、角色、权限）
POST   /personnel/:id/assign-departments    # 分配部门
POST   /personnel/:id/assign-roles          # 分配角色
POST   /personnel/:id/assign-super-admin    # 设为超级管理员
```

### 部门管理
```bash
GET    /departments                         # 列表
POST   /departments                         # 创建
GET    /departments/:id                     # 详情
PUT    /departments/:id                     # 更新
DELETE /departments/:id                     # 删除
```

### 角色管理
```bash
GET    /roles                               # 列表
POST   /roles                               # 创建
GET    /roles/:id                           # 详情
PUT    /roles/:id                           # 更新
DELETE /roles/:id                           # 删除
POST   /roles/:id/permissions               # 分配权限
```

### 菜单管理
```bash
GET    /menus                               # 列表
POST   /menus                               # 创建
GET    /menus/:id                           # 详情
PUT    /menus/:id                           # 更新
DELETE /menus/:id                           # 删除
```

### 权限管理
```bash
GET    /permissions                         # 列表
POST   /permissions                         # 创建
GET    /permissions/by-menu/:id            # 按菜单查询
DELETE /permissions/:id                     # 删除
```

## 数据结构

### personnel（人员）
```json
{
  "id": 1,
  "user_id": 1,
  "name": "张三",
  "gender": "男",
  "email": "zhangsan@example.com",
  "wechat": "zhangsan",
  "phone": "13800138000",
  "last_login_date": "2026-05-29T12:00:00",
  "created_at": "2026-05-29T10:00:00",
  "updated_at": "2026-05-29T12:00:00"
}
```

### departments（部门）
```json
{
  "id": 1,
  "name": "技术部",
  "parent_id": null,
  "description": "负责技术研发",
  "created_at": "2026-05-29T09:00:00",
  "updated_at": "2026-05-29T09:00:00"
}
```

### roles（角色）
```json
{
  "id": 1,
  "name": "超级管理员",
  "description": "拥有所有权限",
  "is_super_admin": true,
  "created_at": "2026-05-29T09:00:00",
  "updated_at": "2026-05-29T09:00:00"
}
```

### menus（菜单）
```json
{
  "id": 1,
  "name": "系统管理",
  "path": "/admin",
  "parent_id": null,
  "icon": "settings",
  "sort_order": 100,
  "created_at": "2026-05-29T09:00:00",
  "updated_at": "2026-05-29T09:00:00"
}
```

### permissions（权限）
```json
{
  "id": 1,
  "menu_id": 1,
  "name": "访问系统管理",
  "description": "view",
  "created_at": "2026-05-29T09:00:00"
}
```

## 默认菜单结构

```
系统管理 (/admin)
├── 用户管理 (/admin/users)
├── 部门管理 (/admin/departments)
├── 角色管理 (/admin/roles)
├── 菜单管理 (/admin/menus)
└── 权限管理 (/admin/permissions)

Agent管理 (/agents)
会话管理 (/sessions)
Flow管理 (/flows)
模型管理 (/models)
MCP管理 (/mcp)
```

## 默认权限（每个菜单4个）
- 访问{菜单名}
- 创建{菜单名}
- 编辑{菜单名}
- 删除{菜单名}

## 核心逻辑

### 1. 用户注册流程
```
用户注册 → 自动创建人员记录 → 初始无部门、无角色
```

### 2. 登录流程
```
用户登录 → 查询人员 → 更新最后登录时间 → 生成Token
```

### 3. 权限检查
```
用户请求 → 获取人员角色 → 合并角色权限 → 检查是否有权限
         ↓
     如果是超级管理员 → 自动通过
```

### 4. 超级管理员
- `is_super_admin = true` 的角色
- 拥有所有权限
- 无需手动分配权限

## 示例场景

### 场景1：创建普通管理员

```bash
# 1. 创建角色
curl -X POST http://localhost:3007/roles \
  -H "Content-Type: application/json" \
  -d '{"name": "普通管理员", "description": "部分权限"}'

# 2. 分配权限给角色（假设权限ID为1,2,3,4）
curl -X POST http://localhost:3007/roles/2/permissions \
  -H "Content-Type: application/json" \
  -d '[1, 2, 3, 4]'

# 3. 分配角色给人员（假设人员ID为2）
curl -X POST http://localhost:3007/personnel/2/assign-roles \
  -H "Content-Type: application/json" \
  -d '{"personnel_id": 2, "role_ids": [2]}'
```

### 场景2：人员跨部门

```bash
# 创建两个部门
curl -X POST http://localhost:3007/departments \
  -H "Content-Type: application/json" \
  -d '{"name": "技术部"}'

curl -X POST http://localhost:3007/departments \
  -H "Content-Type: application/json" \
  -d '{"name": "产品部"}'

# 分配给人员（假设部门ID为1,2，人员ID为1）
curl -X POST http://localhost:3007/personnel/1/assign-departments \
  -H "Content-Type: application/json" \
  -d '{"personnel_id": 1, "department_ids": [1, 2]}'
```

### 场景3：更新人员信息

```bash
curl -X PUT http://localhost:3007/personnel/1 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "李四",
    "gender": "女",
    "email": "lisi@example.com",
    "wechat": "lisi",
    "phone": "13900139000"
  }'
```

## 故障排查

| 问题 | 解决方案 |
|------|---------|
| 服务启动失败 | 检查DATABASE_URL和REDIS_URL |
| 权限不生效 | 确认角色已分配，人员有对应角色 |
| 无法删除部门 | 检查是否有人员属于该部门 |
| 超级管理员无法登录 | 确认用户已注册且分配了超级管理员角色 |

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| DATABASE_URL | 必须设置 | PostgreSQL连接字符串 |
| REDIS_URL | redis://localhost:6379 | Redis连接字符串 |
| PORT | 3007 | 服务端口 |
| RUST_LOG | info | 日志级别 |

## 相关文档

- 完整使用指南：[docs/admin_system_guide.md](docs/admin_system_guide.md)
- 项目总结：[docs/IMPLEMENTATION_SUMMARY.md](docs/IMPLEMENTATION_SUMMARY.md)
- 迁移脚本：
  - [migrations/001_create_admin_tables.sql](migrations/001_create_admin_tables.sql)
  - [migrations/002_admin_tables_simple.sql](migrations/002_admin_tables_simple.sql)

## 技术栈

- **后端**：Rust + Axum + Diesel
- **数据库**：PostgreSQL
- **缓存**：Redis
- **认证**：JWT + bcrypt
