mod send;
pub mod invitation_service;
pub mod invitation_service_trait;
mod find_by_id_and_user_id;
mod update_status;

use async_trait::async_trait;
use crate::dto::invitation_dto::{InvitationCreateDto, InvitationReadDto, InvitationUpdateStatusDto, InvitationUpdateResponseDto};
use crate::error::api_error::ApiError;
pub use crate::service::invitation_service::invitation_service::InvitationService;
pub use crate::service::invitation_service::invitation_service_trait::InvitationServiceTrait;


#[async_trait]
impl InvitationServiceTrait for InvitationService {
    async fn send(&self, payload: InvitationCreateDto, from_user_id: i32) -> Result<InvitationReadDto, ApiError> {
        self.send_internal(payload, from_user_id).await
    }

    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<InvitationReadDto, ApiError> {
        self.find_by_id_and_user_id_internal(id, user_id).await
    }

    async fn update_status(&self, payload: InvitationUpdateStatusDto, auth_user_id: i32) -> Result<InvitationUpdateResponseDto, ApiError> {
        self.update_status_internal(payload, auth_user_id).await
    }
}