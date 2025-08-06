use std::sync::Arc;
use crate::config::database::Database;
use crate::entity::invitation;
pub use crate::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use crate::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};

#[derive(Clone)]
pub struct InvitationService {
    pub(crate) invitation_repo: Arc<dyn InvitationRepositoryTrait>,
    pub(crate) group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub(crate) user_service: Arc<dyn UserServiceTrait>,
    pub(crate) group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
}

impl InvitationService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let group_membership_service = Arc::new(GroupMembershipService::new(db_conn));

        let invitation_service = Self {
            invitation_repo: Arc::new(InvitationRepository::new(db_conn)),
            group_chat_service: Arc::new(GroupChatService::new(db_conn)),
            user_service: Arc::new(UserService::new(db_conn)),
            group_membership_service: group_membership_service.clone(),
        };

        group_membership_service.set_invitation_service(Arc::new(invitation_service.clone()));

        invitation_service
    }

    pub fn with(
        invitation_repo: Arc<dyn InvitationRepositoryTrait>,
        group_chat_service: Arc<dyn GroupChatServiceTrait>,
        user_service: Arc<dyn UserServiceTrait>,
        group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    ) -> Self {
        Self {
            invitation_repo,
            group_chat_service,
            user_service,
            group_membership_service,
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

    pub fn group_membership_service(&self) -> Arc<dyn GroupMembershipServiceTrait> {
        Arc::clone(&self.group_membership_service)
    }
}