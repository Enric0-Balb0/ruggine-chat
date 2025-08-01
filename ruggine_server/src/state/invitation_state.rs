use crate::config::database::Database;
use crate::service::invitation_service::{InvitationService, InvitationServiceTrait};
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use std::sync::Arc;

#[derive(Clone)]
pub struct InvitationState {
    pub invitation_service: Arc<dyn InvitationServiceTrait>,
    pub user_service: Arc<dyn UserServiceTrait>,
    pub group_chat_service: Arc<dyn GroupChatServiceTrait>,
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
        }
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
        }
    }
}
