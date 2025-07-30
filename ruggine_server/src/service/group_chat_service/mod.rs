mod create;
pub mod group_chat_service;
pub mod group_chat_service_trait;
mod find_by_id;

use async_trait::async_trait;
use crate::dto::group_chat_dto::{GroupChatCreateDto, GroupChatReadDto, GroupChatUpdateDto};
use crate::error::api_error::ApiError;
pub use crate::service::group_chat_service::group_chat_service::GroupChatService;
pub use crate::service::group_chat_service::group_chat_service_trait::GroupChatServiceTrait;



#[async_trait]
impl GroupChatServiceTrait for GroupChatService {
    async fn create(&self, payload: GroupChatCreateDto, user_id: i32) -> Result<GroupChatReadDto, ApiError> {
        self.create_internal(payload, user_id).await
    }

    async fn find_by_id(&self, id: i32) -> Result<GroupChatReadDto, ApiError> {
        self.find_by_id_internal(id).await
    }

}
