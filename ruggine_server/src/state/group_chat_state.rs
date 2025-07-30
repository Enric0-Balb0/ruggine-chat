use crate::config::database::Database;
use crate::repository::group_chat_repository::{GroupChatRepository, GroupChatRepositoryTrait};
use crate::repository::user_repository::{UserRepository, UserRepositoryTrait};
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use std::sync::Arc;

#[derive(Clone)]
pub struct GroupChatState {
    pub group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub group_chat_repo: Arc<dyn GroupChatRepositoryTrait>,
    pub user_repo: Arc<dyn UserRepositoryTrait>,
}

impl GroupChatState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let group_chat_repo = Arc::new(GroupChatRepository::new(db_conn));
        let user_repo = Arc::new(UserRepository::new(db_conn));
        let group_chat_service = Arc::new(GroupChatService::new(db_conn));

        Self {
            group_chat_service,
            group_chat_repo,
            user_repo,
        }
    }

    pub fn with_dependencies(
        group_chat_service: Arc<dyn GroupChatServiceTrait>,
        group_chat_repo: Arc<dyn GroupChatRepositoryTrait>,
        user_repo: Arc<dyn UserRepositoryTrait>,
    ) -> Self {
        Self {
            group_chat_service,
            group_chat_repo,
            user_repo,
        }
    }
}
