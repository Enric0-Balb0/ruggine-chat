pub mod invitation_repository;
pub mod invitation_repository_trait;
mod insert;
mod find_by_id;

use async_trait::async_trait;
use sqlx::Error;
pub use crate::repository::invitation_repository::invitation_repository::InvitationRepository;
pub use crate::repository::invitation_repository::invitation_repository_trait::InvitationRepositoryTrait;
use crate::entity::invitation::{NewInvitation, Invitation};

#[async_trait]
impl InvitationRepositoryTrait for InvitationRepository {
    async fn insert(&self, new_invitation: NewInvitation) -> Result<i32, Error> {
        self.insert_inner(new_invitation).await
    }

    async fn find_by_id(&self, id: i32) -> Result<Invitation, Error> {
        self.find_by_id_inner(id).await
    }
}