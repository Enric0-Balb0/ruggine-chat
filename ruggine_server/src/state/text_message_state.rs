use crate::config::database::Database;
use crate::service::text_message_service::{TextMessageService, TextMessageServiceTrait};
use std::sync::Arc;
use crate::utils::service_initializer::ServiceInitializer;

#[derive(Clone)]
pub struct TextMessageState {
    pub text_message_service: Arc<dyn TextMessageServiceTrait>,
}

impl TextMessageState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let service_init = ServiceInitializer::new(db_conn);
        let text_message_service = service_init.text_message_service();

        Self {
            text_message_service,
        }
    }

    pub fn with_dependencies(
        text_message_service: Arc<dyn TextMessageServiceTrait>,
    ) -> Self {
        Self {
            text_message_service,
        }
    }
}
