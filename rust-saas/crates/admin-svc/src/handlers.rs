use axum::{extract::Path, Json};
use shared::errors::{ServiceResult};
use shared::models::{
    AssignDepartmentsRequest, AssignRolesRequest, CreateDepartmentRequest, CreateMenuRequest,
    CreatePermissionRequest, CreatePersonnelRequest, CreateRoleRequest, Department, Menu,
    Permission, Personnel, PersonnelWithDetails, Role, UpdateDepartmentRequest, UpdateMenuRequest,
    UpdatePersonnelRequest, UpdateRoleRequest,
};
use crate::repository::AdminRepository;

pub async fn init_super_admin() -> ServiceResult<Json<Role>> {
    let repo = AdminRepository::new();
    let role = repo.init_super_admin_role().await?;
    Ok(Json(role))
}

pub async fn create_personnel(
    Json(req): Json<CreatePersonnelRequest>,
) -> ServiceResult<Json<Personnel>> {
    let repo = AdminRepository::new();
    let personnel = repo.create_personnel(
        req.user_id,
        req.name,
        req.gender,
        req.email,
        req.wechat,
        req.phone,
    ).await?;
    Ok(Json(personnel))
}

pub async fn get_all_personnel() -> ServiceResult<Json<Vec<Personnel>>> {
    let repo = AdminRepository::new();
    let personnel_list = repo.get_all_personnel().await?;
    Ok(Json(personnel_list))
}

pub async fn get_personnel(Path(id): Path<i64>) -> ServiceResult<Json<Personnel>> {
    let repo = AdminRepository::new();
    let personnel = repo.get_personnel(id).await?;
    Ok(Json(personnel))
}

pub async fn update_personnel(
    Path(id): Path<i64>,
    Json(req): Json<UpdatePersonnelRequest>,
) -> ServiceResult<Json<Personnel>> {
    let repo = AdminRepository::new();
    let personnel = repo
        .update_personnel(id, req.name, req.gender, req.email, req.wechat, req.phone)
        .await?;
    Ok(Json(personnel))
}

pub async fn create_department(
    Json(req): Json<CreateDepartmentRequest>,
) -> ServiceResult<Json<Department>> {
    let repo = AdminRepository::new();
    let department = repo
        .create_department(req.name, req.parent_id, req.description)
        .await?;
    Ok(Json(department))
}

pub async fn get_all_departments() -> ServiceResult<Json<Vec<Department>>> {
    let repo = AdminRepository::new();
    let department_list = repo.get_all_departments().await?;
    Ok(Json(department_list))
}

pub async fn get_department(Path(id): Path<i64>) -> ServiceResult<Json<Department>> {
    let repo = AdminRepository::new();
    let department = repo.get_department(id).await?;
    Ok(Json(department))
}

pub async fn update_department(
    Path(id): Path<i64>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> ServiceResult<Json<Department>> {
    let repo = AdminRepository::new();
    let department = repo
        .update_department(id, req.name, req.parent_id, req.description)
        .await?;
    Ok(Json(department))
}

pub async fn delete_department(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    repo.delete_department(id).await?;
    Ok(Json(()))
}

pub async fn assign_departments(
    Json(req): Json<AssignDepartmentsRequest>,
) -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    repo.assign_departments(req.personnel_id, req.department_ids)
        .await?;
    Ok(Json(()))
}

pub async fn get_personnel_departments(
    Path(personnel_id): Path<i64>,
) -> ServiceResult<Json<Vec<Department>>> {
    let repo = AdminRepository::new();
    let departments = repo.get_personnel_departments(personnel_id).await?;
    Ok(Json(departments))
}

pub async fn create_menu(Json(req): Json<CreateMenuRequest>) -> ServiceResult<Json<Menu>> {
    let repo = AdminRepository::new();
    let menu = repo
        .create_menu(
            req.name,
            req.path,
            req.parent_id,
            req.icon,
            req.sort_order.unwrap_or(0),
        )
        .await?;
    Ok(Json(menu))
}

pub async fn get_all_menus() -> ServiceResult<Json<Vec<Menu>>> {
    let repo = AdminRepository::new();
    let menu_list = repo.get_all_menus().await?;
    Ok(Json(menu_list))
}

