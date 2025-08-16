use async_trait::async_trait;
use crate::dto::text_message_dto::{TextMessageReadDto, TextMessageCreateDto};
use crate::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use crate::error::api_error::ApiError;
use crate::response::paginated_response::PaginatedResponse;
use mockall::automock;
use crate::response::PaginatedTextMessageResponse;

#[automock]
#[async_trait]
pub trait TextMessageServiceTrait: Send + Sync {
    /// Find a text message by its ID
    async fn find_by_id(&self, id: i32) -> Result<TextMessageReadDto, ApiError>;
    
    /// Find text messages by group chat ID with pagination
    /// Verifies that the auth_user_id has membership in the group
    async fn find_by_group_chat_id_paginated(
        &self, 
        group_chat_id: i32,
        auth_user_id: i32,
        pagination_query: TextMessagePaginationQuery,
    ) -> Result<PaginatedTextMessageResponse, ApiError>;
    
    /// Create a new text message
    /// Verifies that the sender has active membership in the group
    async fn create(
        &self, 
        payload: TextMessageCreateDto,
        sender_id: i32,
    ) -> Result<TextMessageReadDto, ApiError>;
}
