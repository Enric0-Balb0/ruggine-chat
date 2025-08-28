pub mod group_chat_repository;
pub mod group_chat_repository_trait;
mod insert;
pub mod find_by_id;

use std::sync::Arc;
use async_trait::async_trait;
use sqlx::Error;
pub use group_chat_repository::GroupChatRepository;
pub use group_chat_repository_trait::GroupChatRepositoryTrait;
use crate::config::database::Database;
use crate::entity::group_chat::{NewGroupChat, GroupChat};

#[async_trait]
impl GroupChatRepositoryTrait for GroupChatRepository {
    async fn insert(&self, new_group_chat: NewGroupChat) -> Result<i32, Error> {
        self.insert_inner(new_group_chat).await
    }

    async fn find_by_id(&self, id: i32) -> Result<GroupChat, Error> {
        self.find_by_id_inner(id).await
    }
    fn db_conn(&self) -> Arc<Database> {
        self.db_conn.clone()
    }
}