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
    #[error("Pending invitation not found")]
    PendingInvitationNotFound,
    #[error("Group not found")]
    GroupNotFound,
    #[error("User already in the group")]
    UserAlreadyInGroup,
}

impl IntoResponse for GroupMembershipError {
    fn into_response(self) -> Response {
        let status_code = match self {
            GroupMembershipError::GroupMembershipNotFound => StatusCode::NOT_FOUND,
            GroupMembershipError::PendingInvitationNotFound => StatusCode::NOT_FOUND,
            GroupMembershipError::GroupNotFound => StatusCode::UNPROCESSABLE_ENTITY,
            GroupMembershipError::UserAlreadyInGroup => StatusCode::CONFLICT,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
