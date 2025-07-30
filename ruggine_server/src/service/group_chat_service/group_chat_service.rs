use std::sync::Arc;
use crate::config::database::Database;
use crate::repository::group_chat_repository::{GroupChatRepository, GroupChatRepositoryTrait};
use crate::service::user_service::{UserService, UserServiceTrait};

#[derive(Clone)]
pub struct GroupChatService {
    pub(crate) group_chat_repo: Arc<dyn GroupChatRepositoryTrait>,
    pub(crate) user_service: Arc<dyn UserServiceTrait>,
}

impl GroupChatService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            group_chat_repo: Arc::new(GroupChatRepository::new(db_conn)),
            user_service: Arc::new(UserService::new(db_conn)),
        }
    }

    pub fn with(
        group_chat_repo: Arc<dyn GroupChatRepositoryTrait>,
        user_service: Arc<dyn UserServiceTrait>,
    ) -> Self {
        Self {
            group_chat_repo,
            user_service
        }
    }

    pub fn group_chat_repo(&self) -> Arc<dyn GroupChatRepositoryTrait> {
        Arc::clone(&self.group_chat_repo)
    }

    pub fn user_service(&self) -> Arc<dyn UserServiceTrait> {
        Arc::clone(&self.user_service)
    }
}