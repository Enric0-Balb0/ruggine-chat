use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextMessageError {
    #[error("Group chat not found")]
    GroupChatNotFound,
    #[error("Message not found")]
    MessageNotFound,
    #[error("User can not access messages")]
    UserCannotAccessMessages,
}

impl IntoResponse for TextMessageError {
    fn into_response(self) -> Response {
        let status_code = match self {
            TextMessageError::GroupChatNotFound => StatusCode::NOT_FOUND,
            TextMessageError::MessageNotFound => StatusCode::NOT_FOUND,
            TextMessageError::UserCannotAccessMessages => StatusCode::FORBIDDEN,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
