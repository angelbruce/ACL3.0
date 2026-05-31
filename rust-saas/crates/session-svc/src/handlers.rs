use axum::{extract::Path, Json};
use shared::errors::{ServiceError, ServiceResult};
use shared::models::{Session, SessionWithNames, SessionItem, CreateSessionRequest, AddMessageRequest, UpdateSessionRequest};
use crate::repository::SessionRepository;

pub async fn get_sessions() -> ServiceResult<Json<Vec<SessionWithNames>>> {
    let repo = SessionRepository::new();
    let sessions = repo.get_all_sessions().await?;
    Ok(Json(sessions))
}

pub async fn get_session(Path(id): Path<i64>) -> ServiceResult<Json<SessionWithNames>> {
    let repo = SessionRepository::new();
    let session = repo.get_session(id).await?;
    Ok(Json(session))
}

pub async fn create_session(Json(req): Json<CreateSessionRequest>) -> ServiceResult<Json<Session>> {
    let repo = SessionRepository::new();
    let session = repo.create_session(req).await?;
    Ok(Json(session))
}

pub async fn update_session(Path(id): Path<i64>, Json(req): Json<UpdateSessionRequest>) -> ServiceResult<Json<SessionWithNames>> {
    let repo = SessionRepository::new();
    let _ = repo.update_session(id, req).await?;
    let session = repo.get_session(id).await?;
    Ok(Json(session))
}

pub async fn delete_session(Path(id): Path<i64>) -> ServiceResult<Json<()>> {
    let repo = SessionRepository::new();
    repo.delete_session(id).await?;
    Ok(Json(()))
}

pub async fn get_session_messages(Path(id): Path<i64>) -> ServiceResult<Json<Vec<SessionItem>>> {
    let repo = SessionRepository::new();
    let messages = repo.get_session_messages(id).await?;
    Ok(Json(messages))
}

pub async fn add_message(Path(id): Path<i64>, Json(req): Json<AddMessageRequest>) -> ServiceResult<Json<SessionItem>> {
    let repo = SessionRepository::new();
    let message = repo.add_message(id, req).await?;
    Ok(Json(message))
}