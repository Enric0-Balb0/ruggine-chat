use std::sync::Arc;
use crate::repository::text_message_repository::TextMessageRepositoryTrait;

#[derive(Clone)]
pub struct TextMessageService {
    pub(crate) text_message_repo: Arc<dyn TextMessageRepositoryTrait>,
}

impl TextMessageService {
    pub fn new(text_message_repo: Arc<dyn TextMessageRepositoryTrait>) -> Self {
        Self {
            text_message_repo,
        }
    }
    
    pub fn text_message_repo(&self) -> Arc<dyn TextMessageRepositoryTrait> {
        Arc::clone(&self.text_message_repo)
    }
}
