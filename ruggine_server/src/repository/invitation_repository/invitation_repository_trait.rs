use crate::entity::invitation::{Invitation, NewInvitation};
use async_trait::async_trait;
use sqlx::Error as SqlxError;
use mockall::automock;

#[async_trait]
#[automock]
pub trait InvitationRepositoryTrait: Send + Sync {
    async fn insert(&self, new_group_chat: NewInvitation) -> Result<i32, SqlxError>;
    async fn find_by_id(&self, id: i32) -> Result<Invitation, SqlxError>;
}
