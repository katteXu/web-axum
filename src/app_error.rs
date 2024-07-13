use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// 应用错误
pub struct AppError(pub anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let msg = format!("服务器异常，{}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

/// 授权错误
#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "missing credentials"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "invalid token"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "token create error"),
        };

        let body = Json(json!({
            "error":error_message
        }));

        (status, body).into_response()
    }
}
