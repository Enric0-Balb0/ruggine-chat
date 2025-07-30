use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::group_chat::GroupChat;
use crate::entity::user::User;
use crate::repository::group_chat_repository::GroupChatRepository;

impl GroupChatRepository {
    pub async fn find_inner(&self, id: i32) -> Result<GroupChat, Error> {
        let group_chat = sqlx::query_as::<_, GroupChat>("SELECT * FROM \"group_chat\" WHERE id = $1")
            .bind(id)
            .fetch_one(self.db_conn.get_pool())
            .await;
        group_chat
    }
}

#[cfg(test)]
mod group_chat_repository_find_tests {
    use super::*;
    use crate::entity::user::{User};
    use chrono::{Utc};
    use mockall::predicate::*;
    use crate::repository::group_chat_repository::group_chat_repository_trait::MockGroupChatRepositoryTrait;
    use crate::repository::group_chat_repository::GroupChatRepositoryTrait;

    #[tokio::test]
    async fn test_find_by_id_success() {

    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        
    }
}