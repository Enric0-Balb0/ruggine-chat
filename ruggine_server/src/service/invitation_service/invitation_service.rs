use std::sync::Arc;
use crate::config::database::Database;
pub use crate::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};

#[derive(Clone)]
pub struct InvitationService {
    pub(crate) invitation_repo: Arc<dyn InvitationRepositoryTrait>,
    pub(crate) group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub(crate) user_service: Arc<dyn UserServiceTrait>,
}

impl InvitationService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            invitation_repo: Arc::new(InvitationRepository::new(db_conn)),
            group_chat_service: Arc::new(GroupChatService::new(db_conn)),
            user_service: Arc::new(UserService::new(db_conn)),
        }
    }

    pub fn with(
        invitation_repo: Arc<dyn InvitationRepositoryTrait>,
        group_chat_service: Arc<dyn GroupChatServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
    ) -> Self {
        Self {
            invitation_repo,
            group_chat_service,
            user_service
        }
    }

    pub fn invitation_repo(&self) -> Arc<dyn InvitationRepositoryTrait> {
        Arc::clone(&self.invitation_repo)
    }

    pub fn user_service(&self) -> Arc<dyn UserServiceTrait> {
        Arc::clone(&self.user_service)
    }

    pub fn group_chat_service(&self) -> Arc<dyn GroupChatServiceTrait> {
        Arc::clone(&self.group_chat_service)
    }
}