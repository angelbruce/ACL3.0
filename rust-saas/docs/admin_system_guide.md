# ACL 权限管理系统使用指南

## 系统概述

本系统提供了一套完整的权限管理解决方案，包括：
- **人员管理**：管理用户信息，与注册账号关联
- **部门管理**：支持树形结构的部门管理
- **角色管理**：定义不同角色的权限集合
- **权限管理**：管理访问各个功能菜单的权限
- **菜单管理**：管理系统功能菜单

## 核心概念

### 1. 人员 (Personnel)
- 与注册账号（User）一一对应
- 包含姓名、性别、邮件、微信、手机号等信息
- 可以属于多个部门
- 可以拥有多个角色
- 通过角色获得相应权限
- 自动记录最后登录时间

### 2. 部门 (Department)
- 支持树形结构（parent_id）
- 人员可以在多个部门中
- 简单的CRUD操作

### 3. 角色 (Role)
- 是权限的复合体
- 可以标记为超级管理员（拥有所有权限）
- 通过 `role_permissions` 关联多个权限

### 4. 权限 (Permission)
- 关联到具体的菜单
- 定义了对菜单的操作类型（访问、创建、编辑、删除）
- 通过 `role_permissions` 分配给角色

### 5. 菜单 (Menu)
- 系统功能入口
- 支持树形结构
- 包含路径、图标等信息

## 快速开始

### 1. 数据库迁移

首先运行迁移脚本创建数据表：

```bash
psql -U postgres -d your_database -f migrations/001_create_admin_tables.sql
```

或手动执行以下SQL创建表结构（参考 `migrations/001_create_admin_tables.sql`）。

### 2. 启动服务

确保 PostgreSQL 和 Redis 服务已启动，然后运行：

```bash
cd rust-saas
cargo build --release
./target/release/admin-svc
```

服务将在 `0.0.0.0:3007` 端口启动。

### 3. 初始化系统

#### 3.1 初始化超级管理员角色

```bash
curl -X POST http://localhost:3007/init-super-admin
```

响应示例：
```json
{
  "id": 1,
  "name": "超级管理员",
  "description": "拥有所有权限的超级管理员角色",
  "is_super_admin": true,
  "created_at": "2026-05-29T12:00:00",
  "updated_at": "2026-05-29T12:00:00"
}
```

#### 3.2 初始化默认菜单

```bash
curl -X POST http://localhost:3007/init-menus
```

这将创建以下默认菜单结构：
- 系统管理
  - 用户管理
  - 部门管理
  - 角色管理
  - 菜单管理
  - 权限管理
- Agent管理
- 会话管理
- Flow管理
- 模型管理
- MCP管理

#### 3.3 初始化默认权限

```bash
curl -X POST http://localhost:3007/init-permissions
```

这将为每个菜单创建4个默认权限：
- 访问{菜单名}
- 创建{菜单名}
- 编辑{菜单名}
- 删除{菜单名}

#### 3.4 分配所有权限给超级管理员

```bash
curl -X POST http://localhost:3007/init-super-admin-all
```

### 4. 设置用户为超级管理员

#### 4.1 注册用户（自动创建人员信息）

```bash
curl -X POST http://localhost:3007/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@example.com", "password": "password123"}'
```

响应示例：
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": 1
}
```

注册成功后，会自动创建对应的人员记录（初始不在任何部门中）。

#### 4.2 获取用户的人员ID

```bash
curl -X GET http://localhost:3007/personnel
```

找到对应用户的人员ID（通过email匹配）。

#### 4.3 设为超级管理员

```bash
curl -X POST http://localhost:3007/personnel/{personnel_id}/assign-super-admin
```

## API 文档

### 初始化 API

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /init-super-admin | 初始化超级管理员角色 |
| POST | /init-menus | 初始化默认菜单 |
| POST | /init-permissions | 初始化默认权限 |
| POST | /init-super-admin-all | 将所有权限分配给超级管理员角色 |

### 人员管理 API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /personnel | 获取所有人员列表 |
| POST | /personnel | 创建人员 |
| GET | /personnel/:id | 获取单个人员信息 |
| PUT | /personnel/:id | 更新人员信息 |
| GET | /personnel/:id/details | 获取人员完整信息（含部门、角色、权限） |
| GET | /personnel/:personnel_id/departments | 获取人员所属部门 |
| GET | /personnel/:personnel_id/roles | 获取人员角色 |
| GET | /personnel/:personnel_id/permissions | 获取人员权限 |
| POST | /personnel/:personnel_id/assign-departments | 分配部门给人员 |
| POST | /personnel/:personnel_id/assign-roles | 分配角色给人员 |
| POST | /personnel/:personnel_id/assign-super-admin | 设为超级管理员 |
| GET | /personnel/:personnel_id/is-super-admin | 检查是否为超级管理员 |

### 部门管理 API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /departments | 获取所有部门 |
| POST | /departments | 创建部门 |
| GET | /departments/:id | 获取部门信息 |
| PUT | /departments/:id | 更新部门 |
| DELETE | /departments/:id | 删除部门 |

### 菜单管理 API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /menus | 获取所有菜单 |
| POST | /menus | 创建菜单 |
| GET | /menus/:id | 获取菜单信息 |
| PUT | /menus/:id | 更新菜单 |
| DELETE | /menus/:id | 删除菜单 |

### 权限管理 API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /permissions | 获取所有权限 |
| POST | /permissions | 创建权限 |
| GET | /permissions/by-menu/:menu_id | 获取菜单下的权限 |
| DELETE | /permissions/:id | 删除权限 |

### 角色管理 API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /roles | 获取所有角色 |
| POST | /roles | 创建角色 |
| GET | /roles/:id | 获取角色信息 |
| PUT | /roles/:id | 更新角色 |
| DELETE | /roles/:id | 删除角色 |
| GET | /roles/:role_id/permissions | 获取角色权限 |
| POST | /roles/:role_id/permissions | 分配权限给角色 |

## 使用示例

### 1. 创建部门

```bash
# 创建顶级部门
curl -X POST http://localhost:3007/departments \
  -H "Content-Type: application/json" \
  -d '{"name": "技术部", "description": "负责技术研发"}'

