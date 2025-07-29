use crate::entity::group_chat::{GroupChat, NewGroupChat, UpdateGroupChat};
use async_trait::async_trait;
use sqlx::Error;
use sqlx::Error as SqlxError;
use mockall::automock;

#[async_trait]
#[automock]
pub trait GroupChatRepositoryTrait: Send + Sync {
    async fn insert(&self, new_group_chat: NewGroupChat) -> Result<i32, SqlxError>;
}
