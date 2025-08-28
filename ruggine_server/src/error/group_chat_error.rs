use crate::{response::api_response::ApiErrorResponse, websocket::message::WsError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum GroupChatError {
    #[error("GroupChat not found")]
    GroupChatNotFound,
    #[error("No fields to update")]
    GroupChatNoFieldsToUpdate,
    #[error("User not authorized for this operation")]
    UserNotAuthorized,
    #[error("Something went wrong: {0}")]
    SomethingWentWrong(String),
}

impl IntoResponse for GroupChatError {
    fn into_response(self) -> Response {
        let status_code = match self {
            GroupChatError::GroupChatNotFound => StatusCode::NOT_FOUND,
            GroupChatError::GroupChatNoFieldsToUpdate => StatusCode::BAD_REQUEST,
            GroupChatError::UserNotAuthorized => StatusCode::FORBIDDEN,
            GroupChatError::SomethingWentWrong(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}

impl From<GroupChatError> for WsError {
    fn from(err: GroupChatError) -> Self {
        match err {
            GroupChatError::GroupChatNotFound => WsError { code: 404, message: err.to_string() },
            GroupChatError::UserNotAuthorized => WsError { code: 403, message: err.to_string() },
            e => {
                error!("Parsing unexpected error for WsError: {:?}", e);
                WsError { code: 500, message: "Unexpected group chat error".into() }
            } ,
        }
    }
}
