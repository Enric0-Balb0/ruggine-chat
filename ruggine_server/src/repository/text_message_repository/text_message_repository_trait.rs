use crate::entity::text_message::{TextMessage, NewTextMessage};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Error;
use sqlx::Error as SqlxError;
use mockall::automock;

#[async_trait]
#[automock]
pub trait TextMessageRepositoryTrait: Send + Sync {
    async fn find(&self, id: i32) -> Result<TextMessage, Error>;
    async fn insert(&self, new_text_message: NewTextMessage) -> Result<i32, SqlxError>;
    async fn find_by_group_chat_id_paginated(&self, group_chat_id: i32, cursor: Option<DateTime<Utc>>, limit: usize) -> Result<Vec<TextMessage>, Error>;
}
