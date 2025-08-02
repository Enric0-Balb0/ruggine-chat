use crate::config::database::{Database, DatabaseTrait};
use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
use crate::entity::group_membership::{NewGroupMembership, GroupMembership};
use async_trait::async_trait;
use std::sync::Arc;
use sqlx::Error as SqlxError;

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
}

#[async_trait]
impl GroupMembershipRepositoryTrait for GroupMembershipRepository {
    async fn insert(&self, new_group_membership: NewGroupMembership) -> Result<i32, SqlxError> {
        self.insert_inner(new_group_membership).await
    }

    async fn find_by_id(&self, id: i32) -> Result<GroupMembership, SqlxError> {
        self.find_by_id_inner(id).await
    }
}