-- ACL 系统初始化脚本
-- 包含所有左侧导航菜单和对应权限

-- 清空现有数据（可选）
-- TRUNCATE TABLE personnel_roles, role_permissions, personnel_departments, permissions, roles, menus, personnel, departments CASCADE;

-- 插入顶级菜单
INSERT INTO menus (name, path, parent_id, icon, sort_order, created_at, updated_at) VALUES
-- 主菜单
('会话', '/sessions', NULL, 'MessageSquare', 1, NOW(), NOW()),
('Agent', '/agents', NULL, 'Bot', 2, NOW(), NOW()),
('工作流', '/flows', NULL, 'Workflow', 3, NOW(), NOW()),
('模型', '/models', NULL, 'Settings', 4, NOW(), NOW()),
('工具', '/tools', NULL, 'Wrench', 5, NOW(), NOW()),
('MCP服务器', '/mcp-servers', NULL, 'Server', 6, NOW(), NOW()),
('工作区', '/workspace', NULL, 'FolderOpen', 7, NOW(), NOW()),
('看板', '/kanban', NULL, 'ClipboardList', 8, NOW(), NOW()),
('订阅', '/subscriptions', NULL, 'Bell', 9, NOW(), NOW()),
-- 系统管理（父菜单）
('系统管理', '/admin', NULL, 'Settings', 10, NOW(), NOW());

-- 插入系统管理子菜单
INSERT INTO menus (name, path, parent_id, icon, sort_order, created_at, updated_at) VALUES
('系统初始化', '/admin/init', (SELECT id FROM menus WHERE name = '系统管理'), 'Settings', 1001, NOW(), NOW()),
('人员管理', '/admin/personnel', (SELECT id FROM menus WHERE name = '系统管理'), 'Users', 1002, NOW(), NOW()),
('部门管理', '/admin/departments', (SELECT id FROM menus WHERE name = '系统管理'), 'Building', 1003, NOW(), NOW()),
('角色管理', '/admin/roles', (SELECT id FROM menus WHERE name = '系统管理'), 'Shield', 1004, NOW(), NOW()),
('菜单管理', '/admin/menus', (SELECT id FROM menus WHERE name = '系统管理'), 'FolderTree', 1005, NOW(), NOW()),
('权限管理', '/admin/permissions', (SELECT id FROM menus WHERE name = '系统管理'), 'Key', 1006, NOW(), NOW());

-- 为每个菜单创建权限（访问、创建、编辑、删除）
-- 会话菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '会话'), '访问会话', 'view', NOW()),
((SELECT id FROM menus WHERE name = '会话'), '创建会话', 'create', NOW()),
((SELECT id FROM menus WHERE name = '会话'), '编辑会话', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '会话'), '删除会话', 'delete', NOW());

-- Agent菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = 'Agent'), '访问Agent', 'view', NOW()),
((SELECT id FROM menus WHERE name = 'Agent'), '创建Agent', 'create', NOW()),
((SELECT id FROM menus WHERE name = 'Agent'), '编辑Agent', 'edit', NOW()),
((SELECT id FROM menus WHERE name = 'Agent'), '删除Agent', 'delete', NOW());

-- 工作流菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '工作流'), '访问工作流', 'view', NOW()),
((SELECT id FROM menus WHERE name = '工作流'), '创建工作流', 'create', NOW()),
((SELECT id FROM menus WHERE name = '工作流'), '编辑工作流', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '工作流'), '删除工作流', 'delete', NOW());

-- 模型菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '模型'), '访问模型', 'view', NOW()),
((SELECT id FROM menus WHERE name = '模型'), '创建模型', 'create', NOW()),
((SELECT id FROM menus WHERE name = '模型'), '编辑模型', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '模型'), '删除模型', 'delete', NOW());

-- 工具菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '工具'), '访问工具', 'view', NOW()),
((SELECT id FROM menus WHERE name = '工具'), '创建工具', 'create', NOW()),
((SELECT id FROM menus WHERE name = '工具'), '编辑工具', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '工具'), '删除工具', 'delete', NOW());

-- MCP服务器菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = 'MCP服务器'), '访问MCP服务器', 'view', NOW()),
((SELECT id FROM menus WHERE name = 'MCP服务器'), '创建MCP服务器', 'create', NOW()),
((SELECT id FROM menus WHERE name = 'MCP服务器'), '编辑MCP服务器', 'edit', NOW()),
((SELECT id FROM menus WHERE name = 'MCP服务器'), '删除MCP服务器', 'delete', NOW());

