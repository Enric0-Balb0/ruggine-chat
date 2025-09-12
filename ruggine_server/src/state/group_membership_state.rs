use std::sync::Arc;
use crate::config::database::Database;
use crate::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use crate::service::websocket::WebSocketGroupServiceTrait;
use crate::utils::service_initializer::ServiceInitializer;
use crate::websocket::core::manager_trait::WebSocketManagerTrait;

#[derive(Clone)]
pub struct GroupMembershipState {
    pub(crate) group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    pub websocket_group_service: Option<Arc<dyn WebSocketGroupServiceTrait>>,
    pub ws_manager: Option<Arc<dyn WebSocketManagerTrait>>,
}

impl GroupMembershipState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let service_init = ServiceInitializer::new(db_conn);
        let group_membership_service = service_init.group_membership_service();

        Self {
            group_membership_service,
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
        group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    ) -> Self {
        Self {
            group_membership_service,
            websocket_group_service: None,
            ws_manager: None,
        }
    }
}
