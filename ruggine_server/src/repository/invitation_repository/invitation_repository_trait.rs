use crate::entity::invitation::{Invitation, NewInvitation, UpdateInvitationStatus};
use async_trait::async_trait;
use sqlx::Error as SqlxError;
use mockall::automock;

#[async_trait]
#[automock]
pub trait InvitationRepositoryTrait: Send + Sync {
    async fn insert(&self, new_group_chat: NewInvitation) -> Result<i32, SqlxError>;
    async fn find_by_id(&self, id: i32) -> Result<Invitation, SqlxError>;
    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<Invitation, SqlxError>;
    async fn find_pending_invitations_for_user(&self, user_id: i32) -> Result<Vec<Invitation>, SqlxError>;
    async fn find_pending_invitation_between_users(&self, from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Result<Option<Invitation>, SqlxError>;
    async fn update_status(&self, invitation_id: i32, update_invitation_status: UpdateInvitationStatus) -> Result<Invitation, SqlxError>;
}
