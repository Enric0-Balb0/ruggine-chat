use leptos::*;
use std::collections::HashMap;

/// Context for managing unread message counts per group
#[derive(Clone)]
pub struct UnreadCountsContext {
    pub unread_counts: RwSignal<HashMap<i32, u32>>, // group_id -> unread count
    // Track message ids returned by get_messages_not_read_yet per group so we can
    // selectively call update_message_read_at only for those messages (or for
    // new socket messages that are not part of the initial set).
    pub unread_message_ids: RwSignal<HashMap<i32, Vec<i32>>>, // group_id -> vec![message_id]
}

/// Provide the unread counts context to the app
pub fn provide_unread_counts_context() -> RwSignal<HashMap<i32, u32>> {
    let unread_counts = create_rw_signal::<HashMap<i32, u32>>(HashMap::new());
    let unread_message_ids = create_rw_signal::<HashMap<i32, Vec<i32>>>(HashMap::new());
    // Noisy debug effect removed; keep the signal provisioning lightweight.
    provide_context(UnreadCountsContext {
        unread_counts: unread_counts.clone(),
        unread_message_ids: unread_message_ids.clone(),
    });
    unread_counts
}

/// Access the unread counts context
pub fn use_unread_counts_context() -> RwSignal<HashMap<i32, u32>> {
    use_context::<UnreadCountsContext>()
        .expect("UnreadCountsContext not found!")
        .unread_counts
}

/// Access the unread message ids context (map group_id -> vec![message_id])
pub fn use_unread_message_ids_context() -> RwSignal<HashMap<i32, Vec<i32>>> {
    use_context::<UnreadCountsContext>()
        .expect("UnreadCountsContext not found!")
        .unread_message_ids
}

/// Refresh unread count and ids for a specific group from the server and overwrite local signals.
pub async fn refresh_unread_for_group(group_id: i32) -> Option<u32> {
    use crate::api::client::ApiClient;
    use crate::config::constants::AppConstants;
    use crate::api::services::message::MessageService;
    use crate::utils::storage::StorageService;

    // Try to access the context; if not present, skip.
    let ctx = match use_context::<UnreadCountsContext>() {
        Some(c) => c,
        None => return None,
    };

    // Build client
    let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
    let storage = StorageService::new();
    if let Some(token) = storage.get_token() {
        http_client.set_auth_token(Some(token.token));
    }
    let message_service = MessageService::new(http_client, storage);

    // Call the endpoint that returns not-read-yet messages for the group
    match message_service.get_messages_not_read_yet(group_id).await {
        Ok(paginated) => {
            let msgs = paginated.data;
            let count = msgs.len() as u32;

            // overwrite unread_counts and unread_message_ids
            ctx.unread_counts.update(|map| {
                map.insert(group_id, count);
            });
            ctx.unread_message_ids.update(|map| {
                map.insert(group_id, msgs.iter().map(|m| m.id).collect());
            });
            Some(count)
        }
        Err(_) => None,
    }
}
