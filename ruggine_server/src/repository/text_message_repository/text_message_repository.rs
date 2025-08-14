use std::sync::Arc;
use crate::config::database::{Database, DatabaseTrait};
use crate::entity::text_message::{TextMessage, NewTextMessage};
use crate::repository::text_message_repository::text_message_repository_trait::TextMessageRepositoryTrait;
use sqlx::Error;
use async_trait::async_trait;

#[derive(Clone)]
pub struct TextMessageRepository {
    pub(crate) db_conn: Arc<Database>,
}

impl TextMessageRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<i32, sqlx::Error> {
        let rec = sqlx::query_scalar(
            r#"
            DELETE FROM text_message
            WHERE id = $1
            RETURNING id
            "#
        )
        .bind(id)
        .fetch_one(self.db_conn.get_pool())
        .await?;

        Ok(rec)
    }
}

#[async_trait]
impl TextMessageRepositoryTrait for TextMessageRepository {
    async fn find(&self, id: i32) -> Result<TextMessage, Error> {
        self.find_inner(id).await
    }

    async fn insert(&self, new_text_message: NewTextMessage) -> Result<i32, Error> {
        self.insert_inner(new_text_message).await
    }

    async fn find_by_group_chat_id_paginated(&self, group_chat_id: i32, cursor: Option<chrono::DateTime<chrono::Utc>>, limit: usize) -> Result<Vec<TextMessage>, Error> {
        self.find_by_group_chat_id_paginated_inner(group_chat_id, cursor, limit).await
    }
}
