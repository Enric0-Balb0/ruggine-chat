use crate::config::database::{Database, DatabaseTrait};
use crate::repository::invitation_repository::InvitationRepositoryTrait;
use crate::entity::invitation::{NewInvitation, Invitation};
use async_trait::async_trait;
use std::sync::Arc;
use sqlx::Error as SqlxError;

#[derive(Clone)]
pub struct InvitationRepository {
    pub(crate) db_conn: Arc<Database>,
}

impl InvitationRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<i32, SqlxError> {
        let result = sqlx::query(
            r#"
            DELETE FROM "invitation"
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(self.db_conn.get_pool())
        .await?;

        Ok(result.rows_affected() as i32)
    }
}