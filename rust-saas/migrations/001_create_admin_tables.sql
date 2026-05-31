-- ACL权限管理系统数据库迁移脚本
-- 运行此脚本以创建所有必要的数据表

-- 1. 部门表
CREATE TABLE IF NOT EXISTS departments (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id BIGINT REFERENCES departments(id) ON DELETE SET NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. 人员表
CREATE TABLE IF NOT EXISTS personnel (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    gender TEXT,
    email TEXT,
    wechat TEXT,
    phone TEXT,
    last_login_date TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_personnel_user_id ON personnel(user_id);

-- 3. 人员和部门关联表（多对多）
CREATE TABLE IF NOT EXISTS personnel_departments (
    id BIGSERIAL PRIMARY KEY,
    personnel_id BIGINT NOT NULL REFERENCES personnel(id) ON DELETE CASCADE,
    department_id BIGINT NOT NULL REFERENCES departments(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(personnel_id, department_id)
);

CREATE INDEX idx_personnel_departments_personnel ON personnel_departments(personnel_id);
CREATE INDEX idx_personnel_departments_department ON personnel_departments(department_id);

-- 4. 菜单表
CREATE TABLE IF NOT EXISTS menus (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT,
    parent_id BIGINT REFERENCES menus(id) ON DELETE SET NULL,
    icon TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_menus_parent ON menus(parent_id);

-- 5. 权限表
CREATE TABLE IF NOT EXISTS permissions (
    id BIGSERIAL PRIMARY KEY,
    menu_id BIGINT NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_permissions_menu ON permissions(menu_id);

-- 6. 角色表
CREATE TABLE IF NOT EXISTS roles (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_super_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 7. 角色和权限关联表（多对多）
CREATE TABLE IF NOT EXISTS role_permissions (
    id BIGSERIAL PRIMARY KEY,
    role_id BIGINT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id BIGINT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(role_id, permission_id)
);

CREATE INDEX idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission ON role_permissions(permission_id);

-- 8. 人员和角色关联表（多对多）
CREATE TABLE IF NOT EXISTS personnel_roles (
    id BIGSERIAL PRIMARY KEY,
    personnel_id BIGINT NOT NULL REFERENCES personnel(id) ON DELETE CASCADE,
    role_id BIGINT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(personnel_id, role_id)
);

CREATE INDEX idx_personnel_roles_personnel ON personnel_roles(personnel_id);
CREATE INDEX idx_personnel_roles_role ON personnel_roles(role_id);

-- 插入初始数据

-- 插入超级管理员角色
INSERT INTO roles (name, description, is_super_admin, created_at, updated_at)
VALUES ('超级管理员', '拥有所有权限的超级管理员角色', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;

-- 插入默认菜单
INSERT INTO menus (name, path, parent_id, icon, sort_order, created_at, updated_at) VALUES
('系统管理', '/admin', NULL, 'settings', 100, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('用户管理', '/admin/users', (SELECT id FROM menus WHERE name = '系统管理'), 'user', 101, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('部门管理', '/admin/departments', (SELECT id FROM menus WHERE name = '系统管理'), 'building', 102, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('角色管理', '/admin/roles', (SELECT id FROM menus WHERE name = '系统管理'), 'shield', 103, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('菜单管理', '/admin/menus', (SELECT id FROM menus WHERE name = '系统管理'), 'menu', 104, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('权限管理', '/admin/permissions', (SELECT id FROM menus WHERE name = '系统管理'), 'key', 105, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('Agent管理', '/agents', NULL, 'bot', 200, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('会话管理', '/sessions', NULL, 'message-square', 300, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('Flow管理', '/flows', NULL, 'workflow', 400, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('模型管理', '/models', NULL, 'cpu', 500, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('MCP管理', '/mcp', NULL, 'plug', 600, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;

-- 为所有菜单创建默认权限
INSERT INTO permissions (menu_id, name, description, created_at)
SELECT id, '访问' || name, 'view', CURRENT_TIMESTAMP FROM menus
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE menu_id = menus.id AND name = '访问' || menus.name)
ON CONFLICT DO NOTHING;

INSERT INTO permissions (menu_id, name, description, created_at)
SELECT id, '创建' || name, 'create', CURRENT_TIMESTAMP FROM menus
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE menu_id = menus.id AND name = '创建' || menus.name)
ON CONFLICT DO NOTHING;

INSERT INTO permissions (menu_id, name, description, created_at)
SELECT id, '编辑' || name, 'edit', CURRENT_TIMESTAMP FROM menus
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE menu_id = menus.id AND name = '编辑' || menus.name)
ON CONFLICT DO NOTHING;

INSERT INTO permissions (menu_id, name, description, created_at)
SELECT id, '删除' || name, 'delete', CURRENT_TIMESTAMP FROM menus
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE menu_id = menus.id AND name = '删除' || menus.name)
ON CONFLICT DO NOTHING;

-- 将所有权限分配给超级管理员角色
INSERT INTO role_permissions (role_id, permission_id, created_at)
SELECT (SELECT id FROM roles WHERE is_super_admin = true), id, CURRENT_TIMESTAMP
FROM permissions
WHERE NOT EXISTS (
    SELECT 1 FROM role_permissions 
    WHERE role_id = (SELECT id FROM roles WHERE is_super_admin = true) 
    AND permission_id = permissions.id
)
AND EXISTS (SELECT 1 FROM roles WHERE is_super_admin = true);

-- 创建示例部门
INSERT INTO departments (name, description, created_at, updated_at) VALUES
('技术部', '负责技术研发', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('产品部', '负责产品设计', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('运营部', '负责运营管理', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;

-- 注释：完成迁移后，可以通过以下API设置超级管理员：
-- POST /admin/init-super-admin - 初始化超级管理员角色
-- POST /admin/personnel/:id/assign-super-admin - 将人员设为超级管理员
