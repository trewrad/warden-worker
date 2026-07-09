use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Worker error: {0}")]
    Worker(#[from] worker::Error),

    #[error("Database query failed")]
    Database,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Cryptography error: {0}")]
    Crypto(String),

    #[error("Internal server error")]
    Internal,

    #[error("Two factor authentication required")]
    TwoFactorRequired(Value),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::TwoFactorRequired(json_body) => {
                // Return 400 Bad Request with the 2FA required JSON response as expected by clients
                (StatusCode::BAD_REQUEST, Json(json_body)).into_response()
            }
            other => {
                let (status, error_message) = match &other {
                    AppError::Worker(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Worker error: {}", e),
                    ),
                    AppError::Database => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error".to_string(),
                    ),
                    AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
                    AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
                    AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
                    AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone()),
                    AppError::Crypto(msg) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Crypto error: {}", msg),
                    ),
                    AppError::Internal => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    ),
                    AppError::TwoFactorRequired(_) => unreachable!(),
                };

                if status.is_server_error() {
                    log::error!("500 Error Generated: {:?}", other);
                }

                let body = Json(json!({ "error": error_message }));
                (status, body).into_response()
            }
        }
    }
}
