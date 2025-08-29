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