pub async fn get_menu(Path(id): Path<i64>) -> ServiceResult<Json<Menu>> {
    let repo = AdminRepository::new();
    let menu = repo.get_menu(id).await?;
    Ok(Json(menu))
}

pub async fn update_menu(
    Path(id): Path<i64>,
    Json(req): Json<UpdateMenuRequest>,
) -> ServiceResult<Json<Menu>> {
    let repo = AdminRepository::new();
    let menu = repo
        .update_menu(id, req.name, req.path, req.parent_id, req.icon, req.sort_order)
        .await?;
    Ok(Json(menu))
}

pub async fn delete_menu(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    repo.delete_menu(id).await?;
    Ok(Json(()))
}

pub async fn create_permission(
    Json(req): Json<CreatePermissionRequest>,
) -> ServiceResult<Json<Permission>> {
    let repo = AdminRepository::new();
    let permission = repo
        .create_permission(req.menu_id, req.name, req.description)
        .await?;
    Ok(Json(permission))
}

pub async fn get_all_permissions() -> ServiceResult<Json<Vec<Permission>>> {
    let repo = AdminRepository::new();
    let permission_list = repo.get_all_permissions().await?;
    Ok(Json(permission_list))
}

pub async fn get_permissions_by_menu(Path(menu_id): Path<i64>) -> ServiceResult<Json<Vec<Permission>>> {
    let repo = AdminRepository::new();
    let permission_list = repo.get_permissions_by_menu(menu_id).await?;
    Ok(Json(permission_list))
}

pub async fn delete_permission(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    repo.delete_permission(id).await?;
    Ok(Json(()))
}

pub async fn create_role(Json(req): Json<CreateRoleRequest>) -> ServiceResult<Json<Role>> {
    let repo = AdminRepository::new();
    let role = repo
        .create_role(
            req.name,
            req.description,
            req.is_super_admin.unwrap_or(false),
        )
        .await?;

    if let Some(permission_ids) = req.permission_ids {
        repo.assign_permissions_to_role(role.id, permission_ids)
            .await?;
    }

    Ok(Json(role))
}

pub async fn get_all_roles() -> ServiceResult<Json<Vec<Role>>> {
    let repo = AdminRepository::new();
    let role_list = repo.get_all_roles().await?;
    Ok(Json(role_list))
}

pub async fn get_role(Path(id): Path<i64>) -> ServiceResult<Json<Role>> {
    let repo = AdminRepository::new();
    let role = repo.get_role(id).await?;
    Ok(Json(role))
}

pub async fn update_role(
    Path(id): Path<i64>,
    Json(req): Json<UpdateRoleRequest>,
) -> ServiceResult<Json<Role>> {
    let repo = AdminRepository::new();
    let role = repo
        .update_role(id, req.name, req.description, req.is_super_admin)
        .await?;
    Ok(Json(role))
}

pub async fn delete_role(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    repo.delete_role(id).await?;
    Ok(Json(()))
}

pub async fn assign_permissions_to_role(
    Path(role_id): Path<i64>,
    Json(permission_ids): Json<Vec<i64>>,
) -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    repo.assign_permissions_to_role(role_id, permission_ids)
        .await?;
    Ok(Json(()))
}

pub async fn get_role_permissions(Path(role_id): Path<i64>) -> ServiceResult<Json<Vec<Permission>>> {
    let repo = AdminRepository::new();
    let permissions = repo.get_role_permissions(role_id).await?;
    Ok(Json(permissions))
}

pub async fn assign_roles_to_personnel(
    Json(req): Json<AssignRolesRequest>,
) -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    repo.assign_roles_to_personnel(req.personnel_id, req.role_ids)
        .await?;
    Ok(Json(()))
}

pub async fn get_personnel_roles(
    Path(personnel_id): Path<i64>,
) -> ServiceResult<Json<Vec<Role>>> {
    let repo = AdminRepository::new();
    let roles = repo.get_personnel_roles(personnel_id).await?;
    Ok(Json(roles))
}

pub async fn get_personnel_permissions(
    Path(personnel_id): Path<i64>,
) -> ServiceResult<Json<Vec<Permission>>> {
    let repo = AdminRepository::new();
    let permissions = repo.get_personnel_permissions(personnel_id).await?;
    Ok(Json(permissions))
}

