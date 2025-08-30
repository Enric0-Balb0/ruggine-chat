use crate::error::group_chat_error::GroupChatError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::invitation_error::InvitationError;
use crate::error::text_message_error::TextMessageError;
use crate::error::{db_error::DbError, token_error::TokenError, user_error::UserError};
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use crate::websocket::message::WsError;

#[derive(Error, Debug, Clone)]
pub enum WebSocketError {
    #[error(transparent)]
    GroupChatError(#[from] GroupChatError),

    #[error("Failed to serialize WebSocket message")]
    SerializationError,
}

impl From<WebSocketError> for WsError {
    fn from(err: WebSocketError) -> Self {
        match err {
            WebSocketError::GroupChatError(error) => error.into(),
            WebSocketError::SerializationError => WsError {
                code: 500,
                message: "Failed to serialize WebSocket message".to_string(),
            },
        }
    }
}
