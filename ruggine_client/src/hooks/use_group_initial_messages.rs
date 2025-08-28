
use leptos::*;
use crate::types::message::Message;
use crate::api::services::message::MessageService;
use crate::utils::storage::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;

/// Hook to fetch the initial (latest N) messages for a group chat on mount.
pub fn use_group_initial_messages(group_chat_id: i32, limit: i32) -> (ReadSignal<Vec<Message>>, ReadSignal<bool>, ReadSignal<Option<String>>) {
    let (messages, set_messages) = create_signal(Vec::<Message>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);

    create_effect(move |_| {
        set_loading.set(true);
        set_error.set(None);
    // debug: fetching initial messages for group
        spawn_local(async move {
            let storage_service = StorageService::new();
            let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
            if let Some(token_response) = storage_service.get_token() {
                http_client.set_auth_token(Some(token_response.token));
            }
            let service = MessageService::new(http_client, storage_service);
            match service.get_messages_by_group(group_chat_id, None, Some(limit)).await {
                Ok(page) => {
                    let msgs = page.data.clone();
                    // debug: fetch ok
                    set_messages.set(msgs);
                    set_loading.set(false);
                }
                Err(e) => {
                    // debug: fetch error: {e}
                    set_error.set(Some(format!("Errore: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    });

    (messages, loading, error)
}
