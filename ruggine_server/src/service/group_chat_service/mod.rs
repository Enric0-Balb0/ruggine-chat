mod create;
pub mod group_chat_service;
pub mod group_chat_service_trait;
mod find_by_id;

use async_trait::async_trait;
use std::sync::Arc;
use crate::dto::group_chat_dto::{GroupChatCreateDto, GroupChatReadDto, GroupChatUpdateDto};
use crate::error::api_error::ApiError;
use crate::service::invitation_service::InvitationServiceTrait;
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

     fn set_invitation_service(&self, invitation_service: Arc<dyn InvitationServiceTrait>) {
        let mut writable = self.invitation_service.write().unwrap();
        *writable = Some(invitation_service);
     }

}
