use leptos::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::rc::Rc;
use std::cell::RefCell;
use crate::api::services::membership::GroupMembershipService;
use crate::types::message_ws::{ServerEvent, GroupEvent, WebSocketMessage};
use crate::hooks::use_group_message_ws::UseGroupMessageWs;
use crate::utils::storage::StorageService;

/// Counter component for displaying online users count following UI mock style
#[component]
pub fn OnlineUsersCounter() -> impl IntoView {
    let (online_count, set_online_count) = create_signal::<Option<i32>>(None);
    let (initial_load_done, set_initial_load_done) = create_signal(false);

    // Get WebSocket context to listen for join/leave events
    let ws_ctx_opt = use_context::<Option<UseGroupMessageWs>>();

    // Defer authoritative initial load until after WS join is acknowledged.
    // Show a spinner while waiting.
    let (join_request_id, set_join_request_id) = create_signal::<Option<String>>(None);
    let (join_acknowledged, set_join_acknowledged) = create_signal(false);

    // Mounted flag to avoid touching signals after the component is disposed
    let mounted = Arc::new(AtomicBool::new(true));
    let mounted_for_cleanup = mounted.clone();
    on_cleanup(move || {
        mounted_for_cleanup.store(false, Ordering::SeqCst);
    });

    // When WS becomes open, send a join request (once) and wait for its response
    let ws_ctx_for_join = ws_ctx_opt.clone();
    let mounted_for_join = mounted.clone();
    create_effect(move |_| {
        if initial_load_done.get() {
            return;
        }

    if let Some(Some(ws)) = &ws_ctx_for_join {
            // If socket is open and we haven't sent join yet, send it
            if ws.status.get_untracked() == crate::types::message_ws::WsStatus::Open {
                if join_request_id.get().is_none() {
                    let req_id = uuid::Uuid::new_v4().to_string();
                    set_join_request_id.set(Some(req_id.clone()));
                    ws.send_message.set(Some(crate::types::WebSocketMessage::Request {
                        request_id: req_id.clone(),
                        action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                    }));
                    leptos::logging::log!("[ONLINE COUNTER] Sent join request {}", req_id);

                    // Fallback: if join ack not received in reasonable time, proceed anyway
                    let join_ack_write = set_join_acknowledged.clone();
                    let mounted_for_timeout = mounted_for_join.clone();
                    // Use cancelable timeout stored in Rc<RefCell<Option<...>>> so we can cancel it on cleanup
                    let timeout = gloo_timers::callback::Timeout::new(6000, move || {
                        if mounted_for_timeout.load(Ordering::SeqCst) {
                            leptos::logging::log!("[ONLINE COUNTER] Join ack timeout for {}, proceeding anyway", req_id);
                            join_ack_write.set(true);
                        }
                    });
                    let timeout_store: Rc<RefCell<Option<gloo_timers::callback::Timeout>>> = Rc::new(RefCell::new(Some(timeout)));
                    let timeout_for_cleanup = timeout_store.clone();
                    on_cleanup(move || {
                        if let Some(t) = timeout_for_cleanup.borrow_mut().take() {
                            t.cancel();
                        }
                    });
                }

                // Check messages buffer for a response matching our join request
                let messages = ws.messages.get();
                if let Some(req_id) = join_request_id.get() {
                    leptos::logging::log!("[ONLINE COUNTER] Checking messages for join ack; looking for {} ({} messages)", req_id, messages.len());
                    for msg in messages.iter().rev() {
                        // log response messages for diagnostics
                        match msg {
                            crate::types::WebSocketMessage::Response { request_id, ok, .. } => {
                                leptos::logging::log!("[ONLINE COUNTER] saw Response req={} ok={}", request_id, ok);
                                // Accept either matching request_id OR any ok response after WS open
                                if (*request_id == req_id && *ok) || *ok {
                                    leptos::logging::log!("[ONLINE COUNTER] Join ack received (via response) {}", request_id);
                                    // Only set if component still mounted
                                    if mounted_for_join.load(Ordering::SeqCst) {
                                        set_join_acknowledged.set(true);
                                    }
                                    break;
                                }
                            }
                            crate::types::WebSocketMessage::Event { event, .. } => {
                                // presence events may be useful to see
                                if let ServerEvent::Groups(g) = event {
                                    match g {
                                        GroupEvent::Joined { user_id } => leptos::logging::log!("[ONLINE COUNTER] saw Joined event for {}", user_id),
                                        GroupEvent::Left { user_id } => leptos::logging::log!("[ONLINE COUNTER] saw Left event for {}", user_id),
                                        GroupEvent::NewGroupMembership { group_id, new_membership_username } => {
                                            leptos::logging::log!("[ONLINE COUNTER] saw NewGroupMembership event: {} joined group {}", new_membership_username, group_id);
                                        },
                                        GroupEvent::LeftGroupMembership { group_id, left_membership_username } => {
                                            leptos::logging::log!("[ONLINE COUNTER] saw LeftGroupMembership event: {} left group {}", left_membership_username, group_id);
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    });

    // Ensure the authoritative initial load runs once after join ack or after a timeout.
    // We use a dedicated async watcher to avoid reactive-scope disposal issues.
    {
        let join_ack_read = join_acknowledged.clone();
        let initial_done_read = initial_load_done.clone();
        let set_initial_done = set_initial_load_done.clone();
        let set_online = set_online_count.clone();
        let mounted_watcher = mounted.clone();

        spawn_local(async move {
            // Poll up to 6s for join acknowledgement
            let mut waited = 0u32;
            while waited < 2000 {
                if !mounted_watcher.load(Ordering::SeqCst) {
                    return; // component unmounted
                }
                if join_ack_read.get_untracked() {
                    break;
                }
                gloo_timers::future::TimeoutFuture::new(100).await;
                waited += 100;
            }

            // If initialization already done, nothing to do
            if initial_done_read.get_untracked() {
                return;
            }

            // Mark as done to avoid races
            set_initial_done.set(true);

            let storage_service = StorageService::new();
            if let Some(token_response) = storage_service.get_token() {
                let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                http_client.set_auth_token(Some(token_response.token));
                let membership_service = GroupMembershipService::new(http_client, storage_service.clone());

                // Debug: confirm token available (do not print the token itself)
                leptos::logging::log!("[ONLINE COUNTER] Token present, delaying 500ms then calling find_connected_users_and_online()");
                // Delay to give server time to finalize join processing
                gloo_timers::future::TimeoutFuture::new(500).await;

                match membership_service.find_connected_users_and_online().await {
                    Ok(online_user_ids) => {
                        let total_count = online_user_ids.len() as i32 + 1;
                        if mounted_watcher.load(Ordering::SeqCst) {
                            set_online.set(Some(total_count));
                        }
                        log::info!("Initial online users count: {}", total_count);
                    }
                    Err(e) => {
                        log::warn!("Failed to load initial online users count: {:?}", e);
                        if mounted_watcher.load(Ordering::SeqCst) {
                            set_online.set(Some(1));
                        }
                    }
                }
            } else {
                log::warn!("No token available for online users count");
                if mounted_watcher.load(Ordering::SeqCst) {
                    set_online.set(Some(1));
                }
            }
        });
    }

    // Listen for WebSocket join/leave events to update counter in real-time
    let ws_ctx_for_msgs = ws_ctx_opt.clone();
    let mounted_for_msgs = mounted.clone();
    create_effect(move |prev_message_count: Option<usize>| {
        if let Some(Some(ws)) = &ws_ctx_for_msgs {
            let messages = ws.messages.get();
            let current_count = messages.len();

            // Only process new messages to avoid re-processing
            if let Some(prev_count) = prev_message_count {
                if current_count > prev_count {
                    // Process only new messages
                    for message in messages.iter().skip(prev_count) {
                        if let WebSocketMessage::Event { event, .. } = message {
                            match event {
                                ServerEvent::Groups(GroupEvent::Joined { user_id }) => {
                                    // Increment counter when someone joins
                                    if mounted_for_msgs.load(Ordering::SeqCst) {
                                        set_online_count.update(|count| {
                                            if let Some(current) = count {
                                                let new_count = *current + 1;
                                                *count = Some(new_count);
                                                log::info!("User {} joined, new count: {}", user_id, new_count);
                                            }
                                        });
                                    }
                                }
                                ServerEvent::Groups(GroupEvent::Left { user_id }) => {
                                    // Decrement counter when someone leaves
                                    if mounted_for_msgs.load(Ordering::SeqCst) {
                                        set_online_count.update(|count| {
                                            if let Some(current) = count {
                                                let new_count = (*current - 1).max(1); // Never go below 1 (current user)
                                                *count = Some(new_count);
                                                log::info!("User {} left, new count: {}", user_id, new_count);
                                            }
                                        });
                                    }
                                }
                                ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, new_membership_username }) => {
                                    // New membership might affect online count, refresh from API
                                    if mounted_for_msgs.load(Ordering::SeqCst) {
                                        log::info!("New member {} joined group {}, refreshing online count", new_membership_username, group_id);
                                        
                                        let storage_service = crate::utils::storage::StorageService::new();
                                        if let Some(token_response) = storage_service.get_token() {
                                            let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                                            http_client.set_auth_token(Some(token_response.token));
                                            let membership_service = crate::api::services::membership::GroupMembershipService::new(http_client, storage_service);
                                            
                                            spawn_local(async move {
                                                match membership_service.find_connected_users_and_online().await {
                                                    Ok(users) => {
                                                        set_online_count.set(Some(users.len() as i32));
                                                        log::info!("Refreshed online count after new membership: {}", users.len());
                                                    }
                                                    Err(e) => {
                                                        log::error!("Failed to refresh online count after new membership: {:?}", e);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                                    // Member left might affect online count, refresh from API
                                    if mounted_for_msgs.load(Ordering::SeqCst) {
                                        log::info!("Member {} left group {}, refreshing online count", left_membership_username, group_id);
                                        
                                        let storage_service = crate::utils::storage::StorageService::new();
                                        if let Some(token_response) = storage_service.get_token() {
                                            let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                                            http_client.set_auth_token(Some(token_response.token));
                                            let membership_service = crate::api::services::membership::GroupMembershipService::new(http_client, storage_service);
                                            
                                            spawn_local(async move {
                                                match membership_service.find_connected_users_and_online().await {
                                                    Ok(users) => {
                                                        set_online_count.set(Some(users.len() as i32));
                                                        log::info!("Refreshed online count after member left: {}", users.len());
                                                    }
                                                    Err(e) => {
                                                        log::error!("Failed to refresh online count after member left: {:?}", e);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            current_count
        } else {
            prev_message_count.unwrap_or(0)
        }
    });

    view! {
        // Render as a dedicated bottom section, centered like the UI mock
        <div class="w-full mt-auto px-4 py-3">
            <div class="bg-white dark:bg-surface-dark border border-border dark:border-border-dark rounded-md p-3 flex items-center justify-start">
                <div class="flex items-center gap-3">
                    <span class="w-3 h-3 rounded-full bg-green-500 ring-2 ring-white dark:ring-gray-800 animate-pulse" />
                    <div class="text-left">
                        <div class="text-sm font-medium text-text-primary dark:text-text-primary-dark">"Online"</div>
                        <div class="text-sm text-text-secondary dark:text-text-secondary-dark">
                            {move || match online_count.get() {
                                Some(count) => format!("{} membri online", count),
                                None => {
                                    // show a small circular loader while waiting for join + initial load
                                    "".to_string()
                                }
                            }}
                        </div>
                    </div>
                    {move || if online_count.get().is_none() {
                        view! {
                            <div class="ml-2">
                                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-brand-primary" />
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }}
                </div>
            </div>
        </div>
    }
}
