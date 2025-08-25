use async_trait::async_trait;
use crate::dto::text_message_dto::{TextMessageCreateDto, TextMessageInfoCreateDto, TextMessageInfoReadDto, TextMessageLastReadAtDto, TextMessageLastSentAtDto, TextMessageInfoReadAtDtoUpdate, TextMessageReadDto, TextMessageInfoSentAtDtoUpdate};
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

    async fn find_messages_not_sent_yet(
        &self,
        group_chat_id: i32,
        auth_user_id: i32
    ) -> Result<PaginatedTextMessageResponse, ApiError>;
    
    /// Create a new text message
    /// Verifies that the sender has active membership in the group
    async fn create(
        &self, 
        payload: TextMessageCreateDto,
        sender_id: i32,
    ) -> Result<TextMessageReadDto, ApiError>;

     async fn create_info(
        &self,
        payload: TextMessageInfoCreateDto,
        sender_id: i32,
    ) -> Result<TextMessageInfoReadDto, ApiError>;

    async fn find_info_by_id(
        &self,
        id: i32
    ) -> Result<TextMessageInfoReadDto, ApiError>;

    async fn find_info_by_id_checked(
        &self,
        id: i32,
        auth_user_id: i32
    ) -> Result<TextMessageInfoReadDto, ApiError>;

    async fn find_info_last_read_at(
        &self,
        auth_user_id: i32,
        payload: TextMessageLastReadAtDto
    ) -> Result<Option<TextMessageInfoReadDto>, ApiError>;

    async fn find_info_last_sent_at(
        &self,
        auth_user_id: i32,
        payload: TextMessageLastSentAtDto
    ) -> Result<Option<TextMessageInfoReadDto>, ApiError>;

    async fn find_info_by_message_id(
        &self,
        auth_user_id: i32,
        message_id: i32
    ) -> Result<Vec<TextMessageInfoReadDto>, ApiError>;

    async fn find_info_by_user_id_and_message_id(
        &self,
        auth_user_id: i32,
        message_id: i32
    ) -> Result<TextMessageInfoReadDto, ApiError>;

    async fn update_read_at(&self,
        auth_user_id: i32,
        payload: TextMessageInfoReadAtDtoUpdate
    ) -> Result<TextMessageInfoReadDto, ApiError>;

    async fn update_sent_at(
        &self,
        auth_user_id: i32,
        payload: TextMessageInfoSentAtDtoUpdate
    ) -> Result<TextMessageInfoReadDto, ApiError>;
}
