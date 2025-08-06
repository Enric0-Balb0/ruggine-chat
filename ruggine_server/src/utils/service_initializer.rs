use std::sync::Arc;
use crate::config::database::Database;
use crate::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use crate::service::invitation_service::{InvitationService, InvitationServiceTrait};
use crate::service::user_service::{UserService, UserServiceTrait};
use crate::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};

/// ServiceInitializer manages the creation and initialization of all services
/// with proper dependency injection to avoid circular dependencies.
#[derive(Clone)]
pub struct ServiceInitializer {
    group_chat_service: Arc<dyn GroupChatServiceTrait>,
    invitation_service: Arc<dyn InvitationServiceTrait>,
    user_service: Arc<dyn UserServiceTrait>,
    group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
}

impl ServiceInitializer {
    /// Initialize all services with proper dependency injection to avoid circular dependencies.
    /// This method creates all services in the correct order and sets up dependencies properly.
    pub fn new(db_conn: &Arc<Database>) -> Self {
        // Create independent services first
        let user_service = Arc::new(UserService::new(db_conn));
        let group_membership_service = Arc::new(GroupMembershipService::new(db_conn));
        
        // Create group chat service without invitation service dependency
        let group_chat_service = Arc::new(GroupChatService::new(db_conn));
        
        // Create invitation service (it will create its own internal group chat service for now)
        let invitation_service = Arc::new(InvitationService::new(db_conn));
        
        // Set up circular dependencies using dependency injection
        group_chat_service.set_invitation_service(invitation_service.clone());
        group_membership_service.set_invitation_service(invitation_service.clone());
        
        Self {
            group_chat_service,
            invitation_service,
            user_service,
            group_membership_service,
        }
    }

    /// Get the GroupChatService instance
    pub fn group_chat_service(&self) -> Arc<dyn GroupChatServiceTrait> {
        Arc::clone(&self.group_chat_service)
    }

    /// Get the InvitationService instance
    pub fn invitation_service(&self) -> Arc<dyn InvitationServiceTrait> {
        Arc::clone(&self.invitation_service)
    }

    /// Get the UserService instance
    pub fn user_service(&self) -> Arc<dyn UserServiceTrait> {
        Arc::clone(&self.user_service)
    }

    /// Get the GroupMembershipService instance
    pub fn group_membership_service(&self) -> Arc<dyn GroupMembershipServiceTrait> {
        Arc::clone(&self.group_membership_service)
    }
}
