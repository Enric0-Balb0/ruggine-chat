use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("User is not active")]
    UserNotActive,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let status_code = match self {
            UserError::UserNotFound => StatusCode::NOT_FOUND,
            UserError::UserAlreadyExists => StatusCode::CONFLICT,
            UserError::InvalidPassword => StatusCode::UNAUTHORIZED,
            UserError::UserNotActive => StatusCode::FORBIDDEN,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
