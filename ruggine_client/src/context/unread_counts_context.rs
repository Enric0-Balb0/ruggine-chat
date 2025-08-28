use leptos::*;
use std::collections::HashMap;

/// Context for managing unread message counts per group
#[derive(Clone)]
pub struct UnreadCountsContext {
    pub unread_counts: RwSignal<HashMap<i32, u32>>, // group_id -> unread count
}

/// Provide the unread counts context to the app
pub fn provide_unread_counts_context() -> RwSignal<HashMap<i32, u32>> {
    let unread_counts = create_rw_signal::<HashMap<i32, u32>>(HashMap::new());
    // Noisy debug effect removed; keep the signal provisioning lightweight.
    provide_context(UnreadCountsContext {
        unread_counts: unread_counts.clone(),
    });
    unread_counts
}

/// Access the unread counts context
pub fn use_unread_counts_context() -> RwSignal<HashMap<i32, u32>> {
    use_context::<UnreadCountsContext>()
        .expect("UnreadCountsContext not found!")
        .unread_counts
}
