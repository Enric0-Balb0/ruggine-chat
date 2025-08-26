use std::sync::Arc;

use crate::{
    state::token_state::TokenState, 
    websocket::WebSocketManager,
    service::websocket::{WebSocketGroupService, WebSocketGroupServiceTrait},
    service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait},
    config::database::Database,
};
use crate::utils::service_initializer::ServiceInitializer;

#[derive(Clone)]
pub struct WebSocketState {
    pub manager: Arc<WebSocketManager>,
    pub token_state: Arc<TokenState>,
    pub group_service: Arc<dyn WebSocketGroupServiceTrait>,
}

impl WebSocketState {
    pub fn new(token_state: Arc<TokenState>, db_conn: &Arc<Database>) -> Self {
        let manager = Arc::new(WebSocketManager::new());
        
        // Crea il servizio per i gruppi WebSocket
        let service_init = ServiceInitializer::new(db_conn);
        
        let group_service = service_init.websocket_group_service();
        
        Self {
            manager,
            token_state,
            group_service,
        }
    }
}