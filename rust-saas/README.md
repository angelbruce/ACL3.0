# ACL 权限管理系统

一个完整的权限管理系统，包括后端服务和前端管理界面。

## 🎯 功能特性

- ✅ **人员管理**: 与用户账号关联，包含姓名、性别、邮件、微信、手机号等
- ✅ **部门管理**: 支持树形结构，人员可在多个部门
- ✅ **角色管理**: 权限的复合体，支持超级管理员
- ✅ **权限管理**: 基于功能菜单的访问控制
- ✅ **菜单管理**: 树形结构，支持图标和排序
- ✅ **自动关联**: 注册自动创建人员，登录更新最后时间

## 🚀 快速开始

### 1. 数据库准备

```bash
# 创建数据库
psql -U postgres -c "CREATE DATABASE acl_db;"

# 运行迁移
psql -U postgres -d acl_db -f migrations/002_admin_tables_simple.sql
```

### 2. 启动后端

```bash
cd rust-saas

# 构建服务
cargo build --release

# 启动Admin服务 (端口3007)
./target/release/admin-svc &
```

### 3. 启动前端

```bash
cd frontend/acl-web

# 安装依赖
npm install

# 启动开发服务器
npm run dev
```

### 4. 初始化系统

1. 打开浏览器: http://localhost:5173
2. 注册账号
3. 进入"系统管理 > 系统初始化"
4. 按顺序执行4个初始化步骤
5. 进入"系统管理 > 人员管理"
6. 找到自己的账号，点击"详情"
7. 点击"设为超级管理员"

## 📁 项目结构

```
rust-saas/
├── crates/                    # Rust后端服务
│   ├── admin-svc/            # Admin管理服务
│   ├── auth-svc/            # 认证服务
│   └── shared/              # 共享模块
├── frontend/
│   └── acl-web/             # Vue3前端应用
├── migrations/              # 数据库迁移脚本
└── docs/                   # 文档
```

## 🔌 API接口

### 初始化
- `POST /init-super-admin` - 初始化超级管理员
- `POST /init-menus` - 初始化菜单
- `POST /init-permissions` - 初始化权限
- `POST /init-super-admin-all` - 分配所有权限

### 人员管理
- `GET/POST /personnel` - 列表/创建
- `GET/PUT /personnel/:id` - 详情/更新
- `POST /personnel/:id/assign-super-admin` - 设为超级管理员

### 部门管理
- `GET/POST /departments` - 列表/创建
- `GET/PUT/DELETE /departments/:id` - 详情/更新/删除

### 角色管理
- `GET/POST /roles` - 列表/创建
- `GET/PUT/DELETE /roles/:id` - 详情/更新/删除
- `POST /roles/:id/permissions` - 分配权限

### 菜单管理
- `GET/POST /menus` - 列表/创建
- `GET/PUT/DELETE /menus/:id` - 详情/更新/删除

### 权限管理
- `GET/POST /permissions` - 列表/创建
- `DELETE /permissions/:id` - 删除

## 📱 前端页面

- `/admin/init` - 系统初始化
- `/admin/personnel` - 人员管理
- `/admin/departments` - 部门管理
- `/admin/roles` - 角色管理
- `/admin/menus` - 菜单管理
- `/admin/permissions` - 权限管理

## 🛠️ 技术栈

### 后端
- Rust + Axum
- Diesel ORM
- PostgreSQL
- Redis
- JWT + bcrypt

### 前端
- Vue 3 + TypeScript
- Vite
- Tailwind CSS
- Pinia
- Vue Router

## 📚 文档

- [完整使用指南](docs/admin_system_guide.md)
- [快速参考](docs/QUICK_REFERENCE.md)
- [前端使用指南](docs/frontend_guide.md)
- [完整项目总结](docs/COMPLETE_PROJECT_SUMMARY.md)

## 🔧 开发

### 前端开发
```bash
cd frontend/acl-web
npm install
npm run dev    # 开发模式
npm run build  # 生产构建
```

### 后端开发
```bash
cargo build    # 编译
cargo run      # 运行
cargo test     # 测试
```

## 📄 许可证

MIT License

## 🎉

如有问题，请查看 [docs/](docs/) 目录下的详细文档。
