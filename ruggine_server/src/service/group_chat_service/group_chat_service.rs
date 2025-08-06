use std::sync::{Arc, Mutex, RwLock};
use crate::config::database::Database;
use crate::repository::group_chat_repository::{GroupChatRepository, GroupChatRepositoryTrait};
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::service::invitation_service::{InvitationService, InvitationServiceTrait};

#[derive(Clone)]
pub struct GroupChatService {
    pub(crate) group_chat_repo: Arc<dyn GroupChatRepositoryTrait>,
    pub(crate) user_service: Arc<dyn UserServiceTrait>,
    pub(crate) invitation_service: Arc<RwLock<Option<Arc<dyn InvitationServiceTrait>>>>,
}

impl GroupChatService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            group_chat_repo: Arc::new(GroupChatRepository::new(db_conn)),
            user_service: Arc::new(UserService::new(db_conn)),
            invitation_service: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with(
        group_chat_repo: Arc<dyn GroupChatRepositoryTrait>,
        user_service: Arc<dyn UserServiceTrait>,
    ) -> Self {
        Self {
            group_chat_repo,
            user_service,
            invitation_service: Arc::new(RwLock::new(None)),
        }
    }

    pub fn group_chat_repo(&self) -> Arc<dyn GroupChatRepositoryTrait> {
        Arc::clone(&self.group_chat_repo)
    }

    pub fn user_service(&self) -> Arc<dyn UserServiceTrait> {
        Arc::clone(&self.user_service)
    }

    pub fn invitation_service(&self) ->Arc<dyn InvitationServiceTrait> {
        self.invitation_service
            .read()
            .unwrap()
            .as_ref()
            .expect("invitation_service not initialized")
            .clone()
    }
}