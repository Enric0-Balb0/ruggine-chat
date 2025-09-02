use leptos::*;
use leptos::html::Div;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use crate::hooks::GroupMembershipWithDetails;
use crate::api::services::GroupMembershipService;
use crate::components::use_toast;
use leptos_router::use_navigate;
use crate::hooks::use_groups_context;
use crate::components::{InviteMemberModal, InviteMemberRequest, MessageInputArea, GroupDetailsModal, LucideIcon};
use crate::hooks::use_group_socket_messages::use_group_socket_messages;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;
use crate::hooks::use_group_initial_messages::use_group_initial_messages;
use crate::context::unread_counts_context::use_unread_counts_context;
use crate::components::chat::chat_message::{ChatMessage, MessageStatus};
use crate::types::message::Message;
use crate::utils::storage::StorageService;
use crate::hooks::use_group_user_cache::use_group_user_cache;
use crate::hooks::fetch_missing_users::fetch_missing_users;
use crate::api::services::UserService;
use super::{compute_bottom_aligned_scroll, is_element_in_viewport, decrement_unread_for_group, process_update_batch, UPDATE_BATCH_SIZE, UPDATE_DEBOUNCE_MS};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChatHeaderAction {
    InviteMembers,
    GroupDetails,
    LeaveGroup,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropdownState {
    Closed,
    Opening,
    Open,
    Closing,
}

#[component]
pub fn ChatView(
    #[prop(into)] group_data: GroupMembershipWithDetails,
) -> impl IntoView {
    // Signal for messages managed by the new hook
    use leptos::use_context;
    use crate::hooks::use_group_message_ws::UseGroupMessageWs;
    let ws_ctx = use_context::<Option<UseGroupMessageWs>>();
    // Signal per messaggi locali inviati via input
    let (local_messages, set_local_messages) = create_signal(Vec::<Message>::new());
    let set_local_messages_rc = Rc::new(set_local_messages);
    let unread_counts = use_unread_counts_context();
    // Also access the unread message ids map so we can selectively call update_message_read_at
    use crate::context::unread_counts_context::use_unread_message_ids_context;
    let unread_message_ids = use_unread_message_ids_context();
    let group_id_for_update = group_data.membership.group_chat_id;
    let add_message: Rc<dyn Fn(Message)> = {
        let set_local_messages_rc = Rc::clone(&set_local_messages_rc);
        let unread_counts = unread_counts.clone();
        let group_id = group_id_for_update;
        Rc::new(move |msg: Message| {
            set_local_messages_rc.update(|msgs| msgs.push(msg.clone()));
            // After sending, call update_message_read_at and decrement the counter using clone-set
            let unread_counts = unread_counts.clone();
            let msg_id = msg.id;
            leptos::spawn_local(async move {
                use crate::api::services::message::MessageService;
                use crate::config::constants::AppConstants;
                use crate::utils::storage::StorageService;
                use crate::api::client::ApiClient;
                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                let storage_service = StorageService::new();
                if let Some(token) = storage_service.get_token() {
                    http_client.set_auth_token(Some(token.token));
                }
                let message_service = MessageService::new(http_client, storage_service);
                let now = chrono::Utc::now().to_rfc3339();
                // Log before attempting update
                let res = message_service.update_message_read_at(msg_id, now).await;
                match res {
                    Ok(()) => {
                        // success: update local state
                        // On success, decrement the unread counter for the group
                        decrement_unread_for_group(&unread_counts, group_id);
                        // Also remove the message id from the initial unread ids map if present
                        unread_message_ids.update(|map| {
                            if let Some(vec_ids) = map.get_mut(&group_id) {
                                vec_ids.retain(|id| *id != msg_id);
                            }
                        });
                    }
                    Err(e) => {
                        // ignore error silently; batching flow will retry for buffered ids
                    }
                }
            });
        })
    };
    let (initial_messages, initial_loading, initial_error, load_more, loading_more, has_more) = use_group_initial_messages(group_data.membership.group_chat_id, 50);
    let user_cache = use_group_user_cache(group_data.membership.group_chat_id);

    // (diagnostics removed)

    let ws_messages = use_group_socket_messages(
        group_data.membership.group_chat_id,
        ws_ctx.as_ref().and_then(|w| w.as_ref().cloned()),
        local_messages,
    );
    use std::collections::HashSet;
    use crate::types::message_ws::{WebSocketMessage, ServerEvent, GroupEvent};
    // Presence tracking: we initialize from the API and keep WS-derived deltas.
    // Keep two sets and compute their union for the UI count so that if other clients
    // are already online (before WS events arrive) we still display them.
    let ws_ctx_for_presence = ws_ctx.clone();
    let (ws_online_user_ids, set_ws_online_user_ids) = create_signal(HashSet::<i32>::new());
    let (initial_online_user_ids, set_initial_online_user_ids) = create_signal(HashSet::<i32>::new());

    // Rebuild WS-derived set from socket events (this represents realtime joins/lefts)
    {
        let ws_ctx_for_presence = ws_ctx_for_presence.clone();
        let set_ws_online_user_ids = set_ws_online_user_ids.clone();
        create_effect(move |_| {
            if let Some(Some(ws)) = ws_ctx_for_presence.as_ref() {
                let msgs = ws.messages.get();
                let mut set: HashSet<i32> = HashSet::new();
                for msg in msgs.into_iter() {
                    if let WebSocketMessage::Event { event, .. } = msg {
                        if let ServerEvent::Groups(group_event) = event {
                            match group_event {
                                GroupEvent::Joined { user_id } => { set.insert(user_id); },
                                GroupEvent::Left { user_id } => { set.remove(&user_id); },
                                _ => (),
                            }
                        }
                    }
                }
                leptos::logging::log!("[WS DEBUG] Rebuilt WS-derived online set for group {} => {:?}", group_data.membership.group_chat_id, set);
                set_ws_online_user_ids.set(set);
            }
        });
    }

    // On mount, fetch the current connected/online users from the API so we don't miss
    // users that were already connected before this client opened the app.
    {
        let set_initial = set_initial_online_user_ids.clone();
        leptos::spawn_local(async move {
            use crate::utils::storage::StorageService;
            let storage = StorageService::new();
            let mut http = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
            if let Some(token_response) = storage.get_token() {
                http.set_auth_token(Some(token_response.token));
            }
            let membership_service = GroupMembershipService::new(http, storage);
            match membership_service.find_connected_users_and_online().await {
                Ok(ids) => {
                    let set: HashSet<i32> = ids.into_iter().collect();
                    leptos::logging::log!("[CHAT] initial online ids => {:?}", set);
                    set_initial.set(set);
                }
                Err(e) => {
                    leptos::logging::log!("[CHAT] failed to load initial online ids: {:?}", e);
                }
            }
        });
    }

    // Combined online count: union of API-initialized set and WS-derived set, +1 for self
    let online_count = create_memo(move |_| {
        let mut union_set = HashSet::<i32>::new();
        for id in initial_online_user_ids.get().iter() { union_set.insert(*id); }
        for id in ws_online_user_ids.get().iter() { union_set.insert(*id); }
        let mut count = union_set.len();
        let storage = StorageService::new();
        if let Some(user) = storage.get_user_profile() {
            if !union_set.contains(&user.id) {
                count += 1;
            }
        }
        count
    });

    // Use unread counts context for badge decrement
    let unread_counts = use_unread_counts_context();

    // decrement_unread_for_group is provided by scroll_helpers
    let messages = create_memo(move |_| {
        let mut all_msgs = Vec::new();
        all_msgs.extend(initial_messages.get());
        all_msgs.extend(ws_messages.get());
        all_msgs.extend(local_messages.get());
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for msg in all_msgs {
            map.insert(msg.id, msg);
        }
        let mut deduped: Vec<_> = map.into_values().collect();
        deduped.sort_by_key(|m| m.sent_at);
        deduped
    });
    // Local UI guards and signals used by scrolling / anchoring logic
    let (scrolled_initial, set_scrolled_initial) = create_signal(false);
    let (first_unread_anchor, set_first_unread_anchor) = create_signal(None::<i32>);
    let (is_loading_local, set_is_loading_local) = create_signal(false);
    let (suppress_scroll_events, set_suppress_scroll_events) = create_signal(false);
    // Show a floating "scroll to bottom" button when the user scrolled up
    let (show_scroll_to_bottom, set_show_scroll_to_bottom) = create_signal(false);
    // Buffer for pending mark-as-read updates
    let pending_update_ids: std::rc::Rc<std::cell::RefCell<Vec<i32>>> = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    // Prev scroll metrics for restoring viewport when prepending older messages
    let (prev_scroll_top, set_prev_scroll_top) = create_signal(0i32);
    let (prev_scroll_height, set_prev_scroll_height) = create_signal(0i32);
    let (dropdown_state, set_dropdown_state) = create_signal(DropdownState::Closed);
    let (invite_modal_open, set_invite_modal_open) = create_signal(false);
    let (group_details_modal_open, set_group_details_modal_open) = create_signal(false);
    let (leave_modal_open, set_leave_modal_open) = create_signal(false);
    let (leave_error, set_leave_error) = create_signal(None::<String>);
    let (leave_loading, set_leave_loading) = create_signal(false);

    let dropdown_ref = create_node_ref::<Div>();
    let messages_container_ref = create_node_ref::<Div>();
    // Strong guard to lock other programmatic scrolls while we anchor to the first-unread position
    let anchor_scroll_locked: std::rc::Rc<std::cell::Cell<bool>> = std::rc::Rc::new(std::cell::Cell::new(false));
    {
        let local_messages_for_scroll = local_messages.clone();
        let messages_container_ref_for_scroll = messages_container_ref.clone();
    let anchor_scroll_locked_for_send = anchor_scroll_locked.clone();
        create_effect(move |_| {
            let locals = local_messages_for_scroll.get();
            if locals.is_empty() {
                return;
            }
            // Get the last local message id
            let last_id = locals.last().unwrap().id;
            // Small timeout to allow the DOM to render the new message
            let anchor_scroll_locked_for_send_inner = anchor_scroll_locked_for_send.clone();
            set_timeout(move || {
                let doc = match web_sys::window() {
                    Some(w) => match w.document() { Some(d) => d, None => return },
                    None => return,
                };
                if let Some(elem) = doc.get_element_by_id(&format!("msg-{}", last_id)) {
                    if !anchor_scroll_locked_for_send_inner.get() {
                        let _ = elem.scroll_into_view_with_bool(true);
                    }
                } else if let Some(container) = messages_container_ref_for_scroll.get() {
                    // fallback: scroll container to bottom only if anchor lock not active
                    if !anchor_scroll_locked_for_send_inner.get() {
                        container.set_scroll_top(container.scroll_height());
                    }
                }
            }, std::time::Duration::from_millis(50));
        });
    }
    // Flag indicating whether a flush is scheduled
    let flush_scheduled: std::rc::Rc<std::cell::Cell<bool>> = std::rc::Rc::new(std::cell::Cell::new(false));
    {
        let messages = messages.clone();
        let messages_container_ref = messages_container_ref.clone();
        let initial_loading = initial_loading.clone();
        let set_scrolled_initial = set_scrolled_initial.clone();
        // clone load_more for this effect so we don't move the original `load_more`
        let load_more_init = load_more.clone();
        // clone the anchor lock for this effect so the original `anchor_scroll_locked` isn't moved
        let anchor_scroll_locked_for_effect = anchor_scroll_locked.clone();

        create_effect(move |_| {
            if initial_loading.get() || scrolled_initial.get() {
                return;
            }
            let msgs = messages.get_untracked();
            if msgs.is_empty() {
                return;
            }

            // Determine first unread id per the initial unread ids (server-provided)
            let unread_map = unread_message_ids.get_untracked();
            let maybe_initial_unread = unread_map.get(&group_id_for_update).cloned();

            // Helper to perform a DOM scroll to an element inside the container
            let anchor_lock_for_closure = anchor_scroll_locked_for_effect.clone();
            let anchor_for_scroll = anchor_scroll_locked_for_effect.clone();
            let scroll_elem_into_view = move |container: Option<leptos::HtmlElement<Div>>, elem_id: String| {
                let doc = match web_sys::window() {
                    Some(w) => match w.document() { Some(d) => d, None => return false },
                    None => return false,
                };
                if let Some(elem) = doc.get_element_by_id(&elem_id) {
                    let _ = elem.scroll_into_view_with_bool(true);
                    true
                } else if let Some(c) = container {
                    if !anchor_for_scroll.get() {
                        c.set_scroll_top(c.scroll_height());
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            // If there are initial unread ids, try to scroll to the earliest one that is loaded.
            if let Some(initial_ids) = maybe_initial_unread {
                if !initial_ids.is_empty() {
                    anchor_scroll_locked_for_effect.set(true);
                    if let Some(first_unread_msg) = msgs.iter().find(|m| initial_ids.contains(&m.id)) {
                        let first_id = first_unread_msg.id;
                        set_first_unread_anchor.set(Some(first_id));
                        anchor_lock_for_closure.set(true);
                        if let Some(container) = messages_container_ref.get() {
                            let did = scroll_elem_into_view(Some(container), format!("msg-{}", first_id));
                            if did {
                                set_scrolled_initial.set(true);
                                return;
                            }
                        } else {
                            let did = scroll_elem_into_view(None, format!("msg-{}", first_id));
                            if did {
                                set_scrolled_initial.set(true);
                                return;
                            }
                        }

                        // If element not present, attempt retries (load_more) up to a limit
                        let attempts = std::rc::Rc::new(std::cell::Cell::new(0));
                        let max_attempts = 8;
                        let messages_clone = messages.clone();
                        let messages_container_ref_clone = messages_container_ref.clone();
                        let load_more_clone = load_more_init.clone();
                        let scrolled_flag = set_scrolled_initial.clone();
                        let set_anchor_clone = set_first_unread_anchor.clone();
                        let unread_ids_clone = initial_ids.clone();

                        let attempts_clone = attempts.clone();
                        let anchor_scroll_locked_for_retry = anchor_lock_for_closure.clone();
                        set_timeout(move || {
                            let mut loop_continue = false;
                            let current_msgs = messages_clone.get_untracked();
                            if let Some(found) = current_msgs.iter().find(|m| unread_ids_clone.contains(&m.id)) {
                                set_anchor_clone.set(Some(found.id));
                                if let Some(container) = messages_container_ref_clone.get() {
                                    let doc = match web_sys::window() {
                                        Some(w) => match w.document() { Some(d) => d, None => return },
                                        None => return,
                                    };
                                    if let Some(elem) = doc.get_element_by_id(&format!("msg-{}", found.id)) {
                                        let desired = compute_bottom_aligned_scroll(&container, &elem);
                                        container.set_scroll_top(desired);
                                        let container_clone_r = container.clone();
                                        let elem_id_r = format!("msg-{}", found.id);
                                        set_timeout(move || {
                                            let doc = match web_sys::window() {
                                                Some(w) => match w.document() { Some(d) => d, None => return },
                                                None => return,
                                            };
                                            if let Some(elem) = doc.get_element_by_id(&elem_id_r) {
                                                let desired = compute_bottom_aligned_scroll(&container_clone_r, &elem);
                                                container_clone_r.set_scroll_top(desired);
                                            }
                                        }, std::time::Duration::from_millis(120));
                                        let container_clone_r2 = container.clone();
                                        let elem_id_r2 = format!("msg-{}", found.id);
                                        set_timeout(move || {
                                            let doc = match web_sys::window() {
                                                Some(w) => match w.document() { Some(d) => d, None => return },
                                                None => return,
                                            };
                                            if let Some(elem) = doc.get_element_by_id(&elem_id_r2) {
                                                let desired = compute_bottom_aligned_scroll(&container_clone_r2, &elem);
                                                container_clone_r2.set_scroll_top(desired);
                                            }
                                        }, std::time::Duration::from_millis(300));
                                    } else {
                                        if !anchor_lock_for_closure.get() {
                                            container.set_scroll_top(container.scroll_height());
                                        }
                                    }
                                }
                                anchor_lock_for_closure.set(true);
                                scrolled_flag.set(true);
                                return;
                            }

                            let mut att = attempts_clone.get();
                            att += 1;
                            attempts_clone.set(att);
                            if att <= max_attempts {
                                (load_more_clone)();
                                loop_continue = true;
                            }

                            if loop_continue {
                                let attempts_clone2 = attempts_clone.clone();
                                let messages_clone2 = messages_clone.clone();
                                let messages_container_ref2 = messages_container_ref_clone.clone();
                                let scrolled_flag2 = scrolled_flag.clone();
                                let unread_ids_clone2 = unread_ids_clone.clone();
                                let anchor_scroll_locked_for_retry2 = anchor_lock_for_closure.clone();
                                set_timeout(move || {
                                    let current_msgs2 = messages_clone2.get_untracked();
                                    if let Some(found2) = current_msgs2.iter().find(|m| unread_ids_clone2.contains(&m.id)) {
                                        set_anchor_clone.set(Some(found2.id));
                                        if let Some(container) = messages_container_ref2.get() {
                                            let doc = match web_sys::window() {
                                                Some(w) => match w.document() { Some(d) => d, None => return },
                                                None => return,
                                            };
                                            if let Some(elem) = doc.get_element_by_id(&format!("msg-{}", found2.id)) {
                                                let desired = compute_bottom_aligned_scroll(&container, &elem);
                                                container.set_scroll_top(desired);
                                                let container_clone_r = container.clone();
                                                let elem_id_r = format!("msg-{}", found2.id);
                                                set_timeout(move || {
                                                    let doc = match web_sys::window() {
                                                        Some(w) => match w.document() { Some(d) => d, None => return },
                                                        None => return,
                                                    };
                                                    if let Some(elem) = doc.get_element_by_id(&elem_id_r) {
                                                        let desired = compute_bottom_aligned_scroll(&container_clone_r, &elem);
                                                        container_clone_r.set_scroll_top(desired);
                                                    }
                                                }, std::time::Duration::from_millis(120));
                                                let container_clone_r2 = container.clone();
                                                let elem_id_r2 = format!("msg-{}", found2.id);
                                                set_timeout(move || {
                                                    let doc = match web_sys::window() {
                                                        Some(w) => match w.document() { Some(d) => d, None => return },
                                                        None => return,
                                                    };
                                                    if let Some(elem) = doc.get_element_by_id(&elem_id_r2) {
                                                        let desired = compute_bottom_aligned_scroll(&container_clone_r2, &elem);
                                                        container_clone_r2.set_scroll_top(desired);
                                                    }
                                                }, std::time::Duration::from_millis(300));
                                            } else {
                                                if !anchor_scroll_locked_for_retry.get() {
                                                    container.set_scroll_top(container.scroll_height());
                                                }
                                            }
                                        }
                                        anchor_scroll_locked_for_retry.set(true);
                                        scrolled_flag2.set(true);
                                        return;
                                    }
                                    if attempts_clone2.get() >= max_attempts {
                                        let msgs_now = messages_clone2.get_untracked();
                                        if !msgs_now.is_empty() {
                                            if let Some(container) = messages_container_ref2.get() {
                                                if !anchor_scroll_locked_for_retry2.get() {
                                                    if let Some(last_elem) = web_sys::window().unwrap().document().unwrap().get_element_by_id(&format!("msg-{}", msgs_now.last().unwrap().id)) {
                                                        let _ = last_elem.scroll_into_view_with_bool(true);
                                                    } else {
                                                        container.set_scroll_top(container.scroll_height());
                                                    }
                                                }
                                            }
                                            scrolled_flag2.set(true);
                                        }
                                    }
                                }, std::time::Duration::from_millis(180));
                            } else {
                                let msgs_now = messages_clone.get_untracked();
                                if !msgs_now.is_empty() {
                                    if let Some(container) = messages_container_ref_clone.get() {
                                        if !anchor_lock_for_closure.get() {
                                            if let Some(last_elem) = web_sys::window().unwrap().document().unwrap().get_element_by_id(&format!("msg-{}", msgs_now.last().unwrap().id)) {
                                                let _ = last_elem.scroll_into_view_with_bool(true);
                                            } else {
                                                container.set_scroll_top(container.scroll_height());
                                            }
                                        }
                                    }
                                    scrolled_flag.set(true);
                                }
                            }
                        }, std::time::Duration::from_millis(160));
                        return;
                    }
                }
            }

            // Default fallback: scroll to the last message
            let last_msg = msgs.last().unwrap();
            if let Some(_container) = messages_container_ref.get() {
                let doc = web_sys::window().unwrap().document().unwrap();
                if !anchor_lock_for_closure.get() {
                    if let Some(last_elem) = doc.get_element_by_id(&format!("msg-{}", last_msg.id)) {
                        let _ = last_elem.scroll_into_view_with_bool(true);
                        set_scrolled_initial.set(true);
                    }
                }
            }
        });
    }
    {
        use crate::api::services::message::MessageService;
        use crate::config::constants::AppConstants;
        use crate::utils::storage::StorageService;
        use crate::api::client::ApiClient;
        let messages = messages.clone();
        let messages_container_ref = messages_container_ref.clone();
        
    // clone guards for this scroll-effect so we don't move the originals
    // Prepare both read and write clones for local guards used by the scroll handler
    let is_loading_local_read = is_loading_local.clone();
    let is_loading_local_set = set_is_loading_local.clone();
    let suppress_scroll_events_read = suppress_scroll_events.clone();
    let suppress_scroll_events_set = set_suppress_scroll_events.clone();
                // no overflow manipulation; only use suppress guard
        create_effect(move |_| {
            if let Some(container) = messages_container_ref.get() {
                let container_clone = container.clone();
                let messages = messages.clone();
                // clone load_more Rc to use inside the closure without moving original
                let load_more = load_more.clone();
                let set_prev_scroll_top = set_prev_scroll_top.clone();
                let set_prev_scroll_height = set_prev_scroll_height.clone();

                // Threshold (pixels) from top to trigger loading more messages
                const LOAD_MORE_THRESHOLD: i32 = 150;

                let loading_more = loading_more.clone();
                let is_loading_local_cl = is_loading_local_read.clone();
                let suppress_scroll_events_cl = suppress_scroll_events_read.clone();

                // clone Rc handles for use inside the scroll-event closure
                let pending_for_closure = pending_update_ids.clone();
                let flush_for_closure = flush_scheduled.clone();
                let unread_counts_for_closure = unread_counts.clone();
                let unread_message_ids_for_closure = unread_message_ids.clone();

                let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    let doc = web_sys::window().unwrap().document().unwrap();
                    let msg_ids: Vec<i32> = messages.get_untracked().iter().map(|m| m.id).collect();

                    for msg_id in msg_ids {
                        if let Some(elem) = doc.get_element_by_id(&format!("msg-{}", msg_id)) {
                            if is_element_in_viewport(&container_clone, &elem) {
                                let mut should_update = false;
                                // check initial unread ids for the group
                                let initial_ids_map = unread_message_ids.get_untracked();
                                if let Some(initial_ids) = initial_ids_map.get(&group_id_for_update) {
                                    if initial_ids.contains(&msg_id) {
                                        should_update = true;
                                    }
                                }

                                // if not in initial set, check if it is a ws message (new) and not in initial set
                                if !should_update {
                                    let ws_ids: Vec<i32> = ws_messages.get_untracked().iter().map(|m| m.id).collect();
                                    if ws_ids.contains(&msg_id) {
                                        // only update if id is not part of the initial unread ids
                                        let in_initial = initial_ids_map.get(&group_id_for_update)
                                            .map(|v| v.contains(&msg_id)).unwrap_or(false);
                                        if !in_initial {
                                            should_update = true;
                                        }
                                    }
                                }

                                if should_update {
                                    // Buffer the id for batched updates instead of calling immediately
                                    {
                                        let mut buf = pending_for_closure.borrow_mut();
                                        if !buf.contains(&msg_id) {
                                            buf.push(msg_id);
                                        }
                                    }

                                    // schedule a flush if not already scheduled
                                    if !flush_for_closure.get() {
                                        let pending_clone = pending_for_closure.clone();
                                        let flush_flag = flush_for_closure.clone();
                                        let unread_counts_clone = unread_counts_for_closure.clone();
                                        let unread_message_ids_clone = unread_message_ids_for_closure.clone();
                                        flush_flag.set(true);

                                        // schedule the debounce timer
                                        set_timeout(move || {
                                            // spawn a local async task that drains the pending buffer in batches
                                            leptos::spawn_local(async move {
                                                loop {
                                                    // assemble a batch from the buffer
                                                    let mut batch: Vec<i32> = Vec::new();
                                                    {
                                                        let mut guard = pending_clone.borrow_mut();
                                                        if guard.is_empty() {
                                                            break;
                                                        }
                                                        for _ in 0..UPDATE_BATCH_SIZE.min(guard.len()) {
                                                            if let Some(id) = guard.pop() {
                                                                batch.push(id);
                                                            }
                                                        }
                                                    }
                                                    if batch.is_empty() {
                                                        break;
                                                    }
                                                    // delegate processing of the batch to the centralized helper
                                                    process_update_batch(batch, unread_counts_clone.clone(), unread_message_ids_clone.clone(), group_id_for_update).await;
                                                    // brief pause between batches
                                                    crate::utils::timers::sleep_ms(10).await;
                                                }
                                                // clear the scheduled flag
                                                flush_flag.set(false);
                                            });
                                        }, std::time::Duration::from_millis(UPDATE_DEBOUNCE_MS));
                                    }
                                } else {
                                    // skipped: not eligible for update
                                }
                            }
                        }
                    }

                    // If user scrolled near the top of the container, trigger load_more
                    // Use scrollTop on HtmlDivElement
                    let scroll_top = container_clone.scroll_top();
                    // Debug: log scroll metrics and loading state
                    let loading_now = loading_more.get_untracked();

                    // If we're suppressing programmatic scroll events, ignore
                    if suppress_scroll_events_cl.get_untracked() {
                        return;
                    }

                    // Trigger when near the top or at the top. Also avoid triggering while a load is in progress.
                    if scroll_top <= LOAD_MORE_THRESHOLD && !loading_now && !is_loading_local_cl.get_untracked() {
                        is_loading_local_set.set(true);
                        suppress_scroll_events_set.set(true);

                        // Fallback: ensure suppression is cleared eventually even if restore doesn't run
                        {
                            let suppress_fallback = suppress_scroll_events_set.clone();
                            set_timeout(move || {
                                suppress_fallback.set(false);
                            }, std::time::Duration::from_millis(1500));
                        }

                        // Fallback: clear local loading guard after a longer timeout in case of failures
                        {
                            let is_loading_fallback = is_loading_local_set.clone();
                            set_timeout(move || {
                                is_loading_fallback.set(false);
                            }, std::time::Duration::from_millis(5000));
                        }

                        // store current scroll metrics so we can restore after older messages are prepended
                        set_prev_scroll_top.set(scroll_top);
                        set_prev_scroll_height.set(container_clone.scroll_height());

                        // call load_more (Rc closure)
                        load_more();
                    }
                }) as Box<dyn FnMut(_)>);

                let _ = container.add_event_listener_with_callback("scroll", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        });
    }

    // Track container scroll to toggle the floating "scroll to bottom" button and
    // ensure we update the state both on scroll events and when messages change.
    {
        let messages_container_ref = messages_container_ref.clone();
        let set_show = set_show_scroll_to_bottom.clone();
        // Attach a scroll listener to update visibility
        create_effect(move |_| {
            if let Some(container) = messages_container_ref.get() {
                let container_clone = container.clone();
                let set_show_clone = set_show.clone();
                let scroll_closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    let scroll_top = container_clone.scroll_top();
                    let remain = container_clone.scroll_height() - scroll_top;
                    // show the button when we're more than 200px away from bottom
                    set_show_clone.set(remain > 200);
                }) as Box<dyn FnMut(_)>);
                let _ = container.add_event_listener_with_callback("scroll", scroll_closure.as_ref().unchecked_ref());
                // Set initial visibility
                let scroll_top_init = container.scroll_top();
                let remain_init = container.scroll_height() - scroll_top_init;
                set_show.set(remain_init > 200);
                scroll_closure.forget();
            }
        });

        // Also update visibility when messages change (e.g. new messages appended)
        let messages_for_visibility = messages.clone();
        let messages_container_ref_for_visibility = messages_container_ref.clone();
        create_effect(move |_| {
            // small debounce: run a microtask after render
            if let Some(container) = messages_container_ref_for_visibility.get() {
                let scroll_top = container.scroll_top();
                let remain = container.scroll_height() - scroll_top;
                set_show.set(remain > 200);
            }
            // depend on messages so effect runs when messages change
            messages_for_visibility.get();
        });
    }

    {
        let anchor_scroll_locked_local = anchor_scroll_locked.clone();
        let suppress_scroll_events = suppress_scroll_events.clone();
        // One-time scroll listener that clears the lock when a user scroll happens while not suppressed
        create_effect(move |_| {
            if !anchor_scroll_locked_local.get() {
                return;
            }
            if let Some(container) = messages_container_ref.get() {
                let anchor_locked_clone = anchor_scroll_locked_local.clone();
                let suppress_clone = suppress_scroll_events.clone();
                let user_scroll_closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    // Only consider this a user scroll if we are not suppressing programmatic scroll events
                    if !suppress_clone.get() {
                        anchor_locked_clone.set(false);
                    }
                }) as Box<dyn FnMut(_)>);
                let _ = container.add_event_listener_with_callback("scroll", user_scroll_closure.as_ref().unchecked_ref());
                // We forget the closure intentionally; it will be cleared when the element is removed
                user_scroll_closure.forget();
            }
        });
        // Unlock when anchor is removed (all unread cleared)
        let anchor_locked_clone2 = anchor_scroll_locked.clone();
        let first_unread_anchor_clone = first_unread_anchor.clone();
        create_effect(move |_| {
            if first_unread_anchor_clone.get().is_none() {
                anchor_locked_clone2.set(false);
            }
        });
    }
    
    // Preserve scroll position when loading older messages
    {
        let messages_container_ref = messages_container_ref.clone();
        let loading_more = loading_more.clone();
        let prev_scroll_top = prev_scroll_top.clone();
        let prev_scroll_height = prev_scroll_height.clone();
        let set_prev_scroll_top = set_prev_scroll_top.clone();
        let set_prev_scroll_height = set_prev_scroll_height.clone();
    // clone local guard to clear it when loading finishes (use setter to call .set())
    let is_loading_local_set_cl = set_is_loading_local.clone();
        create_effect(move |_| {
            // When loading_more becomes false, restore scrollTop relative to the change in scrollHeight
            if loading_more.get() {
                return;
            }
            // Only act if we had previously stored values
            let prev_top = prev_scroll_top.get();
            let prev_height = prev_scroll_height.get();
            if prev_top == 0 && prev_height == 0 {
                return;
            }
            if let Some(container) = messages_container_ref.get() {
                let new_height = container.scroll_height();
                // compute delta and set scrollTop to keep viewport stable
                let delta = new_height - prev_height;
                let new_top = prev_top + delta;
                let clamped_new_top = if new_top < 0 { 0 } else if new_top > new_height { new_height } else { new_top };
                // Suppress scroll events triggered by this programmatic change
                set_suppress_scroll_events.set(true);
                container.set_scroll_top(clamped_new_top);
                let suppress_for_timeout = set_suppress_scroll_events.clone();
                set_timeout(move || {
                    suppress_for_timeout.set(false);
                }, std::time::Duration::from_millis(400));
                // reset stored values
                set_prev_scroll_top.set(0);
                set_prev_scroll_height.set(0);
                // Clear the local guard now that loading is complete
                is_loading_local_set_cl.set(false);
            }
        });
    }

    // Clear the anchored first-unread id when the unread set for this group becomes empty
    {
        let set_first_unread_anchor = set_first_unread_anchor.clone();
        let unread_message_ids = unread_message_ids.clone();
        create_effect(move |_| {
            let map = unread_message_ids.get();
            let empty = map.get(&group_id_for_update).map(|v| v.is_empty()).unwrap_or(true);
            if empty {
                set_first_unread_anchor.set(None);
            }
        });
    }

    // Effect to close the dropdown when clicking outside
    create_effect(move |_| {
        let current_state = dropdown_state.get();
        if matches!(current_state, DropdownState::Open | DropdownState::Opening) {
            let handle_click_outside = move |event: web_sys::Event| {
                if let Some(dropdown_element) = dropdown_ref.get_untracked() {
                    if let Some(target) = event.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                            if !dropdown_element.contains(Some(&element)) {
                                set_dropdown_state.set(DropdownState::Closing);
                                // After the closing animation, set the state to Closed
                                set_timeout(
                                    move || set_dropdown_state.set(DropdownState::Closed),
                                    std::time::Duration::from_millis(150)
                                );
                            }
                        }
                    }
                }
            };
            
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(handle_click_outside) as Box<dyn FnMut(_)>);
            let _ = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });
    
    // Clone values needed for closures
    let group_name = group_data.group_name();
    let group_data_clone = group_data.clone();
    
    // Handle header actions
    let handle_header_action = move |action: ChatHeaderAction| {
        set_dropdown_state.set(DropdownState::Closing);
        set_timeout(
            move || set_dropdown_state.set(DropdownState::Closed),
            std::time::Duration::from_millis(150)
        );
        match action {
            ChatHeaderAction::InviteMembers => {
                set_invite_modal_open.set(true);
                
            }
            ChatHeaderAction::GroupDetails => {
                set_group_details_modal_open.set(true);
                
            }
            ChatHeaderAction::LeaveGroup => {
                set_leave_modal_open.set(true);
                set_leave_error.set(None);
                
            }
        }
    };

    // Leave group logic
    use std::rc::Rc;
    let handle_leave_group = {
        let set_leave_modal_open = set_leave_modal_open.clone();
        let set_leave_error = set_leave_error.clone();
        let set_leave_loading = set_leave_loading.clone();
        let group_data = group_data.clone();
        let navigate = use_navigate();
        let toast = use_toast();
        let groups_ctx = use_groups_context();
        Rc::new(move || {
            set_leave_loading.set(true);
            set_leave_error.set(None);
            let navigate = navigate.clone();
            let toast = toast.clone();
            let refresh_groups = groups_ctx.groups_hook.refresh_groups.clone();
            leptos::spawn_local(async move {
                use crate::utils::storage::StorageService;
                use crate::api::client::ApiClient;
                use crate::config::constants::AppConstants;
                let storage_service = StorageService::new();
                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                if let Some(token_response) = storage_service.get_token() {
                    http_client.set_auth_token(Some(token_response.token));
                }
                let service = GroupMembershipService::new(http_client, storage_service);
                match service.leave_group(group_data.membership.id).await {
                    Ok(_left) => {
                        set_leave_loading.set(false);
                        set_leave_modal_open.set(false);
                        // Refetch sidebar groups
                        refresh_groups.dispatch(());
                        // Show toast
                        toast.success("Hai abbandonato il gruppo con successo!");
                        // Redirect to home
                        navigate("/", Default::default());
                        // left group
                    }
                    Err(e) => {
                        set_leave_loading.set(false);
                        set_leave_error.set(Some(format!("Errore: {}", e)));
                    }
                }
            });
        })
    };

    // Handle dropdown toggle
    let handle_dropdown_toggle = move |_| {
        match dropdown_state.get_untracked() {
            DropdownState::Closed => {
                set_dropdown_state.set(DropdownState::Opening);
                set_timeout(
                    move || set_dropdown_state.set(DropdownState::Open),
                    std::time::Duration::from_millis(200)
                );
            }
            DropdownState::Open | DropdownState::Opening => {
                set_dropdown_state.set(DropdownState::Closing);
                set_timeout(
                    move || set_dropdown_state.set(DropdownState::Closed),
                    std::time::Duration::from_millis(150)
                );
            }
            DropdownState::Closing => {
                // If already closing, do nothing
            }
        }
    };

    // Handle invite member
    let handle_invite_member = move |invite_request: InviteMemberRequest| {
        
    // TODO: Implement actual invitation logic
        set_invite_modal_open.set(false);
    };

    // Handle modal close
    let handle_invite_modal_close = move |_| {
        set_invite_modal_open.set(false);
    };

    let handle_group_details_modal_close = move |_| {
        set_group_details_modal_open.set(false);
    };
    
    // Reactive signal for the background based on the theme
    let (bg_url, set_bg_url) = create_signal(String::new());
    // Function to update the background
    let update_bg = {
        let set_bg_url = set_bg_url.clone();
        move || {
            let is_dark = leptos::window().document().unwrap().document_element().unwrap().class_list().contains("dark");
            if is_dark {
                set_bg_url.set("background-image: url('/public/images/bg-chat-dark.png'); background-size: cover; background-position: center; background-repeat: no-repeat;".to_string());
            } else {
                set_bg_url.set("background-image: url('/public/images/bg-chat-light.png'); background-size: cover; background-position: center; background-repeat: no-repeat;".to_string());
            }
        }
    };
    // Update immediately
    update_bg();
    // Reactive polling to update the background when the theme changes
    {
        use gloo_timers::callback::Interval;
        let update_bg_cb = update_bg.clone();
        create_effect(move |_| {
            // Update immediately
            update_bg_cb();
            // Poll every 300ms
            let interval = Interval::new(300, move || {
                update_bg_cb();
            });
            // Cleanup: stop polling when the effect is dropped
            on_cleanup(move || {
                drop(interval);
            });
        });
    

    // Prepare a simple user service used to fetch missing profiles on-demand
    let storage_for_fetch = StorageService::new();
    let http_client_for_fetch = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
    if let Some(token_response) = storage_for_fetch.get_token() {
        http_client_for_fetch.set_auth_token(Some(token_response.token));
    }
    let user_service = UserService::new(http_client_for_fetch, storage_for_fetch);

    view! {
    <div class="flex flex-col h-full bg-white dark:bg-surface-dark">
        
        <div class="px-6 py-4 border-b border-gray-200 dark:border-border-dark bg-gray-50 dark:bg-surface-dark flex justify-between items-center">
            <div class="flex-1">
                <h2 class="text-xl font-semibold text-gray-800 dark:text-text-primary-dark mb-1">
                    {group_name.clone()}
                </h2>
                <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-text-secondary-dark">
                    <span>
                        {move || match group_data_clone.group_details.as_ref() {
                            Some(group) => match group.member_count {
                                Some(count) => format!("{} membri", count),
                                None => "Membri: N/A".to_string(),
                            },
                            None => "Caricamento...".to_string(),
                        }}
                    </span>
                    <span>"•"</span>
                    <span>{move || format!("{} online", online_count.get())}</span>
                </div>
            </div>
            <div class="flex items-center gap-2 relative" node_ref=dropdown_ref>
                <button 
                    class="bg-white dark:bg-surface-dark border border-gray-300 dark:border-gray-400 text-gray-700 dark:text-text-primary-dark px-3 py-1.5 rounded text-xs flex items-center gap-1 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors shadow-sm dark:shadow-gray-800/20"
                    on:click=handle_dropdown_toggle
                >
                    <span>"Opzioni gruppo"</span>
                    <span>"⋮"</span>
                </button>
                <div
                    class=move || {
                        let base_classes = "absolute top-full right-0 mt-1 bg-white dark:bg-surface-dark border border-gray-200 dark:border-border-dark rounded shadow-lg dark:shadow-black/50 z-50 min-w-48 transition-opacity duration-200";
                        match dropdown_state.get() {
                            DropdownState::Closed => format!("{} opacity-0 pointer-events-none", base_classes),
                            DropdownState::Opening => format!("{} opacity-100 animate-dropdown-open pointer-events-auto", base_classes),
                            DropdownState::Open => format!("{} opacity-100 pointer-events-auto", base_classes),
                            DropdownState::Closing => format!("{} opacity-0 animate-dropdown-close pointer-events-none", base_classes),
                        }
                    }
                    style="will-change: opacity, transform;"
                >
                    <div 
                        class="px-4 py-2 text-sm text-gray-700 dark:text-text-primary-dark hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer flex items-center gap-2 border-b border-gray-100 dark:border-border-dark"
                        on:click=move |_| handle_header_action(ChatHeaderAction::InviteMembers)
                    >
                        <LucideIcon name="user-plus" size=16 />
                        <span>"Invita membri"</span>
                    </div>
                    <div 
                        class="px-4 py-2 text-sm text-gray-700 dark:text-text-primary-dark hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer flex items-center gap-2 border-b border-gray-100 dark:border-border-dark"
                        on:click=move |_| handle_header_action(ChatHeaderAction::GroupDetails)
                    >
                        <LucideIcon name="settings" size=16 />
                        <span>"Dettagli gruppo"</span>
                    </div>
                    <div 
                        class="px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 cursor-pointer flex items-center gap-2"
                        on:click=move |_| handle_header_action(ChatHeaderAction::LeaveGroup)
                    >
                        <LucideIcon name="door-open" size=16 class="text-red-600" />
                        <span>"Abbandona gruppo"</span>
                    </div>
                </div>
            </div>
        </div>
    // Content area - base chat structure
    <div class="flex flex-col h-full min-h-0 relative">
            
            <div
                class="messages-container custom-scrollbar flex-1 min-h-0 overflow-y-auto px-12 py-4 space-y-4 relative"
                node_ref=messages_container_ref
                style=move || format!("{};padding-bottom:24px;", bg_url.get())
            >
                {move || {
                    use chrono::{Weekday, Datelike};
                    let msgs = messages.get();
                    let users = user_cache.get();
                    // Collect any sender_ids missing in the local user cache so we can fetch them
                    let mut missing_sender_ids: Vec<i32> = Vec::new();
                    let storage_service = StorageService::new();
                    let user_profile = storage_service.get_user_profile();

                    // Build views with day separators
                    let mut children = Vec::new();
                    let mut prev_date: Option<chrono::NaiveDate> = None;
                    let today = chrono::Utc::now().date_naive();

                    // Determine the set of unread ids for this group (initially returned by server)
                    let unread_for_group: std::collections::HashSet<i32> = unread_message_ids.get().get(&group_id_for_update).cloned().map_or_else(|| std::collections::HashSet::new(), |v| v.into_iter().collect());
                    // Use the anchored first_unread id (fixed at chat open) to render the separator in a stable position
                    let anchored = first_unread_anchor.get();
                    let mut inserted_unread_separator = false;

                    for msg in msgs.iter() {
                        // Determine message date from the message's DateTime<Utc>
                        let msg_date = msg.sent_at.date_naive();

                        let need_separator = match prev_date {
                            Some(d) => d != msg_date,
                            None => true,
                        };

                        if need_separator {
                            // Compute days difference relative to today
                            let days_diff = (today - msg_date).num_days();
                            let label = if days_diff == 0 {
                                "Oggi".to_string()
                            } else if days_diff == 1 {
                                "Ieri".to_string()
                            } else if days_diff >= 2 && days_diff <= 6 {
                                // weekday name in Italian
                                match msg_date.weekday() {
                                    Weekday::Mon => "Lunedì".to_string(),
                                    Weekday::Tue => "Martedì".to_string(),
                                    Weekday::Wed => "Mercoledì".to_string(),
                                    Weekday::Thu => "Giovedì".to_string(),
                                    Weekday::Fri => "Venerdì".to_string(),
                                    Weekday::Sat => "Sabato".to_string(),
                                    Weekday::Sun => "Domenica".to_string(),
                                }
                            } else {
                                // older than 6 days: full date (DD/MM/YYYY)
                                msg_date.format("%d/%m/%Y").to_string()
                            };

                            // Push separator view with larger horizontal padding for single-word labels
                            let is_single_word = label.split_whitespace().count() == 1;
                            let padding_class = if is_single_word { "px-6" } else { "px-3" };
                            children.push(view! {
                                <div class="w-full flex justify-center">
                                    <div class=move || format!(
                                        "text-xs text-gray-500 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 rounded-full {} py-1 my-2",
                                        padding_class
                                    )>
                                        {label}
                                    </div>
                                </div>
                            });

                            prev_date = Some(msg_date);
                        }

                        // Before rendering this message, check if it's the anchored boundary where unread messages start
                        if !inserted_unread_separator {
                            if let Some(anchor_id) = anchored {
                                if msg.id == anchor_id {
                                    // If the anchored id is present in the rendered list, insert the separator here
                                    inserted_unread_separator = true;
                                    children.push(view! {
                                        <div class="w-full flex justify-center">
                                            <div class="text-sm text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 rounded-md py-1 px-3 my-2">"Messaggi non letti"</div>
                                        </div>
                                    });
                                }
                            } else {
                                // No anchor: fallback to dynamic detection as before
                                if unread_for_group.contains(&msg.id) {
                                    inserted_unread_separator = true;
                                    children.push(view! {
                                        <div class="w-full flex justify-center">
                                            <div class="text-sm text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 rounded-md py-1 px-3 my-2">"Messaggi non letti"</div>
                                        </div>
                                    });
                                }
                            }
                        }

                        // Render the message itself
                        let (is_own, sender_username, sender_name, sender_surname) = if let Some(ref user) = user_profile {
                            if msg.sender_id == user.id {
                                (true, user.username.clone(), user.first_name.clone(), user.last_name.clone())
                            } else if let Some(sender) = users.get(&msg.sender_id) {
                                (false, sender.username.clone(), sender.first_name.clone(), sender.last_name.clone())
                            } else {
                                // Mark sender id as missing so we can request its profile async
                                missing_sender_ids.push(msg.sender_id);
                                (false, "?".to_string(), "".to_string(), "".to_string())
                            }
                        } else {
                            (false, "?".to_string(), "".to_string(), "".to_string())
                        };

                        let msg_clone = msg.clone();
                        let sender_username_clone = sender_username.clone();
                        let sender_name_clone = sender_name.clone();
                        let sender_surname_clone = sender_surname.clone();

                        children.push(view! {
                            <div id={format!("msg-{}", msg_clone.id)}>
                                <ChatMessage
                                    message=msg_clone
                                    sender_username=sender_username_clone
                                    sender_name=sender_name_clone
                                    sender_surname=sender_surname_clone
                                    status=MessageStatus::Delivered
                                    is_own=is_own
                                />
                            </div>
                        });
                    }
                    // If we found any missing sender ids, trigger background fetch to populate the cache
                    if !missing_sender_ids.is_empty() {
                        fetch_missing_users(missing_sender_ids, user_cache.clone(), user_service.clone());
                    }

                    children.into_iter().collect_view()
                }}
                <Show when=move || messages.get().is_empty()>
                    <div class="text-center text-gray-500 dark:text-gray-200 text-sm italic py-2 bg-gray-50 dark:bg-gray-800 rounded-md border border-gray-200 dark:border-gray-700 mx-auto max-w-[80%] shadow-sm">
                        Nessun messaggio ancora. Inizia la conversazione!
                    </div>
                </Show>

            </div>
            
            

            <div class="shrink-0 bg-inherit z-10 relative">
                {move || {
                    use leptos::use_context;
                    let ws_ctx = use_context::<Option<crate::hooks::use_group_message_ws::UseGroupMessageWs>>();
                    view! {
                        <div class="relative w-full">
                            <MessageInputArea
                                ws_ctx=ws_ctx.flatten()
                                group_id=group_data.membership.group_chat_id
                                on_message_sent=add_message.clone()
                            />
                        </div>

                        // Message input area (no duplicate scroll button here)
                    }
                }}
            </div>
        </div>
        
        <InviteMemberModal
            is_open=invite_modal_open
            on_close=handle_invite_modal_close
            group_chat_id=group_data.membership.group_chat_id
            group_name=group_name.clone()
        />
        
        <GroupDetailsModal
            is_open=group_details_modal_open
            on_close=handle_group_details_modal_close
            group_id=group_data_clone.membership.group_chat_id
        />
                
        <Show when=move || leave_modal_open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-40">
                <div class="bg-white dark:bg-gray-900 rounded-lg shadow-lg p-6 w-full max-w-md">
                    <h3 class="text-lg font-semibold mb-2 text-gray-900 dark:text-white">Sei sicuro di voler abbandonare il gruppo?</h3>
                    <p class="mb-4 text-gray-700 dark:text-gray-300">Questa azione è irreversibile.</p>
                    <Show when=move || leave_error.get().is_some()>
                        <div class="mb-2 text-red-600 dark:text-red-400 text-sm">{move || leave_error.get().unwrap_or_default()}</div>
                    </Show>
                    <div class="flex justify-end gap-2 mt-4">
                        <button class="px-4 py-2 rounded bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600" on:click=move |_| set_leave_modal_open.set(false) disabled=move || leave_loading.get()>
                            Annulla
                        </button>
                        <button class="px-4 py-2 rounded bg-red-600 text-white hover:bg-red-700 disabled:opacity-60" on:click={{
                            let handle_leave_group = handle_leave_group.clone();
                            move |_| (handle_leave_group)()
                        }} disabled=move || leave_loading.get()>
                            {move || if leave_loading.get() { "Abbandono..." } else { "Abbandona" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    </div>
    }
    }
}
