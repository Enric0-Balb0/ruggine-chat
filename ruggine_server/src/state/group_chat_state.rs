use crate::config::database::Database;
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use std::sync::Arc;
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::service::websocket::WebSocketGroupServiceTrait;
use crate::utils::service_initializer::ServiceInitializer;
use crate::websocket::core::manager_trait::WebSocketManagerTrait;

#[derive(Clone)]
pub struct GroupChatState {
    pub group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
    pub websocket_group_service: Option<Arc<dyn WebSocketGroupServiceTrait>>,
    pub ws_manager: Option<Arc<dyn WebSocketManagerTrait>>,
}

impl GroupChatState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let service_init = ServiceInitializer::new(db_conn);
        let user_service = service_init.user_service();
        let group_chat_service = service_init.group_chat_service();

        Self {
            group_chat_service,
            user_service,
            websocket_group_service: None, // Sarà impostato quando necessario
            ws_manager: None,
        }
    }

    pub fn with_websocket_service(mut self, ws_manager: Arc<dyn WebSocketManagerTrait>, websocket_group_service: Arc<dyn WebSocketGroupServiceTrait>) -> Self {
        self.websocket_group_service = Some(websocket_group_service);
        self.ws_manager = Some(ws_manager);
        self
    }

    pub fn with_dependencies(
        group_chat_service: Arc<dyn GroupChatServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
    ) -> Self {
        Self {
            group_chat_service,
            user_service,
            websocket_group_service: None,
            ws_manager: None,
        }
    }
}