# 创建子部门
curl -X POST http://localhost:3007/departments \
  -H "Content-Type: application/json" \
  -d '{"name": "前端组", "parent_id": 1, "description": "前端开发"}'
```

### 2. 创建角色

```bash
curl -X POST http://localhost:3007/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "普通管理员",
    "description": "部分权限的管理员",
    "is_super_admin": false,
    "permission_ids": [1, 2, 3, 4]
  }'
```

### 3. 分配部门和角色

```bash
# 分配部门
curl -X POST http://localhost:3007/personnel/1/assign-departments \
  -H "Content-Type: application/json" \
  -d '{"personnel_id": 1, "department_ids": [1, 2]}'

# 分配角色
curl -X POST http://localhost:3007/personnel/1/assign-roles \
  -H "Content-Type: application/json" \
  -d '{"personnel_id": 1, "role_ids": [1, 2]}'
```

### 4. 更新人员信息

```bash
curl -X PUT http://localhost:3007/personnel/1 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "张三",
    "gender": "男",
    "email": "zhangsan@example.com",
    "wechat": "zhangsan",
    "phone": "13800138000"
  }'
```

### 5. 获取人员完整信息

```bash
curl -X GET http://localhost:3007/personnel/1/details
```

响应示例：
```json
{
  "personnel": {
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
  },
  "departments": [
    {
      "id": 1,
      "name": "技术部",
      "parent_id": null,
      "description": "负责技术研发",
      "created_at": "2026-05-29T09:00:00",
      "updated_at": "2026-05-29T09:00:00"
    }
  ],
  "roles": [
    {
      "id": 1,
      "name": "超级管理员",
      "description": "拥有所有权限的超级管理员角色",
      "is_super_admin": true,
      "created_at": "2026-05-29T09:00:00",
      "updated_at": "2026-05-29T09:00:00"
    }
  ],
  "permissions": [
    {
      "id": 1,
      "menu_id": 1,
      "name": "访问系统管理",
      "description": "view",
      "created_at": "2026-05-29T09:00:00"
    }
    // ... 更多权限
  ]
}
```

## 权限检查逻辑

### 超级管理员
- 拥有 `is_super_admin = true` 标记的角色
- 自动拥有所有权限
- 系统会自动检查并授予所有权限

### 普通用户
- 根据分配的角色获取权限
- 权限 = 角色权限的并集
- 如果拥有多个角色，权限为所有角色权限的合集

## 注意事项

1. **用户注册时自动创建人员**：用户在auth服务注册后，会自动创建对应的人员记录
2. **部门树形结构**：通过 parent_id 实现，支持无限层级
3. **权限继承**：超级管理员自动拥有所有权限，无需单独分配
4. **数据一致性**：删除角色时会自动清理关联的权限和人员关系
5. **最后登录时间**：用户登录时自动更新

## Docker 部署

```bash
# 构建镜像
docker build -f crates/admin-svc/Dockerfile -t admin-svc .

# 运行容器
docker run -d -p 3007:3007 \
  -e DATABASE_URL=postgres://user:password@host:5432/db \
  -e REDIS_URL=redis://host:6379 \
  -e PORT=3007 \
  admin-svc
```

## 故障排查

### 1. 服务启动失败
- 检查 PostgreSQL 连接
- 检查 Redis 连接
- 查看日志输出

### 2. 权限不生效
- 确认角色已正确分配给人员
- 确认权限已正确分配给角色
- 检查是否为超级管理员（超级管理员自动拥有所有权限）

### 3. 部门删除失败
- 检查是否有人员属于该部门
- 检查是否有子部门

## 扩展开发

### 添加新的菜单和权限

```bash
# 1. 创建菜单
curl -X POST http://localhost:3007/menus \
  -H "Content-Type: application/json" \
  -d '{
    "name": "报表管理",
    "path": "/reports",
    "icon": "chart",
    "sort_order": 700
  }'

# 2. 创建权限
curl -X POST http://localhost:3007/permissions \
  -H "Content-Type: application/json" \
  -d '{
    "menu_id": 12,
    "name": "查看报表",
    "description": "view"
  }'

# 3. 分配权限给角色
curl -X POST http://localhost:3007/roles/2/permissions \
  -H "Content-Type: application/json" \
  -d [1, 2, 3]  # 权限ID列表
```

## 联系支持

如有问题，请查看系统日志或联系开发团队。
