use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GroupMembershipError {
    #[error("Group membership not found")]
    GroupMembershipNotFound,
    #[error("Invitation not owned by user")]
    InvitationNotOwnedByUser,
    #[error("Invitation not found")]
    InvitationNotFound,
    #[error("Group not found")]
    GroupNotFound,
    #[error("User already in the group")]
    UserAlreadyInGroup,
    #[error("User already left the group")]
    UserAlreadyLeftGroup,
}

impl IntoResponse for GroupMembershipError {
    fn into_response(self) -> Response {
        let status_code = match self {
            GroupMembershipError::GroupMembershipNotFound => StatusCode::NOT_FOUND,
            GroupMembershipError::InvitationNotOwnedByUser => StatusCode::FORBIDDEN,
            GroupMembershipError::InvitationNotFound => StatusCode::NOT_FOUND,
            GroupMembershipError::GroupNotFound => StatusCode::UNPROCESSABLE_ENTITY,
            GroupMembershipError::UserAlreadyInGroup => StatusCode::CONFLICT,
            GroupMembershipError::UserAlreadyLeftGroup => StatusCode::CONFLICT,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