pub async fn get_personnel_with_details(
    Path(id): Path<i64>,
) -> ServiceResult<Json<PersonnelWithDetails>> {
    let repo = AdminRepository::new();
    let personnel = repo.get_personnel(id).await?;
    let departments = repo.get_personnel_departments(id).await?;
    let roles = repo.get_personnel_roles(id).await?;
    let permissions = repo.get_personnel_permissions(id).await?;

    Ok(Json(PersonnelWithDetails {
        personnel,
        departments,
        roles,
        permissions,
    }))
}

pub async fn check_super_admin(Path(personnel_id): Path<i64>) -> ServiceResult<Json<bool>> {
    let repo = AdminRepository::new();
    let is_super_admin = repo.is_super_admin(personnel_id).await?;
    Ok(Json(is_super_admin))
}

pub async fn assign_super_admin_role(Path(personnel_id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    
    let role = repo.init_super_admin_role().await?;
    
    repo.assign_roles_to_personnel(personnel_id, vec![role.id])
        .await?;
    
    Ok(Json(()))
}

pub async fn init_default_menus() -> ServiceResult<Json<Vec<Menu>>> {
    let repo = AdminRepository::new();
    
    let menus = vec![
        ("会话", "/sessions", None, "MessageSquare", 1),
        ("Agent", "/agents", None, "Bot", 2),
        ("工作流", "/flows", None, "Workflow", 3),
        ("模型", "/models", None, "Settings", 4),
        ("工具", "/tools", None, "Wrench", 5),
        ("MCP服务器", "/mcp-servers", None, "Server", 6),
        ("工作区", "/workspace", None, "FolderOpen", 7),
        ("看板", "/kanban", None, "ClipboardList", 8),
        ("订阅", "/subscriptions", None, "Bell", 9),
        ("系统管理", "/admin", None, "Settings", 10),
        ("系统初始化", "/admin/init", Some(9), "Settings", 1001),
        ("人员管理", "/admin/personnel", Some(9), "Users", 1002),
        ("部门管理", "/admin/departments", Some(9), "Building", 1003),
        ("角色管理", "/admin/roles", Some(9), "Shield", 1004),
        ("菜单管理", "/admin/menus", Some(9), "FolderTree", 1005),
        ("权限管理", "/admin/permissions", Some(9), "Key", 1006),
    ];
    
    let mut created_menus = Vec::new();
    let mut menu_ids = Vec::new();
    
    for (i, (name, path, parent_idx, icon, sort)) in menus.iter().enumerate() {
        let parent_id = if let Some(idx) = parent_idx {
            menu_ids.get(*idx).copied()
        } else {
            None
        };
        
        let menu = repo.create_menu(
            name.to_string(),
            Some(path.to_string()),
            parent_id,
            Some(icon.to_string()),
            *sort,
        ).await?;
        
        menu_ids.push(menu.id);
        created_menus.push(menu);
    }
    
    Ok(Json(created_menus))
}

pub async fn init_default_permissions() -> ServiceResult<Json<Vec<Permission>>> {
    let repo = AdminRepository::new();
    
    let menus = repo.get_all_menus().await?;
    
    let mut created_permissions = Vec::new();
    
    for menu in menus {
        let permissions = vec![
            (format!("访问{}", menu.name), "view"),
            (format!("创建{}", menu.name), "create"),
            (format!("编辑{}", menu.name), "edit"),
            (format!("删除{}", menu.name), "delete"),
        ];
        
        for (name, action) in permissions {
            let permission = repo.create_permission(
                menu.id,
                name,
                Some(action.to_string()),
            ).await?;
            created_permissions.push(permission);
        }
    }
    
    Ok(Json(created_permissions))
}

pub async fn assign_super_admin_all_permissions() -> ServiceResult<Json<()>> {
    let repo = AdminRepository::new();
    
    let role = repo.init_super_admin_role().await?;
    let permissions = repo.get_all_permissions().await?;
    
    let permission_ids: Vec<i64> = permissions.iter().map(|p| p.id).collect();
    
    repo.assign_permissions_to_role(role.id, permission_ids).await?;
    
    Ok(Json(()))
}