-- 工作区菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '工作区'), '访问工作区', 'view', NOW()),
((SELECT id FROM menus WHERE name = '工作区'), '创建工作区', 'create', NOW()),
((SELECT id FROM menus WHERE name = '工作区'), '编辑工作区', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '工作区'), '删除工作区', 'delete', NOW());

-- 看板菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '看板'), '访问看板', 'view', NOW()),
((SELECT id FROM menus WHERE name = '看板'), '创建看板', 'create', NOW()),
((SELECT id FROM menus WHERE name = '看板'), '编辑看板', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '看板'), '删除看板', 'delete', NOW());

-- 订阅菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '订阅'), '访问订阅', 'view', NOW()),
((SELECT id FROM menus WHERE name = '订阅'), '创建订阅', 'create', NOW()),
((SELECT id FROM menus WHERE name = '订阅'), '编辑订阅', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '订阅'), '删除订阅', 'delete', NOW());

-- 系统管理菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '系统管理'), '访问系统管理', 'view', NOW()),
((SELECT id FROM menus WHERE name = '系统管理'), '创建系统管理', 'create', NOW()),
((SELECT id FROM menus WHERE name = '系统管理'), '编辑系统管理', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '系统管理'), '删除系统管理', 'delete', NOW());

-- 系统初始化菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '系统初始化'), '访问系统初始化', 'view', NOW()),
((SELECT id FROM menus WHERE name = '系统初始化'), '创建系统初始化', 'create', NOW()),
((SELECT id FROM menus WHERE name = '系统初始化'), '编辑系统初始化', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '系统初始化'), '删除系统初始化', 'delete', NOW());

-- 人员管理菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '人员管理'), '访问人员管理', 'view', NOW()),
((SELECT id FROM menus WHERE name = '人员管理'), '创建人员管理', 'create', NOW()),
((SELECT id FROM menus WHERE name = '人员管理'), '编辑人员管理', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '人员管理'), '删除人员管理', 'delete', NOW());

-- 部门管理菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '部门管理'), '访问部门管理', 'view', NOW()),
((SELECT id FROM menus WHERE name = '部门管理'), '创建部门管理', 'create', NOW()),
((SELECT id FROM menus WHERE name = '部门管理'), '编辑部门管理', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '部门管理'), '删除部门管理', 'delete', NOW());

-- 角色管理菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '角色管理'), '访问角色管理', 'view', NOW()),
((SELECT id FROM menus WHERE name = '角色管理'), '创建角色管理', 'create', NOW()),
((SELECT id FROM menus WHERE name = '角色管理'), '编辑角色管理', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '角色管理'), '删除角色管理', 'delete', NOW());

-- 菜单管理菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '菜单管理'), '访问菜单管理', 'view', NOW()),
((SELECT id FROM menus WHERE name = '菜单管理'), '创建菜单管理', 'create', NOW()),
((SELECT id FROM menus WHERE name = '菜单管理'), '编辑菜单管理', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '菜单管理'), '删除菜单管理', 'delete', NOW());

-- 权限管理菜单权限
INSERT INTO permissions (menu_id, name, description, created_at) VALUES
((SELECT id FROM menus WHERE name = '权限管理'), '访问权限管理', 'view', NOW()),
((SELECT id FROM menus WHERE name = '权限管理'), '创建权限管理', 'create', NOW()),
((SELECT id FROM menus WHERE name = '权限管理'), '编辑权限管理', 'edit', NOW()),
((SELECT id FROM menus WHERE name = '权限管理'), '删除权限管理', 'delete', NOW());

-- 创建超级管理员角色
INSERT INTO roles (name, description, is_super_admin, created_at, updated_at) VALUES
('超级管理员', '拥有所有权限的管理员角色', TRUE, NOW(), NOW());

-- 将所有权限分配给超级管理员角色
INSERT INTO role_permissions (role_id, permission_id, created_at)
SELECT (SELECT id FROM roles WHERE name = '超级管理员'), id, NOW()
FROM permissions;

-- 创建普通管理员角色
INSERT INTO roles (name, description, is_super_admin, created_at, updated_at) VALUES
('管理员', '拥有基本管理权限的角色', FALSE, NOW(), NOW());

-- 创建默认部门
INSERT INTO departments (name, parent_id, description, created_at, updated_at) VALUES
('技术部', NULL, '负责技术开发和维护', NOW(), NOW()),
('产品部', NULL, '负责产品设计和规划', NOW(), NOW()),
('运营部', NULL, '负责日常运营和推广', NOW(), NOW()),
('人事部', NULL, '负责人力资源管理', NOW(), NOW()),
('财务部', NULL, '负责财务和会计', NOW(), NOW());

SELECT '初始化完成！' as result;