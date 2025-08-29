use std::sync::Arc;

use crate::service::user_service::UserServiceTrait;
use crate::utils::service_initializer::ServiceInitializer;
use crate::{
    config::database::Database,
    service::websocket::WebSocketGroupServiceTrait,
    state::token_state::TokenState
    ,
    websocket::WebSocketManager,
};
use crate::websocket::core::manager_trait::WebSocketManagerTrait;

#[derive(Clone)]
pub struct WebSocketState {
    pub manager: Arc<dyn WebSocketManagerTrait>,
    pub token_state: Arc<TokenState>,
    pub group_service: Arc<dyn WebSocketGroupServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
}

impl WebSocketState {
    pub fn new(token_state: Arc<TokenState>, db_conn: &Arc<Database>) -> Self {
        let manager = Arc::new(WebSocketManager::new());
        
        // Crea il servizio per i gruppi WebSocket
        let service_init = ServiceInitializer::new(db_conn);
        
        let group_service = service_init.websocket_group_service();
        let user_service = service_init.user_service();
        
        Self {
            manager,
            token_state,
            group_service,
            user_service,
        }
    }
}