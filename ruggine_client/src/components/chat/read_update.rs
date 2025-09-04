use leptos::*;
use std::collections::HashMap;

// Batch parameters for mark-as-read updates
pub const UPDATE_DEBOUNCE_MS: u64 = 100; // wait this long to coalesce visible ids (reduced for snappier updates)
pub const UPDATE_BATCH_SIZE: usize = 16; // send at most this many updates per flush (larger batches)
pub const UPDATE_INTER_CALL_MS: u64 = 20; // small delay between calls to reduce short bursts
pub const UPDATE_CONCURRENCY: usize = 4; // number of concurrent requests to run when flushing a batch

/// Process a single batch of message ids by calling update_message_read_at sequentially.
/// Updates local unread counts and removes ids from the initial unread map on success.
pub async fn process_update_batch(
    batch: Vec<i32>,
    unread_counts: RwSignal<HashMap<i32, u32>>,
    unread_message_ids: RwSignal<HashMap<i32, Vec<i32>>>,
    unread_marked_read: RwSignal<std::collections::HashMap<i32, std::collections::HashSet<i32>>>,
    group_id: i32,
) {
    use crate::api::client::ApiClient;
    use crate::config::constants::AppConstants;
    use crate::utils::storage::StorageService;
    use crate::api::services::message::MessageService;
    use crate::components::chat::scroll_helpers::decrement_unread_for_group;

    // To speed up updates we run several requests concurrently in controlled chunks.
    // This keeps the server from being flooded while reducing overall latency.
    let mut to_send: Vec<i32> = Vec::new();
    for id in batch.into_iter().rev() {
        let already_marked = {
            let map = unread_marked_read.get_untracked();
            map.get(&group_id).map(|s| s.contains(&id)).unwrap_or(false)
        };
        if !already_marked {
            to_send.push(id);
        }
    }

    use futures::future::join_all;
    // process in chunks of UPDATE_CONCURRENCY
    let mut idx = 0usize;
    while idx < to_send.len() {
        let end = (idx + UPDATE_CONCURRENCY).min(to_send.len());
        let slice = &to_send[idx..end];
        // build futures for this chunk
        let mut futs = Vec::with_capacity(slice.len());
        for &id in slice.iter() {
            let id_clone = id;
            // create a future that performs the request and applies local updates on success
            let fut = async move {
                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                let storage_service = StorageService::new();
                if let Some(token) = storage_service.get_token() {
                    http_client.set_auth_token(Some(token.token));
                }
                let message_service = MessageService::new(http_client, storage_service);
                let now = chrono::Utc::now().to_rfc3339();
                let res = message_service.update_message_read_at(id_clone, now).await;
                (id_clone, res)
            };
            futs.push(fut);
        }

        // await concurrent chunk
        let results = join_all(futs).await;
        for (id, res) in results.into_iter() {
            if res.is_ok() {
                // optimistic decrement locally
                decrement_unread_for_group(&unread_counts, group_id);
                // remove id from initial unread map if present
                unread_message_ids.update(|map| {
                    if let Some(vec_ids) = map.get_mut(&group_id) {
                        vec_ids.retain(|x| *x != id);
                    }
                });
                // record locally that this id has been marked as read
                unread_marked_read.update(|map| {
                    let set = map.entry(group_id).or_insert_with(|| std::collections::HashSet::new());
                    set.insert(id);
                });
            }
            // else ignore; will retry later
        }

        idx = end;
        // small pause between chunks to avoid bursts
        crate::utils::timers::sleep_ms(UPDATE_INTER_CALL_MS).await;
    }
    // NOTE: previously we reconciled the local unread counts with the server after
    // processing a batch. That behavior overwrote local deltas coming from the WS and
    // caused unexpected restores of the count. Per new requirements, unread counts are
    // managed manually: +1 on incoming WS messages and -1 when an update_message_read_at
    // call succeeds. Do not perform an automatic authoritative refresh here.
}
