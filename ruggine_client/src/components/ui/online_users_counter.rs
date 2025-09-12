use leptos::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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

    // Defer authoritative initial load until the WS is open.
    // Show a spinner while waiting.

    // Mounted flag to avoid touching signals after the component is disposed
    let mounted = Arc::new(AtomicBool::new(true));
    let mounted_for_cleanup = mounted.clone();
    on_cleanup(move || {
        mounted_for_cleanup.store(false, Ordering::SeqCst);
    });

    // No join requests are sent from this component. It only observes the
    // shared WebSocket status to decide when to perform the authoritative
    // API load for online users.

    // Ensure the authoritative initial load runs once after WS is Open or after a timeout.
    // We use a dedicated async watcher to avoid reactive-scope disposal issues.
    {
        let initial_done_read = initial_load_done.clone();
        let set_initial_done = set_initial_load_done.clone();
        let set_online = set_online_count.clone();
        let mounted_watcher = mounted.clone();
        let ws_ctx_clone = ws_ctx_opt.clone();

        spawn_local(async move {
            // Wait up to 2s for WS to become Open, polling every 100ms
            let mut waited = 0u32;
            while waited < 2000 {
                if !mounted_watcher.load(Ordering::SeqCst) {
                    return; // component unmounted
                }
                if let Some(Some(ws)) = &ws_ctx_clone {
                    if ws.status.get_untracked() == crate::types::message_ws::WsStatus::Open {
                        break;
                    }
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
                // Delay a bit to give server time to finalize any join processing
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
                    set_online.set(Some(0));
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
                                        let storage_service = crate::utils::storage::StorageService::new();
                                        if let Some(token_response) = storage_service.get_token() {
                                            let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                                            http_client.set_auth_token(Some(token_response.token));
                                            let membership_service = crate::api::services::membership::GroupMembershipService::new(http_client, storage_service);
                                            let mounted_clone = mounted_for_msgs.clone();
                                            
                                            spawn_local(async move {
                                                match membership_service.find_connected_users_and_online().await {
                                                    Ok(users) => {
                                                        if mounted_clone.load(Ordering::SeqCst) {
                                                            set_online_count.set(Some(users.len() as i32));
                                                            log::info!("Refreshed online count after member joined connection: {}", users.len());
                                                        }
                                                    }
                                                    Err(e) => {
                                                        // If we get 404, it means we're no longer in the group
                                                        if e.to_string().contains("404") || e.to_string().contains("Group membership not found") {
                                                            log::info!("No longer member of group, setting online count to 0");
                                                            if mounted_clone.load(Ordering::SeqCst) {
                                                                set_online_count.set(Some(0));
                                                            }
                                                        } else {
                                                            log::error!("Failed to refresh online count after member joined connection: {:?}", e);
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                ServerEvent::Groups(GroupEvent::Left { user_id }) => {
                                    // Decrement counter when someone leaves
                                    if mounted_for_msgs.load(Ordering::SeqCst) {
                                        let storage_service = crate::utils::storage::StorageService::new();
                                        if let Some(token_response) = storage_service.get_token() {
                                            let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                                            http_client.set_auth_token(Some(token_response.token));
                                            let membership_service = crate::api::services::membership::GroupMembershipService::new(http_client, storage_service);
                                            let mounted_clone = mounted_for_msgs.clone();
                                            
                                            spawn_local(async move {
                                                match membership_service.find_connected_users_and_online().await {
                                                    Ok(users) => {
                                                        if mounted_clone.load(Ordering::SeqCst) {
                                                            set_online_count.set(Some(users.len() as i32));
                                                            log::info!("Refreshed online count after member left connection: {}", users.len());
                                                        }
                                                    }
                                                    Err(e) => {
                                                        // If we get 404, it means we're no longer in the group
                                                        if e.to_string().contains("404") || e.to_string().contains("Group membership not found") {
                                                            log::info!("No longer member of group, setting online count to 0");
                                                            if mounted_clone.load(Ordering::SeqCst) {
                                                                set_online_count.set(Some(0));
                                                            }
                                                        } else {
                                                            log::error!("Failed to refresh online count after member left connection: {:?}", e);
                                                        }
                                                    }
                                                }
                                            });
                                        }
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
                                            let mounted_clone = mounted_for_msgs.clone();
                                            
                                            spawn_local(async move {
                                                match membership_service.find_connected_users_and_online().await {
                                                    Ok(users) => {
                                                        if mounted_clone.load(Ordering::SeqCst) {
                                                            set_online_count.set(Some(users.len() as i32));
                                                            log::info!("Refreshed online count after new membership: {}", users.len());
                                                        }
                                                    }
                                                    Err(e) => {
                                                        // If we get 404, it means we're no longer in the group
                                                        if e.to_string().contains("404") || e.to_string().contains("Group membership not found") {
                                                            log::info!("No longer member of group, setting online count to 0");
                                                            if mounted_clone.load(Ordering::SeqCst) {
                                                                set_online_count.set(Some(0));
                                                            }
                                                        } else {
                                                            log::error!("Failed to refresh online count after new membership: {:?}", e);
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                                    // Check if the current user left the group
                                    let storage_service = crate::utils::storage::StorageService::new();
                                    if let Some(user_profile) = storage_service.get_user_profile() {
                                        let current_username = user_profile.email.split('@').next().unwrap_or("");
                                        if current_username == left_membership_username || user_profile.email == *left_membership_username {
                                            // Current user left the group, stop updating online count
                                            log::info!("Current user {} left group {}, stopping online count updates", left_membership_username, group_id);
                                            if mounted_for_msgs.load(Ordering::SeqCst) {
                                                set_online_count.set(Some(0));
                                            }
                                            return current_count;
                                        }
                                    }
                                    
                                    // Member left might affect online count, refresh from API
                                    if mounted_for_msgs.load(Ordering::SeqCst) {
                                        log::info!("Member {} left group {}, refreshing online count", left_membership_username, group_id);
                                        
                                        let storage_service = crate::utils::storage::StorageService::new();
                                        if let Some(token_response) = storage_service.get_token() {
                                            let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                                            http_client.set_auth_token(Some(token_response.token));
                                            let membership_service = crate::api::services::membership::GroupMembershipService::new(http_client, storage_service);
                                            let mounted_clone = mounted_for_msgs.clone();
                                            
                                            spawn_local(async move {
                                                match membership_service.find_connected_users_and_online().await {
                                                    Ok(users) => {
                                                        if mounted_clone.load(Ordering::SeqCst) {
                                                            set_online_count.set(Some(users.len() as i32));
                                                            log::info!("Refreshed online count after member left group: {}", users.len());
                                                        }
                                                    }
                                                    Err(e) => {
                                                        // If we get 404, it means we're no longer in the group
                                                        if e.to_string().contains("404") || e.to_string().contains("Group membership not found") {
                                                            log::info!("No longer member of group, setting online count to 0");
                                                            if mounted_clone.load(Ordering::SeqCst) {
                                                                set_online_count.set(Some(0));
                                                            }
                                                        } else {
                                                            log::error!("Failed to refresh online count after member left group: {:?}", e);
                                                        }
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
