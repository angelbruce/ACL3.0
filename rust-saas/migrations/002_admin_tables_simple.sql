-- 简洁版迁移脚本：直接创建表结构

-- 部门表
CREATE TABLE IF NOT EXISTS departments (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id BIGINT,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 人员表
CREATE TABLE IF NOT EXISTS personnel (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    name TEXT NOT NULL,
    gender TEXT,
    email TEXT,
    wechat TEXT,
    phone TEXT,
    last_login_date TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 人员部门关联表
CREATE TABLE IF NOT EXISTS personnel_departments (
    id BIGSERIAL PRIMARY KEY,
    personnel_id BIGINT NOT NULL,
    department_id BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(personnel_id, department_id)
);

-- 菜单表
CREATE TABLE IF NOT EXISTS menus (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT,
    parent_id BIGINT,
    icon TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 权限表
CREATE TABLE IF NOT EXISTS permissions (
    id BIGSERIAL PRIMARY KEY,
    menu_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 角色表
CREATE TABLE IF NOT EXISTS roles (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_super_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 角色权限关联表
CREATE TABLE IF NOT EXISTS role_permissions (
    id BIGSERIAL PRIMARY KEY,
    role_id BIGINT NOT NULL,
    permission_id BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(role_id, permission_id)
);

-- 人员角色关联表
CREATE TABLE IF NOT EXISTS personnel_roles (
    id BIGSERIAL PRIMARY KEY,
    personnel_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(personnel_id, role_id)
);

-- 添加外键约束
ALTER TABLE departments ADD CONSTRAINT fk_departments_parent FOREIGN KEY (parent_id) REFERENCES departments(id);
ALTER TABLE personnel_departments ADD CONSTRAINT fk_pd_personnel FOREIGN KEY (personnel_id) REFERENCES personnel(id);
ALTER TABLE personnel_departments ADD CONSTRAINT fk_pd_department FOREIGN KEY (department_id) REFERENCES departments(id);
ALTER TABLE menus ADD CONSTRAINT fk_menus_parent FOREIGN KEY (parent_id) REFERENCES menus(id);
ALTER TABLE permissions ADD CONSTRAINT fk_permissions_menu FOREIGN KEY (menu_id) REFERENCES menus(id);
ALTER TABLE role_permissions ADD CONSTRAINT fk_rp_role FOREIGN KEY (role_id) REFERENCES roles(id);
ALTER TABLE role_permissions ADD CONSTRAINT fk_rp_permission FOREIGN KEY (permission_id) REFERENCES permissions(id);
ALTER TABLE personnel_roles ADD CONSTRAINT fk_pr_personnel FOREIGN KEY (personnel_id) REFERENCES personnel(id);
ALTER TABLE personnel_roles ADD CONSTRAINT fk_pr_role FOREIGN KEY (role_id) REFERENCES roles(id);

-- 添加索引
CREATE INDEX IF NOT EXISTS idx_personnel_user_id ON personnel(user_id);
CREATE INDEX IF NOT EXISTS idx_personnel_departments_personnel ON personnel_departments(personnel_id);
CREATE INDEX IF NOT EXISTS idx_personnel_departments_department ON personnel_departments(department_id);
CREATE INDEX IF NOT EXISTS idx_menus_parent ON menus(parent_id);
CREATE INDEX IF NOT EXISTS idx_permissions_menu ON permissions(menu_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission ON role_permissions(permission_id);
CREATE INDEX IF NOT EXISTS idx_personnel_roles_personnel ON personnel_roles(personnel_id);
CREATE INDEX IF NOT EXISTS idx_personnel_roles_role ON personnel_roles(role_id);
