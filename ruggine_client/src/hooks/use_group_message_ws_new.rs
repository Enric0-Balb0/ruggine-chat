use leptos::*;
use crate::api::ws::global_ws;
use crate::types::WebSocketMessage;
use crate::types::message_ws::WsStatus;
use crate::context::unread_counts_context::use_unread_counts_context;
use crate::types::message_ws::{ServerEvent, GroupEvent};
use crate::utils::storage::StorageService;

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
    // Basic signals for the hook interface
    let (messages, set_messages) = create_signal(Vec::<WebSocketMessage>::new());
    let (send_message, set_send_message) = create_signal(None::<WebSocketMessage>);
    let (last_join_request_id, set_last_join_request_id) = create_signal::<Option<String>>(None);
    let (join_confirmed, set_join_confirmed) = create_signal(false);
    let (disconnect, set_disconnect) = create_signal(false);
    let (reset_tracking_flag, set_reset_tracking_flag) = create_signal(false);

    // Store the current WebSocket service reference in a signal so we can update it
    let (ws_service, set_ws_service) = create_signal(None::<std::rc::Rc<std::cell::RefCell<crate::api::MessageWsService>>>);
    let (global_status, set_global_status) = create_signal(WsStatus::Closed);
    let (callback_id, set_callback_id) = create_signal(None::<usize>);

    // Capture global unread_counts context
    let unread_counts = use_unread_counts_context();
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
            let set_messages_for_callback = set_messages_shared.clone();
            let unread_counts_for_callback = unread_counts.clone();
            
            let new_callback_id = ws_svc.borrow_mut().add_on_message({
                move |msg: WebSocketMessage| {
                    // Log the parsed message JSON for debugging
                    match serde_json::to_string(&msg) {
                        Ok(json) => leptos::logging::log!("[WS RAW CALLBACK] Parsed message JSON: {}", json),
                        Err(e) => leptos::logging::log!("[WS RAW CALLBACK] Failed to serialize parsed message: {:?}", e),
                    }

                    // Push incoming message into local messages buffer
                    set_messages_for_callback.update(|msgs| {
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

                    // Handle unread count updates
                    handle_unread_events(&msg, &unread_counts_for_callback);
                }
            });
            
            // Update the stored service and status
            set_ws_service.set(Some(ws_svc));
            set_global_status.set(global_stat.get_untracked());
            set_callback_id.set(Some(new_callback_id));
            
            leptos::logging::log!("[WS INST] Connected to WebSocket service for token");
        });
    }

    // Ensure we remove the callback when the scope using this hook is disposed
    {
        let ws_service = ws_service.clone();
        let callback_id = callback_id.clone();
        on_cleanup(move || {
            if let (Some(service), Some(id)) = (ws_service.get_untracked(), callback_id.get_untracked()) {
                service.borrow_mut().remove_on_message(id);
            }
        });
    }

    // Effect for sending messages
    {
        let ws_service = ws_service.clone();
        let set_send_message = set_send_message.clone();
        let set_last_join_request_id = set_last_join_request_id.clone();
        let set_join_confirmed_effect = set_join_confirmed.clone();
        let global_status = global_status.clone();
        
        create_effect(move |_| {
            if let Some(msg) = send_message.get() {
                let ws_service = ws_service.clone();
                let set_send_message = set_send_message.clone();
                let global_status = global_status.clone();
                
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
                                    if let Some(service) = ws_service.get_untracked() {
                                        service.borrow().send(&msg);
                                    }
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
                            if let Some(service) = ws_service.get_untracked() {
                                service.borrow().send(&msg);
                            }
                        }
                    } else {
                        // Not a request (e.g., event) - send as-is
                        if let Some(service) = ws_service.get_untracked() {
                            service.borrow().send(&msg);
                        }
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
                global_ws::reset_join_tracking(&token_for_reset);
                // reset the flag
                set_reset_tracking_flag.set(false);
                leptos::logging::log!("[WS INST] Reset join tracking and local buffer");
            }
        });
    }

    // Effect to handle disconnect requests
    {
        let ws_service = ws_service.clone();
        let set_disconnect = set_disconnect.clone();
        
        create_effect(move |_| {
            if disconnect.get() {
                if let Some(service) = ws_service.get_untracked() {
                    service.borrow_mut().disconnect();
                }
                set_disconnect.set(false);
                leptos::logging::log!("[WS INST] Disconnected WebSocket service");
            }
        });
    }

    UseGroupMessageWs {
        status: global_status.read_only(),
        send_message: set_send_message,
        messages: messages.read_only(),
        join_confirmed: join_confirmed.read_only(),
        last_join_request_id: last_join_request_id.read_only(),
        reset_tracking: set_reset_tracking_flag,
        disconnect: set_disconnect,
    }
}

fn handle_unread_events(msg: &WebSocketMessage, unread_counts: &Option<crate::context::unread_counts_context::UnreadCountsContext>) {
    if let Some(handler) = unread_counts {
        match msg {
            WebSocketMessage::Event { data: ServerEvent::Groups(GroupEvent::Message { group_id, message: _ }) } => {
                // Only increment if the message is in a group that exists in the current context
                handler.update_group_unread(group_id, 1);
            }
            WebSocketMessage::Event { data: ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, membership: _ }) } => {
                // User added to a new group, don't increment but ensure group exists in the map
                handler.ensure_group_exists(group_id);
            }
            _ => {}
        }
    }
}
