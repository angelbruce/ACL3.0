use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use crate::handlers::*;
use shared::middleware::auth_middleware;

pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/projects", axum::routing::get(list_projects))
        .route("/api/projects", axum::routing::post(create_project))
        .route("/api/projects/:id", axum::routing::get(get_project))
        .route("/api/projects/:id", axum::routing::put(update_project))
        .route("/api/projects/:id", axum::routing::delete(delete_project))
        .route("/api/projects/:id/files", axum::routing::get(list_project_files))
        .route("/api/projects/:id/files", axum::routing::post(create_project_file))
        .route("/api/projects/:id/messages", axum::routing::get(get_project_messages))
        .route("/api/projects/:id/messages", axum::routing::post(add_project_message))
        .route("/api/projects/:id/summaries", axum::routing::get(get_project_summaries))
        .route("/api/projects/:id/summaries", axum::routing::post(create_or_update_project_summary))
        .route("/api/project-files/:id", axum::routing::put(update_project_file))
        .route("/api/project-files/:id", axum::routing::delete(delete_project_file))
        .route("/api/workspace/files", axum::routing::get(list_workspace_files))
        .route("/api/workspace/projects", axum::routing::get(list_projects))
        .route("/api/workspace/projects", axum::routing::post(create_project))
        .route("/api/workspace/projects/:project_name", axum::routing::delete(delete_project))
        .route("/api/workspace/projects/:project_name/files", axum::routing::get(list_project_workspace_files))
        .route("/api/workspace/files/*file_path", axum::routing::get(download_file))
        .route("/api/workspace/files/*file_path", axum::routing::delete(delete_file))
        .route("/api/kanban/boards", axum::routing::get(get_public_kanban_boards))
        .route("/api/kanban/boards", axum::routing::post(create_kanban_board))
        .route("/api/kanban/boards/:board_id", axum::routing::get(get_kanban_board))
        .route("/api/kanban/boards/:board_id", axum::routing::put(update_kanban_board))
        .route("/api/kanban/boards/:board_id", axum::routing::delete(delete_kanban_board))
        .route("/api/kanban/boards/:board_id/subscribe", axum::routing::post(subscribe_board))
        .route("/api/kanban/boards/:board_id/unsubscribe", axum::routing::post(unsubscribe_board))
        .route("/api/kanban/boards/:board_id/files", axum::routing::post(share_file_to_board))
        .route("/api/kanban/items/:item_id", axum::routing::delete(remove_file_from_board))
        .route("/api/kanban/subscriptions", axum::routing::get(get_subscribed_boards))
        .route("/api/kanban/boards/:board_id/files/*file_path", axum::routing::get(download_shared_file))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(cors)
}
