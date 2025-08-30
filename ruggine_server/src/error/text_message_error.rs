use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TextMessageError {
    #[error("Group chat not found")]
    GroupChatNotFound,
    #[error("Message not found")]
    MessageNotFound,
    #[error("User can not access messages")]
    UserCannotAccessMessages,
    #[error("User is not the creator of the message")]
    UserIsNotCreator,
    #[error("Cannot set read at before sent at")]
    CannotSetReadAtBeforeSentAt,
    #[error("Read at must be greater or equals to sent at or lower or equals to now")]
    ReadAtMustBeGreaterOrEqualsToSentAtAndLowerOrEqualsNow,
    #[error("Sent at must be less or equals to now and greater than message creation")]
    SentAtMustBeLessOrEqualsToNowAndGreaterThanMessageCreation,
    #[error("Message info not found")]
    MessageInfoNotFound,
    #[error("Cannot update read at again")]
    CannotUpdateReadAtAgain,
    #[error("Cannot update sent at again")]
    CannotUpdateSentAtAgain,
}

impl IntoResponse for TextMessageError {
    fn into_response(self) -> Response {
        let status_code = match self {
            TextMessageError::GroupChatNotFound => StatusCode::NOT_FOUND,
            TextMessageError::MessageNotFound => StatusCode::NOT_FOUND,
            TextMessageError::UserCannotAccessMessages => StatusCode::FORBIDDEN,
            TextMessageError::UserIsNotCreator => StatusCode::FORBIDDEN,
            TextMessageError::CannotSetReadAtBeforeSentAt => StatusCode::BAD_REQUEST,
            TextMessageError::ReadAtMustBeGreaterOrEqualsToSentAtAndLowerOrEqualsNow => StatusCode::BAD_REQUEST,
            TextMessageError::SentAtMustBeLessOrEqualsToNowAndGreaterThanMessageCreation => StatusCode::BAD_REQUEST,
            TextMessageError::MessageInfoNotFound => StatusCode::NOT_FOUND,
            TextMessageError::CannotUpdateReadAtAgain => StatusCode::BAD_REQUEST,
            TextMessageError::CannotUpdateSentAtAgain => StatusCode::BAD_REQUEST,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
