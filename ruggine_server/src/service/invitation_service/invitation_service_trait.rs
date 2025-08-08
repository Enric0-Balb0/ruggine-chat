use crate::dto::invitation_dto::{InvitationCreateDto, InvitationReadDto, InvitationUpdateStatusDto, InvitationUpdateResponseDto};
use crate::error::api_error::ApiError;
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock]
pub trait InvitationServiceTrait: Send + Sync {
    async fn send(&self, payload: InvitationCreateDto, from_user_id: i32) -> Result<InvitationReadDto, ApiError>;
    async fn send_for_group_chat_create(&self, payload: InvitationCreateDto, from_user_id: i32) -> Result<InvitationReadDto, ApiError>;
    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<InvitationReadDto, ApiError>;
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<InvitationReadDto>, ApiError>;
    async fn update_status(&self, payload: InvitationUpdateStatusDto, auth_user_id: i32) -> Result<InvitationUpdateResponseDto, ApiError>;
}
