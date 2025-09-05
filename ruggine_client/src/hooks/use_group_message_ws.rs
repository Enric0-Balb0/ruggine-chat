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
    pub disconnect: WriteSignal<bool>,
}

pub fn use_group_message_ws(token: String) -> UseGroupMessageWs {
    // Basic version: no polling, no auto-reconnect, just instantiation and direct management
    let (messages, set_messages) = create_signal(Vec::<WebSocketMessage>::new());
    let (send_message, set_send_message) = create_signal(None::<WebSocketMessage>);
    let (disconnect, set_disconnect) = create_signal(false);

    // Capture global unread_counts context here so we can update badges from the WS callback
    let unread_counts = use_unread_counts_context();

    // Clone the signal to ensure it is shared between closures and the hook
    let set_messages_shared = set_messages.clone();
    // Attempt to reuse a global service keyed by token so refresh doesn't re-open a new connection
    let (ws_service, global_status) = global_ws::get_or_create(&token);
    // Instance identifier to help debug duplicate connections
    let instance_id = uuid::Uuid::new_v4().to_string();
    leptos::logging::log!("[WS INST] Created UseGroupMessageWs instance {} (reused)", instance_id);
    ws_service.borrow_mut().set_on_message({
        let _ws_service = ws_service.clone();
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

            // If this is a Group NewMessage event, increment the global unread_counts
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
                    }
        }
    });

    // Do NOT auto-connect here. Connection is triggered centrally (e.g. at login)
    // to ensure the socket is opened only once when the user logs in and
    // remains available across navigation. The global registry still provides
    // the shared service and status signal.

    // Effect for sending messages
    {
        let ws_service = ws_service.clone();
        let set_send_message = set_send_message.clone();
        create_effect(move |_| {
            if let Some(msg) = send_message.get() {
                let ws_service = ws_service.clone();
                let set_send_message = set_send_message.clone();
                // perform mutable borrow inside a separate task to avoid re-entrant RefCell borrows
                spawn_local(async move {
                    ws_service.borrow_mut().send(&msg);
                    set_send_message.set(None);
                });
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
        disconnect: set_disconnect,
    }
}
