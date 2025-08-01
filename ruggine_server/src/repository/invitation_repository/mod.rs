pub mod invitation_repository;
pub mod invitation_repository_trait;
mod insert;
mod find_by_id;
mod find_pending_invitations_for_user;
mod find_pending_invitations_between_users;

use async_trait::async_trait;
use sqlx::Error;
pub use invitation_repository::InvitationRepository;
pub use invitation_repository_trait::InvitationRepositoryTrait;
use crate::entity::{invitation::{Invitation, NewInvitation}, user};

#[async_trait]
impl InvitationRepositoryTrait for InvitationRepository {
    async fn insert(&self, new_invitation: NewInvitation) -> Result<i32, Error> {
        self.insert_inner(new_invitation).await
    }

    async fn find_by_id(&self, id: i32, user_id: i32) -> Result<Invitation, Error> {
        self.find_by_id_inner(id, user_id).await
    }

    async fn find_pending_invitations_for_user(&self, user_id: i32) -> Result<Vec<Invitation>, Error> {
        self.find_pending_invitations_for_user_inner(user_id).await
    }

    async fn find_pending_invitation_between_users(&self, from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Result<Option<Invitation>, Error> {
        self.find_pending_invitation_between_users_inner(from_user_id, to_user_id, group_chat_id).await
    }
}