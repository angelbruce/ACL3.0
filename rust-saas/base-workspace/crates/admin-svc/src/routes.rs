use axum::routing::{delete, get, post, put};
use axum::Router;
use crate::handlers::*;
use shared::middleware::auth_middleware;

pub fn create_router() -> Router {
    Router::new()
        .route("/init-super-admin", post(init_super_admin))
        .route("/init-menus", post(init_default_menus))
        .route("/init-permissions", post(init_default_permissions))
        .route("/init-super-admin-all", post(assign_super_admin_all_permissions))
        
        .route("/personnel", get(get_all_personnel))
        .route("/personnel", post(create_personnel))
        .route("/personnel/:id", get(get_personnel))
        .route("/personnel/:id", put(update_personnel))
        .route("/personnel/:id/details", get(get_personnel_with_details))
        .route("/personnel/:personnel_id/assign-super-admin", post(assign_super_admin_role))
        
        .route("/personnel/:personnel_id/departments", get(get_personnel_departments))
        .route("/personnel/:personnel_id/roles", get(get_personnel_roles))
        .route("/personnel/:personnel_id/permissions", get(get_personnel_permissions))
        .route("/personnel/:personnel_id/assign-departments", post(assign_departments))
        .route("/personnel/:personnel_id/assign-roles", post(assign_roles_to_personnel))
        
        .route("/departments", get(get_all_departments))
        .route("/departments", post(create_department))
        .route("/departments/:id", get(get_department))
        .route("/departments/:id", put(update_department))
        .route("/departments/:id", delete(delete_department))
        
        .route("/menus", get(get_all_menus))
        .route("/menus", post(create_menu))
        .route("/menus/:id", get(get_menu))
        .route("/menus/:id", put(update_menu))
        .route("/menus/:id", delete(delete_menu))
        
        .route("/permissions", get(get_all_permissions))
        .route("/permissions", post(create_permission))
        .route("/permissions/by-menu/:menu_id", get(get_permissions_by_menu))
        .route("/permissions/:id", delete(delete_permission))
        
        .route("/roles", get(get_all_roles))
        .route("/roles", post(create_role))
        .route("/roles/:id", get(get_role))
        .route("/roles/:id", put(update_role))
        .route("/roles/:id", delete(delete_role))
        .route("/roles/:role_id/permissions", get(get_role_permissions))
        .route("/roles/:role_id/permissions", post(assign_permissions_to_role))
        
        .route("/personnel/:personnel_id/is-super-admin", get(check_super_admin))
        .layer(axum::middleware::from_fn(auth_middleware))
}
