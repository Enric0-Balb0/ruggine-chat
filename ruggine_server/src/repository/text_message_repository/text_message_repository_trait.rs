use crate::config::database::{Database, DatabaseTrait};
use crate::entity::text_message::{NewTextMessage, NewTextMessageInfo, TextMessage, TextMessageInfo, TextMessageInfoUpdate};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mockall::automock;
use sqlx::Error;
use sqlx::Error as SqlxError;
use std::sync::Arc;

#[async_trait]
#[automock]
pub trait TextMessageRepositoryTrait: Send + Sync {
    async fn find(&self, id: i32) -> Result<TextMessage, Error>;
    async fn insert(&self, new_text_message: NewTextMessage) -> Result<i32, SqlxError>;
    async fn insert_text_message_info(&self, text_message_info: NewTextMessageInfo) -> Result<i32, SqlxError>;
    async fn find_by_group_chat_id_paginated(&self, group_chat_id: i32, cursor: Option<DateTime<Utc>>, limit: usize) -> Result<Vec<TextMessage>, Error>;
    async fn find_info_by_id(&self, id: i32) -> Result<TextMessageInfo, Error>;
    async fn find_info_by_message_id(&self, message_id: i32) -> Result<Vec<TextMessageInfo>, Error>;
    async fn find_info_last_read(&self, user_id: i32, group_chat_id: i32) -> Result<Option<TextMessageInfo>, Error>;
    async fn find_info_last_sent(&self, user_id: i32, group_chat_id: i32) -> Result<Option<TextMessageInfo>, Error>;
    async fn update_info(&self, text_message_info_update: TextMessageInfoUpdate) -> Result<(), Error>;
    async fn find_info_by_user_id_and_message_id(&self, user_id: i32, text_message_id: i32) -> Result<TextMessageInfo, Error>;
    async fn find_by_group_chat_id_datetime_range(&self, group_chat_id: i32, min_datetime: DateTime<Utc>, max_datetime: DateTime<Utc>) -> Result<Vec<TextMessage>, Error>;
    fn db_conn(&self) -> Arc<Database>;
}
