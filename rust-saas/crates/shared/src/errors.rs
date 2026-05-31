use axum::{response::IntoResponse, http::{StatusCode, Response}, Json};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Record not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("LLM service error: {0}")]
    LlmError(String),

    #[error("MCP service error: {0}")]
    McpError(String),

    #[error("Internal server error")]
    InternalError,

    #[error("Invalid JWT token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match self {
            ServiceError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            ServiceError::NotFound => StatusCode::NOT_FOUND,
            ServiceError::Unauthorized => StatusCode::UNAUTHORIZED,
            ServiceError::Forbidden => StatusCode::FORBIDDEN,
            ServiceError::Conflict(_) => StatusCode::CONFLICT,
            ServiceError::LlmError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::McpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::InvalidToken => StatusCode::UNAUTHORIZED,
            ServiceError::TokenExpired => StatusCode::UNAUTHORIZED,
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
        };

        let body = serde_json::json!({
            "error": status.as_u16(),
            "message": self.to_string()
        });

        (status, Json(body)).into_response()
    }
}

impl From<diesel::result::Error> for ServiceError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => ServiceError::NotFound,
            _ => ServiceError::DatabaseError(e.to_string()),
        }
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(e: serde_json::Error) -> Self {
        ServiceError::InvalidInput(e.to_string())
    }
}

impl From<uuid::Error> for ServiceError {
    fn from(e: uuid::Error) -> Self {
        ServiceError::InvalidInput(e.to_string())
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;