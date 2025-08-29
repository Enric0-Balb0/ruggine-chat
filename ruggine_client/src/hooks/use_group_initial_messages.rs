
use leptos::*;
use crate::types::message::Message;
use crate::api::services::message::MessageService;
use crate::utils::storage::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;

/// Hook to fetch the initial (latest N) messages for a group chat on mount.
pub fn use_group_initial_messages(
    group_chat_id: i32,
    limit: i32,
) -> (
    ReadSignal<Vec<Message>>, // messages
    ReadSignal<bool>,         // initial loading
    ReadSignal<Option<String>>, // error
    std::rc::Rc<dyn Fn()>,   // load_more closure
    ReadSignal<bool>,        // loading_more
    ReadSignal<bool>,        // has_more
) {
    let (messages, set_messages) = create_signal(Vec::<Message>::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    let (loading_more, set_loading_more) = create_signal(false);
    let (has_more, set_has_more) = create_signal(false);
    // Store next cursor as string (date-time)
    let (next_cursor, set_next_cursor) = create_signal(None::<String>);

    create_effect(move |_| {
        set_loading.set(true);
        set_error.set(None);
        // initial fetch
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
                    set_messages.set(msgs);
                    // store pagination
                    set_has_more.set(page.pagination.has_more);
                    set_next_cursor.set(page.pagination.next_cursor.map(|dt| dt.to_rfc3339()));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Errore: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    });

    // load_more closure: fetch older messages using next_cursor and prepend
    let load_more = {
        let next_cursor = next_cursor.clone();
        let has_more = has_more.clone();
        let loading_more = loading_more.clone();
        let set_loading_more = set_loading_more.clone();
        let set_messages = set_messages.clone();
        let set_next_cursor = set_next_cursor.clone();
        let set_has_more = set_has_more.clone();
        std::rc::Rc::new(move || {
            // if no more pages or already loading, skip
            if !has_more.get() || loading_more.get() {
                return;
            }
            set_loading_more.set(true);
            let cursor_opt = next_cursor.get();
            let cursor_clone = cursor_opt.clone();
            spawn_local(async move {
                let storage_service = StorageService::new();
                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                if let Some(token_response) = storage_service.get_token() {
                    http_client.set_auth_token(Some(token_response.token));
                }
                let service = MessageService::new(http_client, storage_service);
                match service.get_messages_by_group(group_chat_id, cursor_clone.clone(), Some(limit)).await {
                    Ok(page) => {
                        let mut new_msgs = page.data.clone();
                        // prepend older messages before existing ones
                        let existing = messages.get();
                        new_msgs.extend(existing.clone());
                        set_messages.set(new_msgs);
                        set_has_more.set(page.pagination.has_more);
                        set_next_cursor.set(page.pagination.next_cursor.map(|dt| dt.to_rfc3339()));
                    }
                    Err(_e) => {
                        // ignore errors for load more for now
                    }
                }
                set_loading_more.set(false);
            });
        })
    };

    (
        messages,
        loading,
        error,
        load_more,
        loading_more,
        has_more,
    )
}
