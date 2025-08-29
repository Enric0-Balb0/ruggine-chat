use std::collections::HashMap;

/// Increment unread count for a group if the sender is not the current user.
/// Pure helper so it can be unit/integration tested without leptos runtime.
pub fn increment_unread_map(map: &mut HashMap<i32, u32>, group_id: i32, sender_id: Option<i32>, current_user_id: Option<i32>) {
    if sender_id.is_some() && Some(sender_id.unwrap()) == current_user_id {
        // message from current user: do nothing
        return;
    }

    let prev = map.get(&group_id).cloned().unwrap_or(0);
    map.insert(group_id, prev.saturating_add(1));
}
