mod find_by_id;
mod find_by_group_chat_id_paginated;
mod create;
mod find_info_by_id;
mod find_info_by_message_id;
mod find_info_last_read_at;
mod find_info_last_sent_at;
mod update_read_at;
mod create_info;
pub mod text_message_service;
pub mod text_message_service_trait;
mod find_info_by_id_checked;
mod update_sent_at;
mod find_info_by_user_id_and_message_id;

use crate::dto::text_message_dto::{TextMessageCreateDto, TextMessageInfoCreateDto, TextMessageInfoReadDto, TextMessageLastReadAtDto, TextMessageLastSentAtDto, TextMessageReadAtDtoUpdate, TextMessageReadDto, TextMessageSentAtDtoUpdate};
use crate::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use crate::error::api_error::ApiError;
use crate::response::PaginatedTextMessageResponse;
pub use crate::service::text_message_service::text_message_service::TextMessageService;
pub use crate::service::text_message_service::text_message_service_trait::TextMessageServiceTrait;
use async_trait::async_trait;

#[async_trait]
impl TextMessageServiceTrait for TextMessageService {
    async fn find_by_id(&self, id: i32) -> Result<TextMessageReadDto, ApiError> {
        self.find_by_id_internal(id).await
    }

    async fn find_by_group_chat_id_paginated(
        &self,
        group_chat_id: i32,
        auth_user_id: i32,
        pagination_query: TextMessagePaginationQuery,
    ) -> Result<PaginatedTextMessageResponse, ApiError> {
        self.find_by_group_chat_id_paginated_internal(group_chat_id, pagination_query, auth_user_id).await
    }

    async fn create(
        &self,
        payload: TextMessageCreateDto,
        sender_id: i32,
    ) -> Result<TextMessageReadDto, ApiError> {
        self.create_internal(payload, sender_id).await
    }

    async fn create_info(
        &self,
        payload: TextMessageInfoCreateDto,
        sender_id: i32,
    ) -> Result<TextMessageInfoReadDto, ApiError> {
        self.create_info_internal(payload, sender_id).await
    }

    async fn find_info_by_id(
        &self,
        id: i32
    ) -> Result<TextMessageInfoReadDto, ApiError> {
        self.find_info_by_id_internal(id).await
    }

    async fn find_info_by_id_checked(
        &self,
        id: i32,
        auth_user_id: i32
    ) -> Result<TextMessageInfoReadDto, ApiError> {
        self.find_info_by_id_checked_internal(id, auth_user_id).await
    }

    async fn find_info_last_read_at(
        &self,
        auth_user_id: i32,
        payload: TextMessageLastReadAtDto
    ) -> Result<Option<TextMessageInfoReadDto>, ApiError> {
        self.find_info_last_read_at_internal(auth_user_id, payload).await
    }

    async fn find_info_last_sent_at(
        &self,
        auth_user_id: i32,
        payload: TextMessageLastSentAtDto
    ) -> Result<Option<TextMessageInfoReadDto>, ApiError> {
        self.find_info_last_sent_at_internal(auth_user_id, payload).await
    }

    async fn find_info_by_message_id(
        &self,
        auth_user_id: i32,
        message_id: i32
    ) -> Result<Vec<TextMessageInfoReadDto>, ApiError> {
        self.find_info_by_message_id_internal(auth_user_id, message_id).await
    }

    async fn find_info_by_user_id_and_message_id(
        &self,
        auth_user_id: i32,
        message_id: i32
    ) -> Result<TextMessageInfoReadDto, ApiError> {
        self.find_info_by_user_id_and_text_message_id_internal(auth_user_id, message_id).await
    }

    async fn update_read_at(&self,
        auth_user_id: i32,
        payload: TextMessageReadAtDtoUpdate
    ) -> Result<TextMessageInfoReadDto, ApiError> {
        self.update_read_at_internal(auth_user_id, payload).await
    }

    async fn update_sent_at(
        &self,
        auth_user_id: i32,
        payload: TextMessageSentAtDtoUpdate
    ) -> Result<TextMessageInfoReadDto, ApiError> {
        self.update_sent_at_internal(auth_user_id, payload).await
    }
}
