use std::sync::Arc;
use crate::repository::text_message_repository::TextMessageRepositoryTrait;
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::service::group_chat_service::GroupChatServiceTrait;
use crate::service::text_message_service::text_message_service_trait::TextMessageServiceTrait;
use async_trait::async_trait;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct TextMessageService {
    pub(crate) text_message_repo: Arc<dyn TextMessageRepositoryTrait>,
    pub(crate) group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    pub(crate) group_chat_service: Arc<dyn GroupChatServiceTrait>,
    pub (crate) insert_text_message_info_semaphore: Arc<Semaphore>,
}

impl TextMessageService {
    pub fn new(
        text_message_repo: Arc<dyn TextMessageRepositoryTrait>,
        group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
        group_chat_service: Arc<dyn GroupChatServiceTrait>,
    ) -> Self {
        Self {
            text_message_repo,
            group_membership_service,
            group_chat_service,
            insert_text_message_info_semaphore: Arc::new(Semaphore::new(1)),
        }
    }
    
    pub fn text_message_repo(&self) -> Arc<dyn TextMessageRepositoryTrait> {
        Arc::clone(&self.text_message_repo)
    }

    pub fn group_membership_service(&self) -> Arc<dyn GroupMembershipServiceTrait> {
        Arc::clone(&self.group_membership_service)
    }

    pub fn group_chat_service(&self) -> Arc<dyn GroupChatServiceTrait> {
        Arc::clone(&self.group_chat_service)
    }
}
