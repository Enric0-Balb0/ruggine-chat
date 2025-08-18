pub mod invitation_repository;
pub mod invitation_repository_trait;
mod insert;
mod find_by_id_and_user_id;
mod find_by_user_id;
mod find_pending_invitations_for_user;
mod find_pending_invitations_between_users;
mod update_status;
mod find_by_id;

use std::sync::Arc;
use async_trait::async_trait;
use sqlx::Error;
pub use invitation_repository::InvitationRepository;
pub use invitation_repository_trait::InvitationRepositoryTrait;
use crate::config::database::Database;
use crate::entity::invitation::{Invitation, NewInvitation, UpdateInvitationStatus, UserInvitationFilter};

#[async_trait]
impl InvitationRepositoryTrait for InvitationRepository {
    async fn insert(&self, new_invitation: NewInvitation) -> Result<i32, Error> {
        self.insert_inner(new_invitation).await
    }

    async fn find_by_id(&self, id: i32) -> Result<Invitation, Error> {
        self.find_by_id_inner(id).await
    }

    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<Invitation, Error> {
        self.find_by_id_and_user_id_inner(id, user_id).await
    }

    async fn find_by_user_id(&self, user_id: i32, filter: UserInvitationFilter) -> Result<Vec<Invitation>, Error> {
        self.find_by_user_id_inner(user_id, filter).await
    }

    async fn find_pending_invitations_for_user(&self, user_id: i32) -> Result<Vec<Invitation>, Error> {
        self.find_pending_invitations_for_user_inner(user_id).await
    }

    async fn find_pending_invitation_between_users(&self, from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Result<Option<Invitation>, Error> {
        self.find_pending_invitation_between_users_inner(from_user_id, to_user_id, group_chat_id).await
    }

    async fn update_status(&self, invitation_id: i32, update_invitation_status: UpdateInvitationStatus) -> Result<Invitation, Error> {
        self.update_status_internal(invitation_id, update_invitation_status).await
    }

    fn db_conn(&self) -> Arc<Database> {
        self.db_conn.clone()
    }
}