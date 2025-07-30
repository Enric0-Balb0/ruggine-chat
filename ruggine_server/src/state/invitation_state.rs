/* use crate::config::database::Database;
use crate::service::invitation_service::{InvitationService, InvitationServiceTrait};
use std::sync::Arc;
use crate::service::user_service::{UserService, UserServiceTrait};

#[derive(Clone)]
pub struct InvitationState {
    pub invitation_service: Arc<dyn InvitationServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
}

impl InvitationState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let user_service = Arc::new(UserService::new(db_conn));
        let invitation_service = Arc::new(InvitationService::new(db_conn));

        Self {
            invitation_service,
            user_service,
        }
    }

    pub fn with_dependencies(
        invitation_service: Arc<dyn InvitationServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
    ) -> Self {
        Self {
            invitation_service,
            user_service,
        }
    }
} */
