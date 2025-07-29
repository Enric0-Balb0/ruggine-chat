use crate::config::database::{Database, DatabaseTrait};
use crate::repository::group_chat_repository::GroupChatRepositoryTrait;
use crate::entity::group_chat::NewGroupChat;
use async_trait::async_trait;
use std::sync::Arc;
use sqlx::Error as SqlxError;

#[derive(Clone)]
pub struct GroupChatRepository {
    pub(crate) db_conn: Arc<Database>,
}

impl GroupChatRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<i32, SqlxError> {
        let result = sqlx::query!(
            "DELETE FROM \"group_chat\" WHERE id = $1",
            id
        )
            .execute(self.db_conn.get_pool())
            .await?;

        Ok(result.rows_affected() as i32)
    }
}
