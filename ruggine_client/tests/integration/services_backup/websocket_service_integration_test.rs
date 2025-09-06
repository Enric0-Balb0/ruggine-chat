use std::collections::HashMap;

use ruggine_client_ui::hooks::unread_helpers::increment_unread_map;

#[test]
fn test_increment_unread_map_increments_for_other_user() {
    let mut map: HashMap<i32, u32> = HashMap::new();
    increment_unread_map(&mut map, 42, Some(10), Some(1));
    assert_eq!(map.get(&42).cloned().unwrap_or(0), 1);

    // increment again
    increment_unread_map(&mut map, 42, Some(11), Some(1));
    assert_eq!(map.get(&42).cloned().unwrap_or(0), 2);
}

#[test]
fn test_increment_unread_map_ignores_current_user_messages() {
    let mut map: HashMap<i32, u32> = HashMap::new();
    increment_unread_map(&mut map, 7, Some(5), Some(5));
    assert_eq!(map.get(&7).cloned().unwrap_or(0), 0);
}

#[test]
fn test_increment_unread_map_handles_none_sender() {
    let mut map: HashMap<i32, u32> = HashMap::new();
    // no sender id => treat as external (increment)
    increment_unread_map(&mut map, 9, None, Some(1));
    assert_eq!(map.get(&9).cloned().unwrap_or(0), 1);
}
