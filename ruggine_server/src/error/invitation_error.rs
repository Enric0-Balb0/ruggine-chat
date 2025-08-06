use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvitationError {
    #[error("Invitation not found")]
    InvitationNotFound,
    #[error("Invited user not found")]
    InvitedUserNotFound,
    #[error("Invitation already responded")]
    InvitationAlreadyResponded,
    #[error("Already an invitation pending")]
    AlreadyInvitationPending,
    #[error("User already in the group")]
    UserAlreadyInGroup,
    #[error("User not authorized")]
    UserNotAuthorized,
    #[error("Invalid status: {0}")]
    InvalidStatus(String),
}

impl IntoResponse for InvitationError {
    fn into_response(self) -> Response {
        let status_code = match self {
            InvitationError::InvitationNotFound => StatusCode::NOT_FOUND,
            InvitationError::InvitedUserNotFound => StatusCode::UNPROCESSABLE_ENTITY,
            InvitationError::InvitationAlreadyResponded => StatusCode::CONFLICT,
            InvitationError::AlreadyInvitationPending => StatusCode::CONFLICT,
            InvitationError::UserAlreadyInGroup => StatusCode::CONFLICT,
            InvitationError::UserNotAuthorized => StatusCode::FORBIDDEN,
            InvitationError::InvalidStatus(_) => StatusCode::BAD_REQUEST,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
