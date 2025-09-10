use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    #[error("Invalid password")]
    InvalidPassword,
    #[error("User is not active")]
    UserNotActive,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("No fields to update")]
    NoFieldsToUpdate,
    #[error("Invalid birthday: {0}")]
    InvalidBirthday(String),
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let status_code = match self {
            UserError::UserNotFound => StatusCode::NOT_FOUND,
            UserError::UserAlreadyExists(_) => StatusCode::CONFLICT,
            UserError::InvalidPassword => StatusCode::UNAUTHORIZED,
            UserError::UserNotActive => StatusCode::FORBIDDEN,
            UserError::InsufficientPermissions => StatusCode::FORBIDDEN,
            UserError::NoFieldsToUpdate => StatusCode::BAD_REQUEST,
            UserError::InvalidBirthday(_) => StatusCode::BAD_REQUEST,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
