use leptos::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::api::services::membership::GroupMembershipService;
use crate::types::message_ws::{ServerEvent, GroupEvent, WebSocketMessage, WsStatus};
use crate::hooks::use_group_message_ws::UseGroupMessageWs;
use crate::context::auth_context::use_auth_context;
use crate::utils::storage::StorageService;
use crate::api::ws::global_ws;

/// Counter component for displaying online users count following UI mock style
/// Solo si aggiorna all'inizio e per eventi Joined/Left
#[component]
pub fn OnlineUsersCounter() -> impl IntoView {
    let (online_count, set_online_count) = create_signal::<Option<i32>>(None);
    
    // Get WebSocket context (provided in `App` via `provide_context`)
    // `App` provides a `ReadSignal<Option<UseGroupMessageWs>>` as the context value.
    let ws_ctx_signal = use_context::<ReadSignal<Option<UseGroupMessageWs>>>().expect("WebSocket context should be provided");

    // Mounted flag to avoid touching signals after the component is disposed
    let mounted = Arc::new(AtomicBool::new(true));
    let mounted_for_cleanup = mounted.clone();
    on_cleanup(move || {
        mounted_for_cleanup.store(false, Ordering::SeqCst);
    });

    // Local flag for this component instance - not static/global
    let has_done_initial_fetch = Arc::new(AtomicBool::new(false));

    // 1. CARICAMENTO INIZIALE: Solo una volta quando il componente è creato - NO EFFECTS!
    leptos::logging::log!("[ONLINE COUNTER] Component mounted, starting initial load");
    
    {
        // Set initial fallback count immediately and wait for WebSocket events to trigger the first authoritative fetch
        let mounted_clone = mounted.clone();
        let set_count = set_online_count.clone();

        spawn_local(async move {
            // Set conservative fallback count immediately - no HTTP call until WS join confirmed
            if mounted_clone.load(Ordering::SeqCst) {
                set_count.set(Some(1)); // Conservative: assume local user is online
                leptos::logging::log!("[ONLINE COUNTER] Set initial fallback count: 1 (waiting for WS join)");
            }
        });
    }

    // 2. AGGIORNAMENTI REAL-TIME: per eventi Joined/Left e per NewGroupMembership/LeftGroupMembership
    // E per il primo join response (initial authoritative fetch)
    let last_event_signal = create_rw_signal::<Option<String>>(None);
    
    // Make effect reactive to authentication token so it runs when the user logs in
    // and the WS context becomes available. We read the WS context inside the effect
    // and then read `ws.messages.get()` so the effect also re-triggers on incoming WS messages.
    let auth_ctx = use_auth_context();
    let token_signal = auth_ctx.token.read_only();

    let mounted_for_events = mounted.clone();
    let has_done_initial_fetch_for_events = has_done_initial_fetch.clone();

    create_effect(move |_| {
        // Depend on token so this effect runs when login happens
        let token_opt = token_signal.get();
        leptos::logging::log!("[ONLINE COUNTER] Effect triggered; token_is_some={}", token_opt.is_some());

        // If not logged in yet, no ws context will be available
        if token_opt.is_none() {
            return;
        }

        // Get the current WebSocket context from the signal
        let ws_ctx_local = ws_ctx_signal.get();
        leptos::logging::log!("[ONLINE COUNTER] WS context available={}", ws_ctx_local.is_some());

        if let Some(ws) = ws_ctx_local {
            // Read messages to establish a reactive dependency on incoming messages
            let messages = ws.messages.get();
            leptos::logging::log!("[ONLINE COUNTER] Effect triggered, checking {} messages", messages.len());

            // Read join_confirmed signal to trigger the first authoritative fetch deterministically
            let join_confirmed = ws.join_confirmed.get();
            let last_join_req = ws.last_join_request_id.get();
            let status = ws.status.get();
            
            // Also check global join state to handle cross-instance join tracking
            let token_str = token_opt.clone().unwrap_or_default();
            let global_join_confirmed = global_ws::is_join_confirmed(&token_str);
            let global_last_join_req = global_ws::get_last_join_request_id(&token_str);
            
            leptos::logging::log!("[ONLINE COUNTER] Effect triggered, local join_confirmed={}, global join_confirmed={}, local last_join_req={:?}, global last_join_req={:?}, status={:?}", 
                join_confirmed, global_join_confirmed, last_join_req, global_last_join_req, status);

            // Use global join state if available, otherwise fall back to local
            let effective_join_confirmed = global_join_confirmed || join_confirmed;
            let effective_last_req = global_last_join_req.clone().or(last_join_req.clone());

            // Only act if we have an actual tracked join request, the socket is Open and the join was confirmed.
            // Also check that we have messages (which suggests WS activity after mount).
            // Additionally, ensure the join confirmation is for a current session (not stale).
            let is_fresh_join = if global_join_confirmed && global_last_join_req.is_some() {
                // If using global tracking, ensure it's confirmed
                true
            } else if join_confirmed && last_join_req.is_some() {
                // If using local tracking, ensure it matches some recent activity
                !messages.is_empty()
            } else {
                false
            };

            if effective_join_confirmed && effective_last_req.is_some() && status == WsStatus::Open && !has_done_initial_fetch_for_events.load(Ordering::SeqCst) && is_fresh_join {
                has_done_initial_fetch_for_events.store(true, Ordering::SeqCst);
                leptos::logging::log!("[ONLINE COUNTER] join_confirmed observed with fresh join, performing initial authoritative fetch");

                let mounted_clone = mounted_for_events.clone();
                let set_count = set_online_count.clone();

                spawn_local(async move {
                    // Small pause to allow backend to fully register the WS join even
                    // after the socket answered ok. This mitigates a narrow race
                    // where the server ack arrives before internal registration is done.
                    crate::utils::timers::sleep_ms(200).await;
                    let storage_service = StorageService::new();
                    if let Some(token_response) = storage_service.get_token() {
                        let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                        http_client.set_auth_token(Some(token_response.token));
                        let membership_service = GroupMembershipService::new(http_client, storage_service);

                        // Retry loop to tolerate a short race where the server hasn't yet registered
                        // the websocket join and returns 403. We only retry on 403/Forbidden.
                        let mut attempts = 0u8;
                        let max_attempts = 5u8;
                        loop {
                            match membership_service.find_connected_users_and_online().await {
                                Ok(online_user_ids) => {
                                    let total_count = online_user_ids.len() as i32 + 1;
                                    if mounted_clone.load(Ordering::SeqCst) {
                                        set_count.set(Some(total_count));
                                        leptos::logging::log!("[ONLINE COUNTER] Initial authoritative count loaded: {}", total_count);
                                    }
                                    break;
                                }
                                Err(e) => {
                                    let err_str = e.to_string();
                                    leptos::logging::log!("[ONLINE COUNTER] Failed to load initial authoritative count (attempt {}): {:?}", attempts, err_str);
                                    if attempts < max_attempts && (err_str.contains("403") || err_str.contains("Forbidden")) {
                                        attempts += 1;
                                        crate::utils::timers::sleep_ms(200).await;
                                        continue;
                                    }
                                    leptos::logging::log!("[ONLINE COUNTER] Giving up after {} attempts: {:?}", attempts, err_str);
                                    break;
                                }
                            }
                        }
                    }
                });
            }

            // Cerca l'ultimo evento rilevante (Joined/Left o membership change)
            let mut latest_event_id: Option<String> = None;
            for message_ref in messages.iter().rev() {
                // Clone to match by-value
                let message = message_ref.clone();
                if let WebSocketMessage::Event { event, timestamp } = message {
                    match event {
                        ServerEvent::Groups(GroupEvent::Joined { user_id }) => {
                            latest_event_id = Some(format!("joined_{}_{}", user_id, timestamp));
                            break;
                        }
                        ServerEvent::Groups(GroupEvent::Left { user_id }) => {
                            latest_event_id = Some(format!("left_{}_{}", user_id, timestamp));
                            break;
                        }
                        // When a membership is created (someone accepted an invite)
                        ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, new_membership_username: _ }) => {
                            latest_event_id = Some(format!("new_membership_{}_{}", group_id, timestamp));
                            break;
                        }
                        // When a membership is removed/left
                        ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username: _ }) => {
                            latest_event_id = Some(format!("left_membership_{}_{}", group_id, timestamp));
                            break;
                        }
                        // When a new group chat is created
                        ServerEvent::Groups(GroupEvent::NewGroupChat { group_chat_id }) => {
                            latest_event_id = Some(format!("new_group_chat_{}_{}", group_chat_id, timestamp));
                            break;
                        }
                        _ => continue,
                    }
                }
            }

            // Se abbiamo un nuovo evento rilevante, aggiorniamo il counter
            if let Some(event_id) = latest_event_id {
                if Some(event_id.clone()) != last_event_signal.get() {
                    last_event_signal.set(Some(event_id.clone()));
                    
                    leptos::logging::log!("[ONLINE COUNTER] Connection event detected: {}", event_id);
                    
                    let mounted_clone = mounted_for_events.clone();
                    let set_count = set_online_count.clone();
                    
                    spawn_local(async move {
                        let storage_service = StorageService::new();
                        if let Some(token_response) = storage_service.get_token() {
                            let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                            http_client.set_auth_token(Some(token_response.token));
                            let membership_service = GroupMembershipService::new(http_client, storage_service);

                            // Retry loop as above for transient 403 during server join registration
                            let mut attempts = 0u8;
                            let max_attempts = 5u8;
                            loop {
                                match membership_service.find_connected_users_and_online().await {
                                    Ok(online_user_ids) => {
                                        let total_count = online_user_ids.len() as i32 + 1;
                                        if mounted_clone.load(Ordering::SeqCst) {
                                            set_count.set(Some(total_count));
                                            leptos::logging::log!("[ONLINE COUNTER] Count updated after connection event: {}", total_count);
                                        }
                                        break;
                                    }
                                    Err(e) => {
                                        let err_str = e.to_string();
                                        leptos::logging::log!("[ONLINE COUNTER] Failed to update count (attempt {}): {:?}", attempts, err_str);
                                        if attempts < max_attempts && (err_str.contains("403") || err_str.contains("Forbidden")) {
                                            attempts += 1;
                                            crate::utils::timers::sleep_ms(200).await;
                                            continue;
                                        }
                                        // Special-case: if the group/membership is truly missing, show 0
                                        if err_str.contains("404") || err_str.contains("Group membership not found") {
                                            if mounted_clone.load(Ordering::SeqCst) {
                                                set_count.set(Some(0));
                                            }
                                        }
                                        leptos::logging::log!("[ONLINE COUNTER] Giving up after {} attempts: {:?}", attempts, err_str);
                                        break;
                                    }
                                }
                            }
                        }
                    });
                }
            }
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
