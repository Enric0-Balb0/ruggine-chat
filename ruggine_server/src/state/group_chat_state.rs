use crate::config::database::Database;
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use std::sync::Arc;
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::utils::service_initializer::ServiceInitializer;

#[derive(Clone)]
pub struct GroupChatState {
    pub group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
}

impl GroupChatState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let service_init = ServiceInitializer::new(db_conn);
        let user_service = service_init.user_service();
        let group_chat_service = service_init.group_chat_service();

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
