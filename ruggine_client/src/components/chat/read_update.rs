use leptos::*;
use std::collections::HashMap;

// Batch parameters for mark-as-read updates
pub const UPDATE_DEBOUNCE_MS: u64 = 200; // wait this long to coalesce visible ids
pub const UPDATE_BATCH_SIZE: usize = 8; // send at most this many updates per flush
pub const UPDATE_INTER_CALL_MS: u64 = 100; // delay between individual update calls to avoid burst

/// Process a single batch of message ids by calling update_message_read_at sequentially.
/// Updates local unread counts and removes ids from the initial unread map on success.
pub async fn process_update_batch(
    batch: Vec<i32>,
    unread_counts: RwSignal<HashMap<i32, u32>>,
    unread_message_ids: RwSignal<HashMap<i32, Vec<i32>>>,
    group_id: i32,
) {
    use crate::api::client::ApiClient;
    use crate::config::constants::AppConstants;
    use crate::utils::storage::StorageService;
    use crate::api::services::message::MessageService;
    use crate::components::chat::scroll_helpers::decrement_unread_for_group;

    for id in batch.into_iter().rev() {
        let id_clone = id;
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        if let Some(token) = storage_service.get_token() {
            http_client.set_auth_token(Some(token.token));
        }
        let message_service = MessageService::new(http_client, storage_service);
        let now = chrono::Utc::now().to_rfc3339();
        let res = message_service.update_message_read_at(id_clone, now).await;
        match res {
            Ok(()) => {
                // optimistic decrement locally
                decrement_unread_for_group(&unread_counts, group_id);
                // remove id from initial unread map if present
                unread_message_ids.update(|map| {
                    if let Some(vec_ids) = map.get_mut(&group_id) {
                        vec_ids.retain(|x| *x != id_clone);
                    }
                });
            }
            Err(_e) => {
                // ignore errors here; batch will retry when visibility triggers again
            }
        }
        // small pause to avoid bursting the server
        crate::utils::timers::sleep_ms(UPDATE_INTER_CALL_MS).await;
    }
    // After processing the batch, reconcile with authoritative server value to handle multi-device cases.
    // Fire-and-forget: spawn a task to fetch the authoritative not-read-yet list and overwrite local count.
    {
        let unread_counts_clone = unread_counts.clone();
        let unread_message_ids_clone = unread_message_ids.clone();
        leptos::spawn_local(async move {
            use crate::context::unread_counts_context::refresh_unread_for_group;
            if let Some(count) = refresh_unread_for_group(group_id).await {
                // already applied inside refresh_unread_for_group via context, but ensure local variables are consistent
                // (no-op here since refresh_mut updates the context directly)
                let _ = (unread_counts_clone, unread_message_ids_clone, count);
            }
        });
    }
}
