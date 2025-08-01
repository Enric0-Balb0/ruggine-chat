use crate::dto::invitation_dto::{InvitationCreateDto, InvitationReadDto};
use crate::error::api_error::ApiError;
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock]
pub trait InvitationServiceTrait: Send + Sync {
    async fn send(&self, payload: InvitationCreateDto, from_user_id: i32) -> Result<InvitationReadDto, ApiError>;
    async fn find_by_id(&self, id: i32, user_id: i32) -> Result<InvitationReadDto, ApiError>;
}
