use leptos::*;
use crate::api::ws::global_ws;
use crate::types::WebSocketMessage;
use crate::types::message_ws::WsStatus;
use crate::context::unread_counts_context::use_unread_counts_context;
use crate::types::message_ws::{ServerEvent, GroupEvent};
use crate::utils::storage::StorageService;
use crate::hooks::unread_helpers::increment_unread_map;

#[derive(Clone, PartialEq)]
pub struct UseGroupMessageWs {
    pub status: ReadSignal<WsStatus>,
    pub send_message: WriteSignal<Option<WebSocketMessage>>,
    pub messages: ReadSignal<Vec<WebSocketMessage>>,
    /// True when server confirmed a join for the tracked request id
    pub join_confirmed: ReadSignal<bool>,
    /// The last join request id tracked by this hook (if any)
    pub last_join_request_id: ReadSignal<Option<String>>,
    /// Request this instance to reset any join-tracking state and clear the buffer
    pub reset_tracking: WriteSignal<bool>,
    pub disconnect: WriteSignal<bool>,
}

pub fn use_group_message_ws(token: String) -> UseGroupMessageWs {
    // Basic version: no polling, no auto-reconnect, just instantiation and direct management
    let (messages, set_messages) = create_signal(Vec::<WebSocketMessage>::new());
    let (send_message, set_send_message) = create_signal(None::<WebSocketMessage>);
    // Track the last join request id so we can detect the matching response
    let (last_join_request_id, set_last_join_request_id) = create_signal::<Option<String>>(None);
    // Expose a join_confirmed signal that becomes true when the server responds ok to the join request
    let (join_confirmed, set_join_confirmed) = create_signal(false);
    let (disconnect, set_disconnect) = create_signal(false);
    // External reset trigger to clear stale state on login or reconnect flows
    let (reset_tracking_flag, set_reset_tracking_flag) = create_signal(false);

    // Store the current WebSocket service reference in a signal so we can update it
    let (ws_service, set_ws_service) = create_signal(None::<std::rc::Rc<std::cell::RefCell<crate::api::MessageWsService>>>);
    let (global_status, set_global_status) = create_signal(WsStatus::Closed);
    
    // Store callback_id in a signal so we can update it when the service changes
    let (callback_id, set_callback_id) = create_signal(None::<usize>);

    // Capture global unread_counts context here so we can update badges from the WS callback
    let unread_counts = use_unread_counts_context();

    // Clone the signal to ensure it is shared between closures and the hook
    let set_messages_shared = set_messages.clone();
    
    // Effect to manage WebSocket service connection based on token
    {
        let token_for_effect = token.clone();
        let set_ws_service = set_ws_service.clone();
        let set_global_status = set_global_status.clone();
        let set_callback_id = set_callback_id.clone();
        let set_messages_shared = set_messages_shared.clone();
        let set_join_confirmed = set_join_confirmed.clone();
        let last_join_request_id = last_join_request_id.clone();
        
        create_effect(move |_| {
            // Get or create the WebSocket service for the current token
            let (ws_svc, global_stat) = global_ws::get_or_create(&token_for_effect);
            
            // Remove old callback if exists
            if let Some(old_id) = callback_id.get_untracked() {
                if let Some(old_service) = ws_service.get_untracked() {
                    old_service.borrow_mut().remove_on_message(old_id);
                }
            }
            
            // Register callback for the new service
            let set_join_confirmed_cb = set_join_confirmed.clone();
            let last_join_request_id_cb = last_join_request_id.clone();
            let token_for_callback = token_for_effect.clone();
            let new_callback_id = ws_svc.borrow_mut().add_on_message({
                move |msg: WebSocketMessage| {
                    // Log the parsed message JSON for debugging (helps correlate with raw frames)
                    match serde_json::to_string(&msg) {
                        Ok(json) => leptos::logging::log!("[WS RAW CALLBACK] Parsed message JSON: {}", json),
                        Err(e) => leptos::logging::log!("[WS RAW CALLBACK] Failed to serialize parsed message: {:?}", e),
                    }

                    // push incoming message into local messages buffer
                    set_messages_shared.update(|msgs| {
                        msgs.push(msg.clone());
                    });

                    // If this is a Response matching the last join request id and it's ok, mark join_confirmed
                    if let WebSocketMessage::Response { request_id, ok, .. } = &msg {
                        if *ok {
                            // Check against global join tracking first
                            if global_ws::confirm_join(&token_for_callback, request_id) {
                                set_join_confirmed_cb.set(true);
                                leptos::logging::log!("[WS INST] Join confirmed globally for request {}", request_id);
                            }
                            // Also check against this hook's local tracking for backwards compatibility
                            if let Some(local_req_id) = last_join_request_id_cb.get_untracked() {
                                if local_req_id == *request_id {
                                    set_join_confirmed_cb.set(true);
                                    leptos::logging::log!("[WS INST] Join confirmed locally for request {}", request_id);
                                }
                            }
                        }
                    }

                    // Pass certain events to the unread count manager (no-op if not eligible)
                    handle_unread_events(&msg, &unread_counts);
                }
            });
            
            // Update the stored service and status
            set_ws_service.set(Some(ws_svc));
            set_global_status.set(global_stat.get_untracked());
            set_callback_id.set(Some(new_callback_id));
            
            leptos::logging::log!("[WS INST] Connected to WebSocket service for token");
        });
    }
                        leptos::logging::log!("[WS RAW CALLBACK] Global join confirmed, setting local join_confirmed=true (request_id={})", request_id);
                    }
                    // Also check local tracking for backwards compatibility
                    if let Some(last_req) = last_join_request_id_cb.get_untracked() {
                        if last_req == *request_id {
                            set_join_confirmed_cb.set(true);
                            leptos::logging::log!("[WS RAW CALLBACK] Local join response matched, setting join_confirmed=true (request_id={})", request_id);
                        }
                    }
                }
            }
        }
    });

    // Attempt to register a single global unread/invitation handler. Only the
    // first caller will succeed; subsequent callers will get None and skip
    // registering the global handler to avoid duplicate increments.
    let unread_handler_ws = ws_service.clone();
    let maybe_unread_id = ws_service.borrow_mut().add_unread_incrementer({
        move |msg: WebSocketMessage| {
            if let WebSocketMessage::Event { event, .. } = &msg {
                // Log presence events for debugging
                if let ServerEvent::Groups(GroupEvent::Joined { user_id }) = event {
                    leptos::logging::log!("[WS DEBUG] Joined event received for user {}", user_id);
                } else if let ServerEvent::Groups(GroupEvent::Left { user_id }) = event {
                    leptos::logging::log!("[WS DEBUG] Left event received for user {}", user_id);
                }
                if let ServerEvent::Groups(GroupEvent::NewMessage { message_id: _, group_id, sender_id, sender_username: _, content: _, sent_at: _ }) = event {
                    // avoid increment for messages sent by current user
                    let current_user_id = StorageService::new().get_user_profile().map(|u| u.id);
                    if Some(*sender_id) != current_user_id {
                        // Update in-place to avoid clobbering concurrent updates from other tasks
                        unread_counts.update(|map| {
                            increment_unread_map(map, *group_id, Some(*sender_id), current_user_id);
                        });
                    }
                }
                // Handle new invitation events
                if let ServerEvent::Groups(GroupEvent::NewInvitation { invitation_id }) = event {
                    leptos::logging::log!("[WS DEBUG] NewInvitation event received for invitation {}", invitation_id);
                    // Update invitations context
                    spawn_local({
                        let invitation_id = *invitation_id;
                        async move {
                            crate::context::invitations_context::add_new_invitation(invitation_id).await;
                        }
                    });
                }
            }
        }
    });

    // Ensure we remove the callback when the scope using this hook is disposed
    {
        let ws_service = ws_service.clone();
        on_cleanup(move || {
            ws_service.borrow_mut().remove_on_message(callback_id);
        });
    }

    // Do NOT auto-connect here. Connection is triggered centrally (e.g. at login)
    // to ensure the socket is opened only once when the user logs in and
    // remains available across navigation. The global registry still provides
    // the shared service and status signal.

    // Effect for sending messages
    {
        let ws_service = ws_service.clone();
        let set_send_message = set_send_message.clone();
        let set_last_join_request_id = set_last_join_request_id.clone();
        let set_join_confirmed_effect = set_join_confirmed.clone();
        create_effect(move |_| {
            if let Some(msg) = send_message.get() {
                let ws_service = ws_service.clone();
                let set_send_message = set_send_message.clone();
                // perform mutable borrow inside a separate task to avoid re-entrant RefCell borrows
                spawn_local(async move {
                        // If this is a Join request, clear any previous confirmation and wait for the socket to be Open
                        if let WebSocketMessage::Request { request_id, action } = &msg {
                            if let crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}) = action {
                                // Reset confirmation so stale confirmations from earlier sessions don't trigger logic
                                set_join_confirmed_effect.set(false);

                                // Wait a short while for the global socket to be Open before sending
                                let mut sent = false;
                                for _ in 0..20 {
                                    // Check global status without tracking
                                    let status_val = global_status.get_untracked();
                                    if status_val == WsStatus::Open {
                                        // mark the tracked id and send
                                        set_last_join_request_id.set(Some(request_id.clone()));
                                        leptos::logging::log!("[WS INST] Sending join request id {} after socket Open", request_id);
                                        ws_service.borrow_mut().send(&msg);
                                        sent = true;
                                        break;
                                    }
                                    // sleep 100ms
                                    gloo_timers::future::TimeoutFuture::new(100).await;
                                }
                                if !sent {
                                    leptos::logging::log!("[WS INST] Failed to send join request {}: socket never opened", request_id);
                                    // clear tracked id to avoid false positives
                                    set_last_join_request_id.set(None);
                                }
                            } else {
                                // Non-join requests: try to send immediately (best-effort)
                                ws_service.borrow_mut().send(&msg);
                            }
                        } else {
                            // Not a request (e.g., event) - send as-is
                            ws_service.borrow_mut().send(&msg);
                        }
                    set_send_message.set(None);
                });
            }
        });
    }

    // Effect to handle external reset of join-tracking and local buffer
    {
        let set_join_confirmed = set_join_confirmed.clone();
        let set_last_join_request_id = set_last_join_request_id.clone();
        let set_messages_reset = set_messages.clone();
        let set_reset_tracking_flag = set_reset_tracking_flag.clone();
        let token_for_reset = token.clone();
        create_effect(move |_| {
            if reset_tracking_flag.get() {
                // Clear tracking and buffer
                set_join_confirmed.set(false);
                set_last_join_request_id.set(None);
                set_messages_reset.set(Vec::new());
                // Also reset global tracking to ensure consistency
                global_ws::reset_join_tracking(&token_for_reset);
                // acknowledge and lower the flag
                set_reset_tracking_flag.set(false);
                leptos::logging::log!("[WS INST] Reset tracking state and cleared message buffer");
            }
        });
    }

    // Effect for manual disconnection
    {
        let ws_service = ws_service.clone();
        let set_disconnect = set_disconnect.clone();
        create_effect(move |_| {
            if disconnect.get() {
                let ws_service = ws_service.clone();
                let set_disconnect = set_disconnect.clone();
                spawn_local(async move {
                    ws_service.borrow_mut().disconnect();
                    set_disconnect.set(false);
                });
            }
        });
    }

    // Note: do NOT disconnect on unmount — keep the global connection alive so
    // page refreshes or navigations reuse the same socket.

    UseGroupMessageWs {
    status: global_status,
        send_message: set_send_message,
        messages,
    join_confirmed: join_confirmed.clone(),
    last_join_request_id: last_join_request_id.clone(),
    reset_tracking: set_reset_tracking_flag,
        disconnect: set_disconnect,
    }
}
