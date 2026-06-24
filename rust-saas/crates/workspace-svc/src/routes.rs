use axum::Router;
use std::env;
use tower_http::cors::{Any, CorsLayer};
use crate::handlers::*;
use shared::middleware::auth_middleware;
use tower_http::services::{ServeDir, ServeFile};


pub fn create_router() -> Router {
    let root_path = env::var("WORKSPACE_ROOT").unwrap_or_else(|_| "./workspace_storage".to_string());
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
        .route("/api/projects-files/voice/:id", axum::routing::post(get_project_file_voice))
        .route("/api/projects-files/voice/link/:id", axum::routing::post(get_project_file_voice_link))
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
        .route("/api/project-container-configs/:project_id", axum::routing::get(get_project_container_config))
        .route("/api/project-container-configs/:project_id", axum::routing::post(save_project_container_config))
        .route("/api/project-container-configs/:project_id/start", axum::routing::post(start_container))
        .route("/api/project-container-configs/:project_id/stop", axum::routing::post(stop_container))
        .route("/api/project-container-configs/:project_id/status", axum::routing::get(get_container_status))
        .route("/api/project-container-configs/:project_id/logs", axum::routing::post(get_container_logs))
        .route("/api/project-container-configs/:project_id/cleanup", axum::routing::post(cleanup_container))
        .route("/api/projects/:id/refresh-file", axum::routing::post(refresh_project_file_to_container))
        .route("/api/projects/execute-command", axum::routing::post(execute_command))
        .route("/api/projects/execute-command-stream", axum::routing::post(execute_command_stream_handler))
        .route("/api/projects/:project_id/messages/:message_id", axum::routing::delete(delete_project_message))
        .route("/api/chat/stream", axum::routing::post(workspace_chat_stream))
        .layer(axum::middleware::from_fn(auth_middleware))
        .nest_service("/voice", ServeDir::new(root_path))
        .layer(cors)
}
