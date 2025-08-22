use crate::config::database::Database;
use crate::service::text_message_service::{TextMessageService, TextMessageServiceTrait};
use crate::service::websocket::WebSocketGroupService;
use crate::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use crate::websocket::WebSocketManager;
use std::sync::Arc;
use crate::utils::service_initializer::ServiceInitializer;

#[derive(Clone)]
pub struct TextMessageState {
    pub text_message_service: Arc<dyn TextMessageServiceTrait>,
    pub websocket_group_service: Option<Arc<WebSocketGroupService>>,
    pub ws_manager: Option<Arc<WebSocketManager>>,
}

impl TextMessageState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let service_init = ServiceInitializer::new(db_conn);
        let text_message_service = service_init.text_message_service();

        Self {
            text_message_service,
            websocket_group_service: None, // Sarà impostato quando necessario
            ws_manager: None,
        }
    }

    pub fn with_websocket_service(mut self, ws_manager: Arc<WebSocketManager>, websocket_group_service: Arc<WebSocketGroupService>) -> Self {
        self.websocket_group_service = Some(websocket_group_service);
        self.ws_manager = Some(ws_manager);
        self
    }

    pub fn with_dependencies(
        text_message_service: Arc<dyn TextMessageServiceTrait>,
    ) -> Self {
        Self {
            text_message_service,
            websocket_group_service: None,
            ws_manager: None,
        }
    }
}
