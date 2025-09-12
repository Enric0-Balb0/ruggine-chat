use leptos::*;
use leptos::html::Div;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use gloo_timers;
use crate::hooks::GroupMembershipWithDetails;
use crate::api::services::GroupMembershipService;
use crate::components::use_toast;
use leptos_router::use_navigate;
use crate::hooks::use_groups_context;
use crate::components::{InviteMemberModal, InviteMemberRequest, MessageInputArea, GroupDetailsModal, LucideIcon};
use crate::hooks::use_group_socket_messages::use_group_socket_messages;
use crate::hooks::use_group_message_ws::use_group_message_ws;
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
    use leptos::use_context;
    use crate::hooks::use_group_message_ws::UseGroupMessageWs;
    let ws_ctx = use_context::<Option<UseGroupMessageWs>>();
    let (local_messages, set_local_messages) = create_signal(Vec::<Message>::new());
    let set_local_messages_rc = Rc::new(set_local_messages);
    let unread_counts = use_unread_counts_context();
    use crate::context::unread_counts_context::use_unread_message_ids_context;
    use crate::context::unread_counts_context::use_unread_marked_read_context;
    let unread_message_ids = use_unread_message_ids_context();
    let unread_marked_read = use_unread_marked_read_context();
    let group_id_for_update = group_data.membership.group_chat_id;

    let (initial_messages, initial_loading, _initial_error, load_more, loading_more, has_more) = use_group_initial_messages(group_data.membership.group_chat_id, 50);
    let user_cache = use_group_user_cache(group_data.membership.group_chat_id);

    // Create or reuse a single WebSocket hook for both chat messages and presence tracking.
    // Prefer a hook provided via context (app-level) to ensure we subscribe to the same
    // message buffer that is already receiving messages; fallback to creating a new
    // hook using the stored token if none is available in context.
    let storage = StorageService::new();
    // use_context returns Option<T>, and T here is Option<UseGroupMessageWs>, so we may
    // get Some(Some(hook)). Handle both layers safely.
    let unified_ws_hook: Option<UseGroupMessageWs> = match ws_ctx.clone() {
        Some(Some(ctx_hook)) => {
            leptos::logging::log!("[PRESENCE] Using UseGroupMessageWs from context (shared)");
            Some(ctx_hook)
        }
        _ => {
            if let Some(token_response) = storage.get_token() {
                leptos::logging::log!("[PRESENCE] No context WS hook found, creating local UseGroupMessageWs");
                Some(use_group_message_ws(token_response.token))
            } else {
                leptos::logging::log!("[PRESENCE] No WS token available; presence tracking disabled");
                None
            }
        }
    };

    // Use the unified hook for chat messages
    let ws_messages = use_group_socket_messages(
        group_data.membership.group_chat_id,
        unified_ws_hook.clone(),
        local_messages,
    );
    use std::collections::HashSet;
    use crate::types::message_ws::{WebSocketMessage, ServerEvent, GroupEvent};
    let (ws_online_user_ids, set_ws_online_user_ids) = create_signal(HashSet::<i32>::new());
    let (initial_online_user_ids, set_initial_online_user_ids) = create_signal(HashSet::<i32>::new());

    // Member count signal (authoritative refresh will be triggered by WS events)
    let (current_member_count, set_current_member_count) = create_signal(
        group_data.group_details
            .as_ref()
            .and_then(|g| g.member_count)
            .unwrap_or(1)
    );

    // Signal to track when current user has left the group
    let (current_user_left_group, set_current_user_left_group) = create_signal(false);
    
    // Signal to track if leave was initiated locally (to avoid double navigation)
    let (leave_initiated_locally, set_leave_initiated_locally) = create_signal(false);

    // Navigation hooks
    let navigate = use_navigate();
    let groups_ctx = use_groups_context();

    // Effect to handle navigation when current user leaves group (only for remote leave events)
    {
        let navigate = navigate.clone();
        let groups_ctx = groups_ctx.clone();
        create_effect(move |_| {
            if current_user_left_group.get() && !leave_initiated_locally.get() {
                log::info!("Current user left group {} remotely, redirecting to home", group_data.membership.group_chat_id);
                // Refresh groups list and navigate to home
                groups_ctx.groups_hook.refresh_groups.dispatch(());
                navigate("/", Default::default());
            }
        });
    }

    // Process WebSocket events for presence tracking in a reactive way (no polling loop)
    {
        let ws_hook = unified_ws_hook.clone();
        let set_initial = set_initial_online_user_ids.clone();
        let set_current_member_count = set_current_member_count.clone();
        let set_user_left = set_current_user_left_group.clone();
        let current_group_id = group_data.membership.group_chat_id;
        create_effect(move |prev_len: Option<usize>| {
            if let Some(hook) = ws_hook.as_ref() {
                let msgs = hook.messages.get();
                let current_len = msgs.len();
                if let Some(prev) = prev_len {
                    if current_len > prev {
                        // process only new messages
                        for msg in msgs.iter().skip(prev) {
                            if let WebSocketMessage::Event { event, .. } = msg {
                                if let ServerEvent::Groups(group_event) = event {
                                    match group_event {
                                        GroupEvent::Joined { .. } | GroupEvent::Left { .. } | 
                                        GroupEvent::NewGroupMembership { .. } | GroupEvent::LeftGroupMembership { .. } => {
                                            // For presence changes, refresh authoritative lists once
                                            let set_initial_inner = set_initial.clone();
                                            let set_members_inner = set_current_member_count.clone();
                                            let gid = group_id_for_update;
                                            
                                            // Check if this is the current user leaving
                                            let is_current_user_leaving = if let GroupEvent::LeftGroupMembership { left_membership_username, .. } = group_event {
                                                let storage = StorageService::new();
                                                if let Some(user_profile) = storage.get_user_profile() {
                                                    let current_username = user_profile.email.split('@').next().unwrap_or("");
                                                    current_username == left_membership_username || user_profile.email == *left_membership_username
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            };
                                            
                                            // If current user left, trigger navigation
                                            if is_current_user_leaving {
                                                log::info!("Current user left group {}, triggering navigation", current_group_id);
                                                set_user_left.set(true);
                                                return current_len;
                                            }
                                            
                                            spawn_local(async move {
                                                use crate::api::client::ApiClient;
                                                use crate::api::services::membership::GroupMembershipService;
                                                use crate::config::constants::AppConstants;
                                                use crate::utils::storage::StorageService;

                                                let storage = StorageService::new();
                                                let http = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                                                if let Some(token_response) = storage.get_token() {
                                                    http.set_auth_token(Some(token_response.token));
                                                }
                                                let membership_service = GroupMembershipService::new(http, storage);

                                                // Refresh online users for this group
                                                match membership_service.find_online_users_in_group(gid).await {
                                                    Ok(ids) => {
                                                        let new_set: std::collections::HashSet<i32> = ids.into_iter().collect();
                                                        set_initial_inner.set(new_set);
                                                    }
                                                    Err(e) => {
                                                        if e.to_string().contains("404") || e.to_string().contains("Group membership not found") {
                                                            log::info!("No longer member of group {}, stopping online user updates", gid);
                                                            set_initial_inner.set(std::collections::HashSet::new());
                                                        } else {
                                                            log::error!("Error refreshing online users for group {}: {:?}", gid, e);
                                                        }
                                                    }
                                                }

                                                // Refresh member count for this group
                                                match membership_service.get_by_group_chat_id(&gid.to_string()).await {
                                                    Ok(group_members) => {
                                                        let new_count = group_members.len() as i32;
                                                        set_members_inner.set(new_count);
                                                    }
                                                    Err(e) => {
                                                        if e.to_string().contains("404") || e.to_string().contains("Group membership not found") {
                                                            log::info!("No longer member of group {}, stopping member count updates", gid);
                                                            set_members_inner.set(0);
                                                        } else {
                                                            log::error!("Error refreshing group members for group {}: {:?}", gid, e);
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                current_len
            } else {
                prev_len.unwrap_or(0)
            }
        });
    }

    {
        let set_initial = set_initial_online_user_ids.clone();
        leptos::spawn_local(async move {
            use crate::utils::storage::StorageService;
            use crate::utils::error_recovery::NetworkOperation;
            
            let storage = StorageService::new();
            let http = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
            if let Some(token_response) = storage.get_token() {
                http.set_auth_token(Some(token_response.token));
            }
            let membership_service = GroupMembershipService::new(http, storage);
            
            if let Some(ids) = membership_service.find_online_users_in_group(group_id_for_update)
                .with_auto_retry("load online users").await {
                let set: HashSet<i32> = ids.into_iter().collect();
                set_initial.set(set);
            }
        });
    }

    let online_count = create_memo(move |_| {
        let mut union_set = HashSet::<i32>::new();
        let initial_ids = initial_online_user_ids.get();
        let ws_ids = ws_online_user_ids.get();
        
        for id in initial_ids.iter() { union_set.insert(*id); }
        for id in ws_ids.iter() { union_set.insert(*id); }
        
        let mut count = union_set.len();
        let storage = StorageService::new();
        if let Some(user) = storage.get_user_profile() {
            if !union_set.contains(&user.id) {
                count += 1;
            }
        }
        
        count
    });

    
    
    let member_count = create_memo(move |_| current_member_count.get());

    let unread_counts = use_unread_counts_context();

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
    let (scrolled_initial, set_scrolled_initial) = create_signal(false);
    let (first_unread_anchor, set_first_unread_anchor) = create_signal(None::<i32>);
    let (is_loading_local, set_is_loading_local) = create_signal(false);
    let (suppress_scroll_events, set_suppress_scroll_events) = create_signal(false);
    let (show_scroll_to_bottom, set_show_scroll_to_bottom) = create_signal(false);
    let (user_scrolled_once, set_user_scrolled_once) = create_signal(false);
    let pending_update_ids: std::rc::Rc<std::cell::RefCell<Vec<i32>>> = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
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
            let last_id = locals.last().unwrap().id;
            let anchor_scroll_locked_for_send_inner = anchor_scroll_locked_for_send.clone();
            // Capture the current container (if any) now so the delayed closure does not
            // access the reactive NodeRef signal after the component may have been disposed.
            let captured_container_for_scroll = messages_container_ref_for_scroll.get();
            set_timeout(move || {
                let doc = match web_sys::window() {
                    Some(w) => match w.document() { Some(d) => d, None => return },
                    None => return,
                };
                if let Some(elem) = doc.get_element_by_id(&format!("msg-{}", last_id)) {
                    if !anchor_scroll_locked_for_send_inner.get() && !user_scrolled_once.get() {
                        let _ = elem.scroll_into_view_with_bool(true);
                    }
                } else if let Some(container) = captured_container_for_scroll.clone() {
                    if !anchor_scroll_locked_for_send_inner.get() && !user_scrolled_once.get() {
                        container.set_scroll_top(container.scroll_height());
                    }
                }
            }, std::time::Duration::from_millis(50));
        });
    }
    
    // Create add_message callback with auto-scroll functionality
    let add_message: Rc<dyn Fn(Message)> = {
        let set_local_messages_rc = Rc::clone(&set_local_messages_rc);
        let unread_counts = unread_counts.clone();
        let group_id = group_id_for_update;
        let messages_container_ref_for_send = messages_container_ref.clone();
        Rc::new(move |msg: Message| {
            set_local_messages_rc.update(|msgs| msgs.push(msg.clone()));
            
            // Auto-scroll to bottom after sending a message
            let messages_container_ref_scroll = messages_container_ref_for_send.clone();
            // Capture the container now to avoid accessing the NodeRef inside the async task
            let captured_container_for_send = messages_container_ref_for_send.get();
            leptos::spawn_local(async move {
                // Small delay to ensure DOM is updated
                crate::utils::timers::sleep_ms(10).await;
                if let Some(container) = captured_container_for_send.clone() {
                    container.set_scroll_top(container.scroll_height());
                    leptos::logging::log!("[SEND MESSAGE DEBUG] Auto-scrolled to bottom after sending message");
                }
            });
            
            let unread_counts = unread_counts.clone();
            let msg_id = msg.id;
            leptos::spawn_local(async move {
                use crate::api::services::message::MessageService;
                use crate::config::constants::AppConstants;
                use crate::utils::storage::StorageService;
                use crate::api::client::ApiClient;
                use crate::utils::error_recovery::NetworkOperation;
                
                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                let storage_service = StorageService::new();
                if let Some(token) = storage_service.get_token() {
                    http_client.set_auth_token(Some(token.token));
                }
                let message_service = MessageService::new(http_client, storage_service);
                
                if let Some(()) = message_service.update_message_read_at(msg_id)
                    .with_auto_retry("mark message as read").await {
                    decrement_unread_for_group(&unread_counts, group_id);
                    unread_message_ids.update(|map| {
                        if let Some(vec_ids) = map.get_mut(&group_id) {
                            vec_ids.retain(|id| *id != msg_id);
                        }
                    });
                }
            });
        })
    };
    
    let flush_scheduled: std::rc::Rc<std::cell::Cell<bool>> = std::rc::Rc::new(std::cell::Cell::new(false));
    {
        let messages = messages.clone();
        let messages_container_ref = messages_container_ref.clone();
        let initial_loading = initial_loading.clone();
        let set_scrolled_initial = set_scrolled_initial.clone();
        let load_more_init = load_more.clone();
        let anchor_scroll_locked_for_effect = anchor_scroll_locked.clone();

        create_effect(move |_| {
            if initial_loading.get() || scrolled_initial.get() || user_scrolled_once.get() {
                return;
            }
            let msgs = messages.get_untracked();
            if msgs.is_empty() {
                return;
            }

            let unread_map = unread_message_ids.get_untracked();
            let maybe_initial_unread = unread_map.get(&group_id_for_update).cloned();

            let anchor_lock_for_closure = anchor_scroll_locked_for_effect.clone();
            let anchor_for_scroll = anchor_scroll_locked_for_effect.clone();
            let scroll_elem_into_view = move |container: Option<leptos::HtmlElement<Div>>, elem_id: String| {
                let doc = match web_sys::window() {
                    Some(w) => match w.document() { Some(d) => d, None => return false },
                    None => return false,
                };
                
                // Try to find the unread separator first, fallback to message element
                let separator_id = format!("unread-separator-{}", elem_id.trim_start_matches("msg-"));
                let target_element = doc.get_element_by_id(&separator_id)
                    .or_else(|| doc.get_element_by_id(&elem_id));
                
                if let Some(elem) = target_element {
                    if let Some(c) = container {
                        let desired = compute_bottom_aligned_scroll(&c, &elem);
                        c.set_scroll_top(desired);
                        true
                    } else {
                        let _ = elem.scroll_into_view_with_bool(true);
                        true
                    }
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
                                        if !anchor_lock_for_closure.get() && !user_scrolled_once.get() {
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
                                                if !anchor_scroll_locked_for_retry.get() && !user_scrolled_once.get() {
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
                                                if !anchor_scroll_locked_for_retry2.get() && !user_scrolled_once.get() {
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
                                        if !anchor_lock_for_closure.get() && !user_scrolled_once.get() {
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

            let last_msg = msgs.last().unwrap();
            if let Some(_container) = messages_container_ref.get() {
                let doc = web_sys::window().unwrap().document().unwrap();
                if !anchor_lock_for_closure.get() && !user_scrolled_once.get() {
                    if let Some(last_elem) = doc.get_element_by_id(&format!("msg-{}", last_msg.id)) {
                        let _ = last_elem.scroll_into_view_with_bool(true);
                        set_scrolled_initial.set(true);
                    }
                }
            }
        });
    }
    {
        let messages = messages.clone();
        let messages_container_ref = messages_container_ref.clone();
        
    let is_loading_local_read = is_loading_local.clone();
    let is_loading_local_set = set_is_loading_local.clone();
    let suppress_scroll_events_read = suppress_scroll_events.clone();
    let suppress_scroll_events_set = set_suppress_scroll_events.clone();
        create_effect(move |_| {
            if let Some(container) = messages_container_ref.get() {
                let container_clone = container.clone();
                let messages = messages.clone();
                let load_more = load_more.clone();
                let has_more_cl = has_more.clone();
                let set_prev_scroll_top = set_prev_scroll_top.clone();
                let set_prev_scroll_height = set_prev_scroll_height.clone();

                const LOAD_MORE_THRESHOLD: i32 = 150;

                let loading_more = loading_more.clone();
                let is_loading_local_cl = is_loading_local_read.clone();
                let suppress_scroll_events_cl = suppress_scroll_events_read.clone();

                let pending_for_closure = pending_update_ids.clone();
                let flush_for_closure = flush_scheduled.clone();
                let unread_counts_for_closure = unread_counts.clone();
                let unread_message_ids_for_closure = unread_message_ids.clone();

                let user_scrolled_once_read = user_scrolled_once.clone();
                let set_user_scrolled_once_cl = set_user_scrolled_once.clone();
                let set_scrolled_initial_cl2 = set_scrolled_initial.clone();
                let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    if !suppress_scroll_events_cl.get_untracked() && !user_scrolled_once_read.get_untracked() {
                        set_user_scrolled_once_cl.set(true);
                        set_scrolled_initial_cl2.set(true);
                    }
                    
                    // Ottenere l'ID utente corrente per escludere i propri messaggi
                    let current_user_id = {
                        use crate::utils::storage::StorageService;
                        let storage = StorageService::new();
                        storage.get_user_profile().map(|profile| profile.id)
                    };
                    
                    let doc = web_sys::window().unwrap().document().unwrap();
                    let current_messages = messages.get_untracked();
                    let msg_ids: Vec<i32> = current_messages.iter().map(|m| m.id).collect();

                    for msg_id in msg_ids {
                        // Trovare il messaggio per controllare il sender_id
                        let msg_opt = current_messages.iter().find(|m| m.id == msg_id);
                        if let Some(msg) = msg_opt {
                            // Saltare i messaggi inviati dall'utente corrente
                            if let Some(user_id) = current_user_id {
                                if msg.sender_id == user_id {
                                    continue;
                                }
                            }
                        }
                        
                        if let Some(elem) = doc.get_element_by_id(&format!("msg-{}", msg_id)) {
                            if is_element_in_viewport(&container_clone, &elem) {
                                let mut should_update = false;
                                let initial_ids_map = unread_message_ids.get_untracked();
                                if let Some(initial_ids) = initial_ids_map.get(&group_id_for_update) {
                                    if initial_ids.contains(&msg_id) {
                                        should_update = true;
                                    }
                                }

                                if !should_update {
                                    let ws_ids: Vec<i32> = ws_messages.get_untracked().iter().map(|m| m.id).collect();
                                    if ws_ids.contains(&msg_id) {
                                        let in_initial = initial_ids_map.get(&group_id_for_update)
                                            .map(|v| v.contains(&msg_id)).unwrap_or(false);
                                        if !in_initial {
                                            should_update = true;
                                        }
                                    }
                                }

                                if should_update {
                                    {
                                        let mut buf = pending_for_closure.borrow_mut();
                                        if !buf.contains(&msg_id) {
                                            buf.push(msg_id);
                                        }
                                    }

                                    if !flush_for_closure.get() {
                                        let pending_clone = pending_for_closure.clone();
                                        let flush_flag = flush_for_closure.clone();
                                        let unread_counts_clone = unread_counts_for_closure.clone();
                                        let unread_message_ids_clone = unread_message_ids_for_closure.clone();
                                        flush_flag.set(true);

                                        set_timeout(move || {
                                            leptos::spawn_local(async move {
                                                loop {
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
                                                    process_update_batch(batch, unread_counts_clone.clone(), unread_message_ids_clone.clone(), unread_marked_read.clone(), group_id_for_update).await;
                                                    crate::utils::timers::sleep_ms(10).await;
                                                }
                                                
                                                // Sincronizza il counter con il server dopo tutti gli aggiornamenti
                                                leptos::spawn_local({
                                                    let unread_counts_sync = unread_counts_clone.clone();
                                                    async move {
                                                        use crate::api::client::ApiClient;
                                                        use crate::config::constants::AppConstants;
                                                        use crate::utils::storage::StorageService;
                                                        use crate::api::services::message::MessageService;
                                                        
                                                        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                                                        let storage_service = StorageService::new();
                                                        if let Some(token) = storage_service.get_token() {
                                                            http_client.set_auth_token(Some(token.token));
                                                        }
                                                        let message_service = MessageService::new(http_client, storage_service);
                                                        
                                                        match message_service.get_messages_not_read_yet(group_id_for_update).await {
                                                            Ok(message_page) => {
                                                                let actual_unread_count = message_page.data.len() as u32;
                                                                // Aggiorna il counter locale con il valore reale del server
                                                                unread_counts_sync.update(|counts| {
                                                                    counts.insert(group_id_for_update, actual_unread_count);
                                                                });
                                                            },
                                                            Err(e) => {
                                                                log::warn!("Failed to sync unread count for group {}: {:?}", group_id_for_update, e);
                                                            }
                                                        }
                                                    }
                                                });
                                                
                                                flush_flag.set(false);
                                            });
                                        }, std::time::Duration::from_millis(UPDATE_DEBOUNCE_MS));
                                    }
                                } else {
                                }
                            }
                        }
                    }

                    let scroll_top = container_clone.scroll_top();
                    let loading_now = loading_more.get_untracked();

                    if suppress_scroll_events_cl.get_untracked() {
                        return;
                    }

                    if scroll_top <= LOAD_MORE_THRESHOLD && !loading_now && !is_loading_local_cl.get_untracked() && has_more_cl.get_untracked() {
                        is_loading_local_set.set(true);
                        suppress_scroll_events_set.set(true);

                        {
                            let suppress_fallback = suppress_scroll_events_set.clone();
                            set_timeout(move || {
                                suppress_fallback.set(false);
                            }, std::time::Duration::from_millis(1500));
                        }

                        {
                            let is_loading_fallback = is_loading_local_set.clone();
                            set_timeout(move || {
                                is_loading_fallback.set(false);
                            }, std::time::Duration::from_millis(5000));
                        }

                        set_prev_scroll_top.set(scroll_top);
                        set_prev_scroll_height.set(container_clone.scroll_height());

                        load_more();
                    } else if scroll_top <= LOAD_MORE_THRESHOLD && !has_more_cl.get_untracked() {
                        set_prev_scroll_top.set(0);
                        set_prev_scroll_height.set(0);
                    }
                }) as Box<dyn FnMut(_)>);

                let _ = container.add_event_listener_with_callback("scroll", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        });
    }

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
                        let remain = container_clone.scroll_height() - (scroll_top + container_clone.client_height());
                        set_show_clone.set(remain > 200);
                }) as Box<dyn FnMut(_)>);
                let _ = container.add_event_listener_with_callback("scroll", scroll_closure.as_ref().unchecked_ref());
                let scroll_top_init = container.scroll_top();
                let remain_init = container.scroll_height() - (scroll_top_init + container.client_height());
                set_show.set(remain_init > 200);
                scroll_closure.forget();
            }
        });

        let messages_for_visibility = messages.clone();
        let messages_container_ref_for_visibility = messages_container_ref.clone();
        create_effect(move |_| {
            if let Some(container) = messages_container_ref_for_visibility.get() {
                let scroll_top = container.scroll_top();
                let remain = container.scroll_height() - (scroll_top + container.client_height());
                set_show.set(remain > 200);
            }
            messages_for_visibility.get();
        });
    }

    // Auto mark visible messages as read when messages list changes
    {
        let messages_for_auto_read = messages.clone();
        let messages_container_ref_for_auto_read = messages_container_ref.clone();
        let pending_auto_read = std::rc::Rc::new(std::cell::RefCell::new(Vec::<i32>::new()));
        let flush_auto_read = std::rc::Rc::new(std::cell::Cell::new(false));
        let unread_counts_auto_read = unread_counts.clone();
        let unread_message_ids_auto_read = unread_message_ids.clone();
        let unread_marked_read_auto_read = unread_marked_read.clone();
        let unified_ws_hook_auto_read = unified_ws_hook.clone();
        
        create_effect(move |_| {
            let current_messages = messages_for_auto_read.get();
            
            // Only process if we have a container and messages
            if let Some(container) = messages_container_ref_for_auto_read.get() {
                if !current_messages.is_empty() {
                    
                    // Use a short timeout to ensure DOM is updated
                    let container_clone = container.clone();
                    let pending_clone = pending_auto_read.clone();
                    let flush_flag = flush_auto_read.clone();
                    let unread_counts_clone = unread_counts_auto_read.clone();
                    let unread_message_ids_clone = unread_message_ids_auto_read.clone();
                    let unread_marked_read_clone = unread_marked_read_auto_read.clone();
                    let ws_hook_clone = unified_ws_hook_auto_read.clone();
                    
                    set_timeout(move || {
                        // Ottenere l'ID utente corrente per escludere i propri messaggi
                        let current_user_id = {
                            use crate::utils::storage::StorageService;
                            let storage = StorageService::new();
                            storage.get_user_profile().map(|profile| profile.id)
                        };
                        
                        let doc = web_sys::window().unwrap().document().unwrap();
                        let mut found_visible = false;
                        
                        for msg in current_messages.iter() {
                            // Saltare i messaggi inviati dall'utente corrente
                            if let Some(user_id) = current_user_id {
                                if msg.sender_id == user_id {
                                    continue;
                                }
                            }
                            
                            if let Some(elem) = doc.get_element_by_id(&format!("msg-{}", msg.id)) {
                                if is_element_in_viewport(&container_clone, &elem) {
                                    
                                    let mut should_update = false;
                                    let initial_ids_map = unread_message_ids_clone.get_untracked();
                                    if let Some(initial_ids) = initial_ids_map.get(&group_id_for_update) {
                                        if initial_ids.contains(&msg.id) {
                                            should_update = true;
                                        }
                                    }

                                    if !should_update {
                                        // Check if it's from WebSocket messages
                                        if let Some(ws_hook) = ws_hook_clone.clone() {
                                            let ws_ids: Vec<i32> = ws_hook.messages.get_untracked()
                                                .iter()
                                                .filter_map(|ws_msg| {
                                                    if let WebSocketMessage::Event { event: ServerEvent::Groups(GroupEvent::NewMessage { message_id, .. }), .. } = ws_msg {
                                                        Some(*message_id)
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect();
                                            if ws_ids.contains(&msg.id) {
                                                let in_initial = initial_ids_map.get(&group_id_for_update)
                                                    .map(|v| v.contains(&msg.id)).unwrap_or(false);
                                                if !in_initial {
                                                    should_update = true;
                                                }
                                            }
                                        }
                                    }

                                    if should_update {
                                        {
                                            let mut buf = pending_clone.borrow_mut();
                                            if !buf.contains(&msg.id) {
                                                buf.push(msg.id);
                                                found_visible = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        // If we found visible messages, trigger the batch processing
                        if found_visible && !flush_flag.get() {
                            let pending_for_flush = pending_clone.clone();
                            let flush_flag_for_flush = flush_flag.clone();
                            
                            flush_flag_for_flush.set(true);
                            
                            set_timeout(move || {
                                leptos::spawn_local(async move {
                                    loop {
                                        let mut batch: Vec<i32> = Vec::new();
                                        {
                                            let mut guard = pending_for_flush.borrow_mut();
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
                                        process_update_batch(batch, unread_counts_clone.clone(), unread_message_ids_clone.clone(), unread_marked_read_clone.clone(), group_id_for_update).await;
                                        crate::utils::timers::sleep_ms(10).await;
                                    }
                                    
                                    flush_flag_for_flush.set(false);
                                });
                            }, std::time::Duration::from_millis(UPDATE_DEBOUNCE_MS));
                        }
                        
                    }, std::time::Duration::from_millis(100));
                }
            }
        });
    }

    {
        let first_unread_anchor = first_unread_anchor.clone();
        let messages_for_anchor = messages.clone();
        let set_show_anchor = set_show_scroll_to_bottom.clone();
        create_effect(move |_| {
            if let Some(anchor_id) = first_unread_anchor.get() {
                let msgs = messages_for_anchor.get();
                if let Some(pos) = msgs.iter().position(|m| m.id == anchor_id) {
                    if pos + 1 < msgs.len() {
                        set_show_anchor.set(true);
                    }
                }
            }
        });
    }

    {
        let anchor_scroll_locked_local = anchor_scroll_locked.clone();
        let suppress_scroll_events = suppress_scroll_events.clone();
        create_effect(move |_| {
            if !anchor_scroll_locked_local.get() {
                return;
            }
            if let Some(container) = messages_container_ref.get() {
                let anchor_locked_clone = anchor_scroll_locked_local.clone();
                let suppress_clone = suppress_scroll_events.clone();
                let user_scroll_closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    if !suppress_clone.get() {
                        anchor_locked_clone.set(false);
                    }
                }) as Box<dyn FnMut(_)>);
                let _ = container.add_event_listener_with_callback("scroll", user_scroll_closure.as_ref().unchecked_ref());
                user_scroll_closure.forget();
            }
        });
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
    let is_loading_local_set_cl = set_is_loading_local.clone();
        create_effect(move |_| {
            if loading_more.get() {
                return;
            }
            let prev_top = prev_scroll_top.get();
            let prev_height = prev_scroll_height.get();
            if prev_top == 0 && prev_height == 0 {
                return;
            }
            if let Some(container) = messages_container_ref.get() {
                let new_height = container.scroll_height();
                let delta = new_height - prev_height;
                let new_top = prev_top + delta;
                let clamped_new_top = if new_top < 0 { 0 } else if new_top > new_height { new_height } else { new_top };
                set_suppress_scroll_events.set(true);
                container.set_scroll_top(clamped_new_top);
                let suppress_for_timeout = set_suppress_scroll_events.clone();
                set_timeout(move || {
                    suppress_for_timeout.set(false);
                }, std::time::Duration::from_millis(400));
                set_prev_scroll_top.set(0);
                set_prev_scroll_height.set(0);
                is_loading_local_set_cl.set(false);
            }
        });
    }

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

    create_effect(move |_| {
        let current_state = dropdown_state.get();
        if matches!(current_state, DropdownState::Open | DropdownState::Opening) {
            let handle_click_outside = move |event: web_sys::Event| {
                if let Some(dropdown_element) = dropdown_ref.get_untracked() {
                    if let Some(target) = event.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                            if !dropdown_element.contains(Some(&element)) {
                                set_dropdown_state.set(DropdownState::Closing);
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
    
    let group_name = group_data.group_name();
    let group_data_clone = group_data.clone();
    
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

    use std::rc::Rc;
    let handle_leave_group = {
        let set_leave_modal_open = set_leave_modal_open.clone();
        let set_leave_error = set_leave_error.clone();
        let set_leave_loading = set_leave_loading.clone();
        let group_data = group_data.clone();
        let navigate = navigate.clone();
        let toast = use_toast();
        let groups_ctx = groups_ctx.clone();
        let set_local_leave = set_leave_initiated_locally.clone();
        Rc::new(move || {
            set_leave_loading.set(true);
            set_leave_error.set(None);
            set_local_leave.set(true); // Mark that leave was initiated locally
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
                        refresh_groups.dispatch(());
                        toast.success("Hai abbandonato il gruppo con successo!");
                        navigate("/", Default::default());
                    }
                    Err(e) => {
                        set_leave_loading.set(false);
                        set_leave_error.set(Some(format!("Errore: {}", e)));
                    }
                }
            });
        })
    };

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
            }
        }
    };

    let _handle_invite_member = move |_invite_request: InviteMemberRequest| {
        set_invite_modal_open.set(false);
    };

    let handle_invite_modal_close = move |_| {
        set_invite_modal_open.set(false);
    };

    let handle_group_details_modal_close = move |_| {
        set_group_details_modal_open.set(false);
    };
    
    let (bg_url, set_bg_url) = create_signal(String::new());
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
    update_bg();
    {
        use gloo_timers::callback::Interval;
        let update_bg_cb = update_bg.clone();
        create_effect(move |_| {
            update_bg_cb();
            let interval = Interval::new(300, move || {
                update_bg_cb();
            });
            on_cleanup(move || {
                drop(interval);
            });
        });
    

    let storage_for_fetch = StorageService::new();
    let http_client_for_fetch = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
    if let Some(token_response) = storage_for_fetch.get_token() {
        http_client_for_fetch.set_auth_token(Some(token_response.token));
    }
    let user_service = UserService::new(http_client_for_fetch, storage_for_fetch);

    let handle_scroll_to_bottom = {
        let messages_container_ref = messages_container_ref.clone();
        let set_show_scroll_to_bottom = set_show_scroll_to_bottom.clone();
        move |_| {
            if let Some(container) = messages_container_ref.get() {
                let start = container.scroll_top();
                let end = container.scroll_height();
                let distance = end - start;
                if distance <= 0 { return; }
                use std::rc::Rc;
                use std::cell::{Cell, RefCell};
                let start_time = Rc::new(Cell::new(0f64));
                let duration = 420f64;
                let container_clone = container.clone();
                let start_time_clone = start_time.clone();
                let raf_closure: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
                let raf_closure_clone = raf_closure.clone();
                let set_show_scroll_to_bottom_clone = set_show_scroll_to_bottom.clone();
                *raf_closure.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
                    if start_time_clone.get() == 0f64 { start_time_clone.set(timestamp); }
                    let elapsed = timestamp - start_time_clone.get();
                    let progress = (elapsed / duration).min(1.0);
                    let eased = 1.0 - (1.0 - progress).powf(3.0);
                    let new_top = start as f64 + (distance as f64 * eased);
                    container_clone.set_scroll_top(new_top as i32);
                    if progress < 1.0 {
                        if let Some(cb) = raf_closure_clone.borrow().as_ref() {
                            let _ = web_sys::window().unwrap().request_animation_frame(cb.as_ref().unchecked_ref());
                        }
                    } else {
                        container_clone.set_scroll_top(container_clone.scroll_height());
                        set_show_scroll_to_bottom_clone.set(false);
                        raf_closure_clone.borrow_mut().take();
                    }
                }) as Box<dyn FnMut(f64)>));
                {
                    if let Some(cb) = raf_closure.borrow().as_ref() {
                        let _ = web_sys::window().unwrap().request_animation_frame(cb.as_ref().unchecked_ref());
                    }
                };
            }
        }
    };

    view! {
    <div class="flex flex-col h-full bg-white dark:bg-surface-dark">
        
        <div class="px-6 py-4 border-b border-gray-200 dark:border-border-dark bg-gray-50 dark:bg-surface-dark flex justify-between items-center">
            <div class="flex-1">
                <h2 class="text-xl font-semibold text-gray-800 dark:text-text-primary-dark mb-1">
                    {group_name.clone()}
                </h2>
                <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-text-secondary-dark">
                    <span>
                        {move || format!("{} membri", member_count.get())}
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
    <div class="flex flex-col h-full min-h-0 relative">
            
            <div
                class="messages-container custom-scrollbar flex-1 min-h-0 overflow-y-auto px-12 py-4 relative"
                node_ref=messages_container_ref
                style=move || format!("{};padding-bottom:24px;", bg_url.get())
            >
                {move || {
                    use chrono::{Weekday, Datelike};
                    let msgs = messages.get();
                    let users = user_cache.get();
                    let mut missing_sender_ids: Vec<i32> = Vec::new();
                    let storage_service = StorageService::new();
                    let user_profile = storage_service.get_user_profile();
                    
                    let mut children = Vec::new();
                    let mut prev_date: Option<chrono::NaiveDate> = None;
                    let today = chrono::Utc::now().date_naive();

                    let unread_for_group: std::collections::HashSet<i32> = unread_message_ids.get().get(&group_id_for_update).cloned().map_or_else(|| std::collections::HashSet::new(), |v| v.into_iter().collect());
                    let anchored = first_unread_anchor.get();
                    let mut inserted_unread_separator = false;

                    let mut prev_sender: Option<i32> = None;
                    for msg in msgs.iter() {
                        let msg_date = msg.sent_at.date_naive();

                        let need_separator = match prev_date {
                            Some(d) => d != msg_date,
                            None => true,
                        };

                        if need_separator {
                            let days_diff = (today - msg_date).num_days();
                            let label = if days_diff == 0 {
                                "Oggi".to_string()
                            } else if days_diff == 1 {
                                "Ieri".to_string()
                            } else if days_diff >= 2 && days_diff <= 6 {
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
                                msg_date.format("%d/%m/%Y").to_string()
                            };

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
                            prev_sender = None;
                        }

                        if !inserted_unread_separator {
                            if let Some(anchor_id) = anchored {
                                if msg.id == anchor_id {
                                    inserted_unread_separator = true;
                                    children.push(view! {
                                        <div id={format!("unread-separator-{}", anchor_id)} class="w-full flex justify-center">
                                            <div class="text-sm text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 rounded-md py-1 px-3 my-2">"Messaggi non letti"</div>
                                        </div>
                                    });
                                    prev_sender = None; // break message grouping
                                }
                            } else {
                                if unread_for_group.contains(&msg.id) {
                                    inserted_unread_separator = true;
                                    children.push(view! {
                                        <div id={format!("unread-separator-{}", msg.id)} class="w-full flex justify-center">
                                            <div class="text-sm text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 rounded-md py-1 px-3 my-2">"Messaggi non letti"</div>
                                        </div>
                                    });
                                    prev_sender = None; // break message grouping
                                }
                            }
                        }

                        let (is_own, sender_username, sender_name, sender_surname) = if let Some(ref user) = user_profile {
                            if msg.sender_id == user.id {
                                (true, user.username.clone(), user.first_name.clone(), user.last_name.clone())
                            } else if let Some(sender) = users.get(&msg.sender_id) {
                                (false, sender.username.clone(), sender.first_name.clone(), sender.last_name.clone())
                            } else {
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

                        let continued = prev_sender.map(|s| s == msg.sender_id).unwrap_or(false);
                        let show_sender = !continued;
                        prev_sender = Some(msg.sender_id);
                        children.push(view! {
                            <div id={format!("msg-{}", msg_clone.id)} class=move || {
                                if continued { "msg-wrapper continued".to_string() } else { "msg-wrapper first".to_string() }
                            }>
                                <ChatMessage
                                    message=msg_clone
                                    sender_username=sender_username_clone
                                    sender_name=sender_name_clone
                                    sender_surname=sender_surname_clone
                                    _status=MessageStatus::Delivered
                                    is_own=is_own
                                    continued=continued
                                    show_sender=show_sender
                                />
                            </div>
                        });
                    }
                    if !missing_sender_ids.is_empty() {
                        fetch_missing_users(missing_sender_ids, user_cache.clone(), user_service.clone());
                    }

                    children.into_iter().collect_view()
                }}
                <Show when=move || messages.get().is_empty()>
                    <div class="text-center text-gray-500 dark:text-gray-200 text-sm italic py-2 bg-gray-50 dark:bg-gray-800 rounded-md border border-gray-200 dark:border-gray-700 mx-auto max-w-[80%] shadow-sm">
                        "Nessun messaggio ancora." {" "} "Inizia la conversazione!"
                    </div>
                </Show>

            </div>

            <Show when=move || show_scroll_to_bottom.get()>
                <button
                    on:click=handle_scroll_to_bottom
                    aria-label="Scorri in basso"
                    class="absolute bottom-24 right-6 w-10 h-10 rounded-full bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 shadow-md flex items-center justify-center hover:bg-gray-300 dark:hover:bg-gray-600 focus:outline-none focus:ring-0 transition z-30 backdrop-blur-sm/40"
                >
                    <LucideIcon name="arrow-down" size=20 />
                </button>
            </Show>
            
            

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
