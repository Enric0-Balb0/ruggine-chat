use async_trait::async_trait;
use crate::dto::text_message_dto::{TextMessageReadDto};
use crate::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use crate::error::api_error::ApiError;
use crate::response::paginated_response::PaginatedResponse;
use mockall::automock;

#[automock]
#[async_trait]
pub trait TextMessageServiceTrait: Send + Sync {
    /// Find a text message by its ID
    async fn find_by_id(&self, id: i32) -> Result<TextMessageReadDto, ApiError>;
    
    /// Find text messages by group chat ID with pagination
    async fn find_by_group_chat_id_paginated(
        &self, 
        group_chat_id: i32, 
        pagination_query: TextMessagePaginationQuery
    ) -> Result<PaginatedResponse<TextMessageReadDto>, ApiError>;
}
