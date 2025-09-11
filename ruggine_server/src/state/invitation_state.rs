use crate::config::database::Database;
use crate::service::invitation_service::{InvitationService, InvitationServiceTrait};
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use std::sync::Arc;
use crate::service::websocket::WebSocketGroupServiceTrait;
use crate::websocket::core::manager_trait::WebSocketManagerTrait;

#[derive(Clone)]
pub struct InvitationState {
    pub invitation_service: Arc<dyn InvitationServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
    pub group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub websocket_group_service: Option<Arc<dyn WebSocketGroupServiceTrait>>,
    pub ws_manager: Option<Arc<dyn WebSocketManagerTrait>>,
}

impl InvitationState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let user_service = Arc::new(UserService::new(db_conn));
        let group_chat_service = Arc::new(GroupChatService::new(db_conn));
        let invitation_service = Arc::new(InvitationService::new(db_conn));

        Self {
            invitation_service,
            user_service,
            group_chat_service,
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
        invitation_service: Arc<dyn InvitationServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
        group_chat_service: Arc<dyn GroupChatServiceTrait>,
    ) -> Self {
        Self {
            invitation_service,
            user_service,
            group_chat_service,
            websocket_group_service: None,
            ws_manager: None,
        }
    }
}
