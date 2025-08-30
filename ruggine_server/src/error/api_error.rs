use crate::error::{db_error::DbError,token_error::TokenError, user_error::UserError };
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use crate::error::cpu_usage_log_error::CpuUsageLogError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::invitation_error::InvitationError;
use crate::error::text_message_error::TextMessageError;

#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error(transparent)]
    TokenError(#[from] TokenError),
    #[error(transparent)]
    UserError(#[from] UserError),
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error(transparent)]
    GroupChatError(#[from] GroupChatError),
    #[error(transparent)]
    InvitationError(#[from] InvitationError),
    #[error(transparent)]
    GroupMembershipError(#[from] GroupMembershipError),
    #[error(transparent)]
    TextMessageError(#[from] TextMessageError),
    #[error(transparent)]
    CpuUsageLogError(#[from] CpuUsageLogError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::TokenError(error) => error.into_response(),
            ApiError::UserError(error) => error.into_response(),
            ApiError::DbError(error) => error.into_response(),
            ApiError::GroupChatError(error) => error.into_response(),
            ApiError::InvitationError(error) => error.into_response(),
            ApiError::GroupMembershipError(error) => error.into_response(),
            ApiError::TextMessageError(error) => error.into_response(),
            ApiError::CpuUsageLogError(error) => error.into_response(),
        }
    }
}