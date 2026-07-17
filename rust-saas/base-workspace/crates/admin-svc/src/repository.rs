use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{
    Department, Menu, Permission, Personnel, 
    // PersonnelDepartment, 
    // PersonnelRole, 
    Role,
    // RolePermission,
};
use shared::schema::{
    departments, menus, permissions, personnel, personnel_departments, personnel_roles,
    role_permissions, roles,
};
use std::env;

pub struct AdminRepository {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl AdminRepository {
    pub fn new() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        AdminRepository { pool }
    }

    pub async fn init_super_admin_role(&self) -> ServiceResult<Role> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let role = diesel::insert_into(roles::table)
            .values((
                roles::name.eq("超级管理员"),
                roles::description.eq("拥有所有权限的超级管理员角色"),
                roles::is_super_admin.eq(true),
                roles::created_at.eq(now),
                roles::updated_at.eq(now),
            ))
            .returning(Role::as_select())
            .get_result(&mut conn)?;

        Ok(role)
    }

    pub async fn create_personnel(
        &self,
        user_id: Option<i64>,
        name: String,
        gender: Option<String>,
        email: Option<String>,
        wechat: Option<String>,
        phone: Option<String>,
    ) -> ServiceResult<Personnel> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let personnel = diesel::insert_into(personnel::table)
            .values((
                personnel::user_id.eq(user_id),
                personnel::name.eq(name),
                personnel::gender.eq(gender),
                personnel::email.eq(email),
                personnel::wechat.eq(wechat),
                personnel::phone.eq(phone),
                personnel::created_at.eq(now),
                personnel::updated_at.eq(now),
            ))
            .returning(Personnel::as_select())
            .get_result(&mut conn)?;

        Ok(personnel)
    }

    pub async fn get_personnel_by_user_id(&self, user_id: i64) -> ServiceResult<Option<Personnel>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let personnel = personnel::table
            .filter(personnel::user_id.eq(user_id))
            .first::<Personnel>(&mut conn)
            .optional()?;

        Ok(personnel)
    }

    pub async fn get_all_personnel(&self) -> ServiceResult<Vec<Personnel>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let personnel_list = personnel::table
            .order(personnel::created_at.desc())
            .load::<Personnel>(&mut conn)?;

        Ok(personnel_list)
    }

    pub async fn get_personnel(&self, id: i64) -> ServiceResult<Personnel> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let personnel = personnel::table
            .filter(personnel::id.eq(id))
            .first::<Personnel>(&mut conn)?;

        Ok(personnel)
    }

    pub async fn update_personnel(
        &self,
        id: i64,
        name: Option<String>,
        gender: Option<String>,
        email: Option<String>,
        wechat: Option<String>,
        phone: Option<String>,
    ) -> ServiceResult<Personnel> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let existing = personnel::table.filter(personnel::id.eq(id)).first::<Personnel>(&mut conn)?;

        let name = name.unwrap_or(existing.name);

        let personnel = diesel::update(personnel::table.filter(personnel::id.eq(id)))
            .set((
                personnel::name.eq(name),
                personnel::gender.eq(gender),
                personnel::email.eq(email),
                personnel::wechat.eq(wechat),
                personnel::phone.eq(phone),
                personnel::updated_at.eq(now),
            ))
            .returning(Personnel::as_select())
            .get_result(&mut conn)?;

        Ok(personnel)
    }

    pub async fn update_last_login_date(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        diesel::update(personnel::table.filter(personnel::id.eq(id)))
            .set(personnel::last_login_date.eq(now))
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn create_department(
        &self,
        name: String,
        parent_id: Option<i64>,
        description: Option<String>,
    ) -> ServiceResult<Department> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let department = diesel::insert_into(departments::table)
            .values((
                departments::name.eq(name),
                departments::parent_id.eq(parent_id),
                departments::description.eq(description),
                departments::created_at.eq(now),
                departments::updated_at.eq(now),
            ))
            .returning(Department::as_select())
            .get_result(&mut conn)?;

        Ok(department)
    }

    pub async fn get_all_departments(&self) -> ServiceResult<Vec<Department>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let department_list = departments::table
            .order(departments::created_at.desc())
            .load::<Department>(&mut conn)?;

        Ok(department_list)
    }

    pub async fn get_department(&self, id: i64) -> ServiceResult<Department> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let department = departments::table
            .filter(departments::id.eq(id))
            .first::<Department>(&mut conn)?;

        Ok(department)
    }

    pub async fn update_department(
        &self,
        id: i64,
        name: Option<String>,
        parent_id: Option<i64>,
        description: Option<String>,
    ) -> ServiceResult<Department> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let existing = departments::table.filter(departments::id.eq(id)).first::<Department>(&mut conn)?;

        let name = name.unwrap_or(existing.name);

        let department = diesel::update(departments::table.filter(departments::id.eq(id)))
            .set((
                departments::name.eq(name),
                departments::parent_id.eq(parent_id),
                departments::description.eq(description),
                departments::updated_at.eq(now),
            ))
            .returning(Department::as_select())
            .get_result(&mut conn)?;

        Ok(department)
    }

    pub async fn delete_department(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        diesel::delete(departments::table.filter(departments::id.eq(id))).execute(&mut conn)?;

        Ok(())
    }

    pub async fn assign_departments(
        &self,
        personnel_id: Option<i64>,
        department_ids: Vec<i64>,
    ) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let personnel_id = personnel_id.unwrap_or(0);
        if personnel_id == 0 {
            return Err(ServiceError::InvalidInput("personnel_id is required".to_string()));
        }
        if(department_ids.is_empty()) {
            return Err(ServiceError::InvalidInput("department_ids is required".to_string()));
        }


        // Delete all existing departments for this personnel
        diesel::delete(
            personnel_departments::table.filter(personnel_departments::personnel_id.eq(personnel_id)),
        )
        .execute(&mut conn)?;

        for department_id in department_ids {
            diesel::insert_into(personnel_departments::table)
                .values((
                    personnel_departments::personnel_id.eq(personnel_id),
                    personnel_departments::department_id.eq(department_id),
                    personnel_departments::created_at.eq(now),
                ))
                .execute(&mut conn)?;
        }

        Ok(())
    }

    pub async fn get_personnel_departments(&self, personnel_id: i64) -> ServiceResult<Vec<Department>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let department_ids = personnel_departments::table
            .filter(personnel_departments::personnel_id.eq(personnel_id))
            .select(personnel_departments::department_id)
            .load::<i64>(&mut conn)?;

        let department_list = departments::table
            .filter(departments::id.eq_any(department_ids))
            .load::<Department>(&mut conn)?;

        Ok(department_list)
    }

    pub async fn create_menu(
        &self,
        name: String,
        path: Option<String>,
        parent_id: Option<i64>,
        icon: Option<String>,
        sort_order: i32,
    ) -> ServiceResult<Menu> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let menu = diesel::insert_into(menus::table)
            .values((
                menus::name.eq(name),
                menus::path.eq(path),
                menus::parent_id.eq(parent_id),
                menus::icon.eq(icon),
                menus::sort_order.eq(sort_order),
                menus::created_at.eq(now),
                menus::updated_at.eq(now),
            ))
            .returning(Menu::as_select())
            .get_result(&mut conn)?;

        Ok(menu)
    }

    pub async fn get_all_menus(&self) -> ServiceResult<Vec<Menu>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let menu_list = menus::table
            .order(menus::sort_order.asc())
            .load::<Menu>(&mut conn)?;

        Ok(menu_list)
    }

    pub async fn get_menu(&self, id: i64) -> ServiceResult<Menu> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let menu = menus::table.filter(menus::id.eq(id)).first::<Menu>(&mut conn)?;

        Ok(menu)
    }

    pub async fn update_menu(
        &self,
        id: i64,
        name: Option<String>,
        path: Option<String>,
        parent_id: Option<i64>,
        icon: Option<String>,
        sort_order: Option<i32>,
    ) -> ServiceResult<Menu> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let existing = menus::table.filter(menus::id.eq(id)).first::<Menu>(&mut conn)?;

        let name = name.unwrap_or(existing.name);
        let sort_order = sort_order.unwrap_or(existing.sort_order);

        let menu = diesel::update(menus::table.filter(menus::id.eq(id)))
            .set((
                menus::name.eq(name),
                menus::path.eq(path),
                menus::parent_id.eq(parent_id),
                menus::icon.eq(icon),
                menus::sort_order.eq(sort_order),
                menus::updated_at.eq(now),
            ))
            .returning(Menu::as_select())
            .get_result(&mut conn)?;

        Ok(menu)
    }

    pub async fn delete_menu(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        diesel::delete(menus::table.filter(menus::id.eq(id))).execute(&mut conn)?;

        Ok(())
    }

    pub async fn create_permission(
        &self,
        menu_id: i64,
        name: String,
        description: Option<String>,
    ) -> ServiceResult<Permission> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let permission = diesel::insert_into(permissions::table)
            .values((
                permissions::menu_id.eq(menu_id),
                permissions::name.eq(name),
                permissions::description.eq(description),
                permissions::created_at.eq(now),
            ))
            .returning(Permission::as_select())
            .get_result(&mut conn)?;

        Ok(permission)
    }

    pub async fn get_all_permissions(&self) -> ServiceResult<Vec<Permission>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let permission_list = permissions::table
            .order(permissions::created_at.desc())
            .load::<Permission>(&mut conn)?;

        Ok(permission_list)
    }

    pub async fn get_permissions_by_menu(&self, menu_id: i64) -> ServiceResult<Vec<Permission>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let permission_list = permissions::table
            .filter(permissions::menu_id.eq(menu_id))
            .load::<Permission>(&mut conn)?;

        Ok(permission_list)
    }

    pub async fn delete_permission(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        diesel::delete(permissions::table.filter(permissions::id.eq(id))).execute(&mut conn)?;

        Ok(())
    }

    pub async fn create_role(
        &self,
        name: String,
        description: Option<String>,
        is_super_admin: bool,
    ) -> ServiceResult<Role> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let role = diesel::insert_into(roles::table)
            .values((
                roles::name.eq(name),
                roles::description.eq(description),
                roles::is_super_admin.eq(is_super_admin),
                roles::created_at.eq(now),
                roles::updated_at.eq(now),
            ))
            .returning(Role::as_select())
            .get_result(&mut conn)?;

        Ok(role)
    }

    pub async fn get_all_roles(&self) -> ServiceResult<Vec<Role>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let role_list = roles::table
            .order(roles::created_at.desc())
            .load::<Role>(&mut conn)?;

        Ok(role_list)
    }

    pub async fn get_role(&self, id: i64) -> ServiceResult<Role> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let role = roles::table.filter(roles::id.eq(id)).first::<Role>(&mut conn)?;

        Ok(role)
    }

    pub async fn update_role(
        &self,
        id: i64,
        name: Option<String>,
        description: Option<String>,
        is_super_admin: Option<bool>,
    ) -> ServiceResult<Role> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        let existing = roles::table.filter(roles::id.eq(id)).first::<Role>(&mut conn)?;

        let name = name.unwrap_or(existing.name);
        let is_super_admin = is_super_admin.unwrap_or(existing.is_super_admin);

        let role = diesel::update(roles::table.filter(roles::id.eq(id)))
            .set((
                roles::name.eq(name),
                roles::description.eq(description),
                roles::is_super_admin.eq(is_super_admin),
                roles::updated_at.eq(now),
            ))
            .returning(Role::as_select())
            .get_result(&mut conn)?;

        Ok(role)
    }

    pub async fn delete_role(&self, id: i64) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        diesel::delete(role_permissions::table.filter(role_permissions::role_id.eq(id)))
            .execute(&mut conn)?;

        diesel::delete(personnel_roles::table.filter(personnel_roles::role_id.eq(id)))
            .execute(&mut conn)?;

        diesel::delete(roles::table.filter(roles::id.eq(id))).execute(&mut conn)?;

        Ok(())
    }

    pub async fn assign_permissions_to_role(
        &self,
        role_id: i64,
        permission_ids: Vec<i64>,
    ) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        diesel::delete(role_permissions::table.filter(role_permissions::role_id.eq(role_id)))
            .execute(&mut conn)?;

        for permission_id in permission_ids {
            diesel::insert_into(role_permissions::table)
                .values((
                    role_permissions::role_id.eq(role_id),
                    role_permissions::permission_id.eq(permission_id),
                    role_permissions::created_at.eq(now),
                ))
                .execute(&mut conn)?;
        }

        Ok(())
    }

    pub async fn get_role_permissions(&self, role_id: i64) -> ServiceResult<Vec<Permission>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let permission_ids = role_permissions::table
            .filter(role_permissions::role_id.eq(role_id))
            .select(role_permissions::permission_id)
            .load::<i64>(&mut conn)?;

        let permission_list = permissions::table
            .filter(permissions::id.eq_any(permission_ids))
            .load::<Permission>(&mut conn)?;

        Ok(permission_list)
    }

    pub async fn assign_roles_to_personnel(
        &self,
        personnel_id: i64,
        role_ids: Vec<i64>,
    ) -> ServiceResult<()> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        let now = Utc::now().naive_utc();

        diesel::delete(
            personnel_roles::table.filter(personnel_roles::personnel_id.eq(personnel_id)),
        )
        .execute(&mut conn)?;

        for role_id in role_ids {
            diesel::insert_into(personnel_roles::table)
                .values((
                    personnel_roles::personnel_id.eq(personnel_id),
                    personnel_roles::role_id.eq(role_id),
                    personnel_roles::created_at.eq(now),
                ))
                .execute(&mut conn)?;
        }

        Ok(())
    }

    pub async fn get_personnel_roles(&self, personnel_id: i64) -> ServiceResult<Vec<Role>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let role_ids = personnel_roles::table
            .filter(personnel_roles::personnel_id.eq(personnel_id))
            .select(personnel_roles::role_id)
            .load::<i64>(&mut conn)?;

        let role_list = roles::table
            .filter(roles::id.eq_any(role_ids))
            .load::<Role>(&mut conn)?;

        Ok(role_list)
    }

    pub async fn get_personnel_permissions(&self, personnel_id: i64) -> ServiceResult<Vec<Permission>> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let role_ids = personnel_roles::table
            .filter(personnel_roles::personnel_id.eq(personnel_id))
            .select(personnel_roles::role_id)
            .load::<i64>(&mut conn)?;

        let roles = roles::table
            .filter(roles::id.eq_any(role_ids.clone()))
            .load::<Role>(&mut conn)?;

        if roles.iter().any(|r| r.is_super_admin) {
            return permissions::table
                .load::<Permission>(&mut conn)
                .map_err(|e| ServiceError::DatabaseError(e.to_string()));
        }

        let permission_ids = role_permissions::table
            .filter(role_permissions::role_id.eq_any(role_ids))
            .select(role_permissions::permission_id)
            .load::<i64>(&mut conn)?;

        let permission_list = permissions::table
            .filter(permissions::id.eq_any(permission_ids))
            .load::<Permission>(&mut conn)?;

        Ok(permission_list)
    }

    pub async fn is_super_admin(&self, personnel_id: i64) -> ServiceResult<bool> {
        let mut conn = self.pool.get().map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let role_ids = personnel_roles::table
            .filter(personnel_roles::personnel_id.eq(personnel_id))
            .select(personnel_roles::role_id)
            .load::<i64>(&mut conn)?;

        let exists = roles::table
            .filter(roles::id.eq_any(role_ids))
            .filter(roles::is_super_admin.eq(true))
            .select(roles::id)
            .first::<i64>(&mut conn)
            .optional()?;

        Ok(exists.is_some())
    }
}
