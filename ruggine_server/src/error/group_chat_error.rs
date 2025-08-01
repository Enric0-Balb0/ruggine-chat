use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GroupChatError {
    #[error("GroupChat not found")]
    GroupChatNotFound,
    #[error("No fields to update")]
    GroupChatNoFieldsToUpdate,
    #[error("User not authorized for this operation")]
    UserNotAuthorized,
}

impl IntoResponse for GroupChatError {
    fn into_response(self) -> Response {
        let status_code = match self {
            GroupChatError::GroupChatNotFound => StatusCode::NOT_FOUND,
            GroupChatError::GroupChatNoFieldsToUpdate => StatusCode::BAD_REQUEST,
            GroupChatError::UserNotAuthorized => StatusCode::FORBIDDEN,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
