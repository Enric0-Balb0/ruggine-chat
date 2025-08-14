mod find_by_id;
mod find_by_group_chat_id_paginated;
pub mod text_message_service;
pub mod text_message_service_trait;

use async_trait::async_trait;
use crate::dto::text_message_dto::TextMessageReadDto;
use crate::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use crate::error::api_error::ApiError;
use crate::response::paginated_response::PaginatedResponse;
pub use crate::service::text_message_service::text_message_service::TextMessageService;
pub use crate::service::text_message_service::text_message_service_trait::TextMessageServiceTrait;

#[async_trait]
impl TextMessageServiceTrait for TextMessageService {
    async fn find_by_id(&self, id: i32) -> Result<TextMessageReadDto, ApiError> {
        self.find_by_id_internal(id).await
    }

    async fn find_by_group_chat_id_paginated(
        &self,
        group_chat_id: i32,
        pagination_query: TextMessagePaginationQuery,
    ) -> Result<PaginatedResponse<TextMessageReadDto>, ApiError> {
        self.find_by_group_chat_id_paginated_internal(group_chat_id, pagination_query).await
    }
}
