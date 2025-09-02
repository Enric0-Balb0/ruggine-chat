use leptos::*;
use std::time::Duration;

pub struct UnreadRefresherHandle {
    stopped: RwSignal<bool>,
}

impl UnreadRefresherHandle {
    pub fn stop(&self) {
        self.stopped.set(true);
    }
}

/// Start a background refresher for the provided group ids.
/// Returns a handle that can be used to stop the poller.
pub fn start_unread_refresher(group_ids: Vec<i32>, interval_ms: u64) -> UnreadRefresherHandle {
    // NOTE: continuous polling removed by request. This function now performs a
    // one-shot authoritative refresh for the provided group ids and returns a
    // handle with a stopped flag for API compatibility.
    let stopped = create_rw_signal(true);
    let ids = group_ids.clone();
    spawn_local(async move {
        use crate::context::unread_counts_context::refresh_unread_for_group;
        for gid in ids.into_iter() {
            let _ = refresh_unread_for_group(gid).await;
        }
    });
    UnreadRefresherHandle { stopped }
}

/// Force a refresh for a single group (fire-and-forget)
pub fn force_refresh_for_group(group_id: i32) {
    spawn_local(async move {
        use crate::context::unread_counts_context::refresh_unread_for_group;
        let _ = refresh_unread_for_group(group_id).await;
    });
}
