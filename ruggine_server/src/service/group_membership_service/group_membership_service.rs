use std::sync::{Arc, RwLock};
use crate::config::database::Database;
use crate::service::invitation_service::{InvitationService, InvitationServiceTrait};
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use crate::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};

#[derive(Clone)]
pub struct GroupMembershipService {
    pub(crate) group_membership_repo: Arc<dyn GroupMembershipRepositoryTrait>,
    pub(crate) group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub(crate) user_service: Arc<dyn UserServiceTrait>,
    // Use this approach to break the dependency loop, add last Arc beacuse RwLock is not clonable
    pub(crate) invitation_service: Arc<RwLock<Option<Arc<dyn InvitationServiceTrait>>>>,
}

impl GroupMembershipService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            group_membership_repo: Arc::new(GroupMembershipRepository::new(db_conn)),
            group_chat_service: Arc::new(GroupChatService::new(db_conn)),
            user_service: Arc::new(UserService::new(db_conn)),
            invitation_service: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with(
        group_membership_repo: Arc<dyn GroupMembershipRepositoryTrait>,
        group_chat_service: Arc<dyn GroupChatServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
    ) -> Self {
        Self {
            group_membership_repo,
            group_chat_service,
            user_service,
            invitation_service: Arc::new(RwLock::new(None)),
        }
    }

    pub fn invitation_service(&self) -> Arc<dyn InvitationServiceTrait> {
        self.invitation_service
            .read()
            .unwrap()
            .as_ref()
            .expect("invitation_service not initialized")
            .clone()
    }


    pub fn group_membership_repo(&self) -> Arc<dyn GroupMembershipRepositoryTrait> {
        Arc::clone(&self.group_membership_repo)
    }

    pub fn user_service(&self) -> Arc<dyn UserServiceTrait> {
        Arc::clone(&self.user_service)
    }

    pub fn group_chat_service(&self) -> Arc<dyn GroupChatServiceTrait> {
        Arc::clone(&self.group_chat_service)
    }
}