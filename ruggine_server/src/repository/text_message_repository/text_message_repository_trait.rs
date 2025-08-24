use crate::entity::text_message::{TextMessage, NewTextMessage, TextMessageInfoUpdate, TextMessageInfo, NewTextMessageInfo};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Error;
use sqlx::Error as SqlxError;
use mockall::automock;
use crate::dto::text_message_dto::TextMessageInfoReadDto;

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
}
