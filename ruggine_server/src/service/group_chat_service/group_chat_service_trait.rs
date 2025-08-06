use std::sync::Arc;
use crate::dto::group_chat_dto::{GroupChatCreateDto, GroupChatReadDto};
use crate::error::api_error::ApiError;
use crate::service::invitation_service::InvitationServiceTrait;
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock]
pub trait GroupChatServiceTrait: Send + Sync {
    fn set_invitation_service(&self, invitation_service: Arc<dyn InvitationServiceTrait>);
    async fn create(&self, payload: GroupChatCreateDto, created_by: i32) -> Result<GroupChatReadDto, ApiError>;
    async fn find_by_id(&self, id: i32) -> Result<GroupChatReadDto, ApiError>;
}
