use std::sync::Arc;
use crate::config::database::{Database, DatabaseTrait};
use crate::entity::text_message::{NewTextMessage, NewTextMessageInfo, TextMessage, TextMessageInfo, TextMessageInfoUpdate};
use crate::repository::text_message_repository::text_message_repository_trait::TextMessageRepositoryTrait;
use sqlx::Error;
use async_trait::async_trait;
use crate::dto::text_message_dto::TextMessageInfoReadDto;

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

    async fn insert_text_message_info(&self, text_message_info: NewTextMessageInfo) -> Result<i32, Error> {
        self.insert_text_message_info_inner(text_message_info).await
    }

    async fn find_info_by_id(&self, id: i32) -> Result<TextMessageInfo, Error> {
        self.find_info_by_id_inner(id).await
    }

    async fn find_info_by_message_id(&self, message_id: i32) -> Result<Vec<TextMessageInfo>, Error> {
        self.find_info_by_message_id_inner(message_id).await
    }

    async fn find_info_last_read(&self, user_id: i32, group_chat_id: i32) -> Result<TextMessageInfo, Error> {
        self.find_info_last_read_inner(user_id, group_chat_id).await
    }

    async fn find_info_last_sent(&self, user_id: i32, group_chat_id: i32) -> Result<TextMessageInfo, Error> {
        self.find_info_last_sent_inner(user_id, group_chat_id).await
    }

    async fn update_info(&self, text_message_info_update: TextMessageInfoUpdate) -> Result<(), Error> {
        self.update_info_inner(text_message_info_update).await
    }
}
