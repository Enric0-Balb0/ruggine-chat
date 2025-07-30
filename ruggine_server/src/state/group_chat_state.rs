use crate::config::database::Database;
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use std::sync::Arc;
use crate::service::user_service::{UserService, UserServiceTrait};

#[derive(Clone)]
pub struct GroupChatState {
    pub group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
}

impl GroupChatState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let user_service = Arc::new(UserService::new(db_conn));
        let group_chat_service = Arc::new(GroupChatService::new(db_conn));

        Self {
            group_chat_service,
            user_service,
        }
    }

    pub fn with_dependencies(
        group_chat_service: Arc<dyn GroupChatServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
    ) -> Self {
        Self {
            group_chat_service,
            user_service,
        }
    }
}
