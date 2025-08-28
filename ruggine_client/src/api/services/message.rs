use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::config::endpoints::ApiEndpoints;
use crate::types::message::{
    TextMessageCreateRequest, TextMessageReadDto, PaginatedTextMessageResponse, Message, MessagePage
};
use serde_json::json;

#[derive(Clone)]
pub struct MessageService {
    http_client: ApiClient,
    storage_service: StorageService,
}

impl MessageService {
    pub fn new(http_client: ApiClient, storage_service: StorageService) -> Self {
        Self { http_client, storage_service }
    }

    /// Crea un nuovo messaggio di testo
    pub async fn create_message(&self, req: &TextMessageCreateRequest) -> Result<Message, AuthError> {
        let resp: crate::types::common::ApiSuccessResponse<TextMessageReadDto> =
            self.http_client.post(ApiEndpoints::TEXT_MESSAGE_CREATE, req).await.map_err(AuthError::from)?;
        Ok(Message::from(resp.data))
    }

    /// Recupera i messaggi di un gruppo (paginati)
    pub async fn get_messages_by_group(
        &self,
        group_chat_id: i32,
        cursor: Option<String>,
        limit: Option<i32>,
    ) -> Result<MessagePage, AuthError> {
        let mut url = ApiEndpoints::text_messages_by_group(&group_chat_id.to_string());
        let mut params = vec![];
        if let Some(cursor) = cursor {
            params.push(("cursor", cursor));
        }
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if !params.is_empty() {
            let query = serde_urlencoded::to_string(&params).unwrap();
            url = format!("{}?{}", url, query);
        }
        let resp: PaginatedTextMessageResponse = self.http_client.get(&url).await.map_err(AuthError::from)?;
        Ok(MessagePage::from(resp))
    }
    /// Recupera i messaggi non letti di un gruppo
    pub async fn get_messages_not_read_yet(
        &self,
        group_chat_id: i32,
    ) -> Result<MessagePage, AuthError> {
        let url = ApiEndpoints::text_messages_not_read_yet(&group_chat_id.to_string());
        let resp: PaginatedTextMessageResponse = self.http_client.get(&url).await.map_err(AuthError::from)?;
        Ok(MessagePage::from(resp))
    }

    /// Aggiorna la lettura di un messaggio (update read_at)
    pub async fn update_message_read_at(
        &self,
        text_message_id: i32,
        read_at: String,
    ) -> Result<(), AuthError> {
        use crate::types::message::TextMessageInfoReadAtDtoUpdate;
        let req = TextMessageInfoReadAtDtoUpdate { text_message_id, read_at };
        // La risposta non viene usata, ma si può gestire se serve
        let _resp: crate::types::message::ApiSuccessResponseUserReadDto =
            self.http_client.patch(ApiEndpoints::TEXT_MESSAGE_UPDATE_READ_AT, &req).await.map_err(AuthError::from)?;

        Ok(())
    }
}
