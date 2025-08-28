use crate::config::database::{Database, DatabaseTrait};
use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
use crate::entity::group_membership::{MembershipStatus, NewGroupMembership, UpdateGroupMembership};
use async_trait::async_trait;
use std::sync::Arc;
use sqlx::Error as SqlxError;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;

#[derive(Clone)]
pub struct GroupMembershipRepository {
    pub(crate) db_conn: Arc<Database>,
}

impl GroupMembershipRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<i32, SqlxError> {
        let result = sqlx::query!(
            "DELETE FROM \"group_membership\" WHERE id = $1",
            id
        )
            .execute(self.db_conn.get_pool())
            .await?;

        Ok(result.rows_affected() as i32)
    }

    pub async fn delete_by_invitation_id(&self, invitation_id: i32) -> Result<i32, SqlxError> {
        let result = sqlx::query!(
            "DELETE FROM \"group_membership\" WHERE invitation_id = $1",
            invitation_id
        )
            .execute(self.db_conn.get_pool())
            .await?;

        Ok(result.rows_affected() as i32)
    }
}

#[async_trait]
impl GroupMembershipRepositoryTrait for GroupMembershipRepository {
    async fn insert(&self, new_group_membership: NewGroupMembership) -> Result<i32, SqlxError> {
        self.insert_inner(new_group_membership).await
    }

    async fn update(&self, update_group_membership: UpdateGroupMembership) -> Result<(), SqlxError> {
        self.update_inner(update_group_membership).await
    }

    async fn find_by_id_and_user_id(&self, id: i32, user_id: i32) -> Result<GroupMembershipWithInvitationRow, SqlxError> {
        self.find_by_id_and_user_id_inner(id, user_id).await
    }

    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<GroupMembershipWithInvitationRow>, SqlxError> {
        self.find_by_user_id_inner(user_id).await
    }

    async fn find_by_user_id_and_group_id(&self, user_id: i32, group_id: i32, membership_statuses: Vec<MembershipStatus>) -> Result<Vec<GroupMembershipWithInvitationRow>, SqlxError> {
        self.find_by_user_id_and_group_id_inner(user_id, group_id, membership_statuses).await
    }

    async fn find_active_by_user_id_and_group_id(&self, user_id: i32, group_id: i32) -> Result<GroupMembershipWithInvitationRow, SqlxError> {
        self.find_active_by_user_id_and_group_id_inner(user_id, group_id).await
    }

    async fn find_by_group_chat_id(&self, group_id: i32) -> Result<Vec<GroupMembershipWithInvitationRow>, SqlxError> {
        self.find_by_group_chat_id_inner(group_id).await
    }

    async fn find_connected_users(
        &self,
        user_id: i32,
    ) -> Result<Vec<i32>, SqlxError> {
        self.find_connected_users_inner(user_id).await
    }
}