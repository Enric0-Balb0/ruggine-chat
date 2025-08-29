use leptos::*;
use crate::api::MessageWsService;
use crate::config::endpoints::WebSocketEndpoints;
use crate::config::constants::AppConstants;
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
    let status = leptos::create_rw_signal(WsStatus::Connecting);
    let (messages, set_messages) = create_signal(Vec::<WebSocketMessage>::new());
    let (send_message, set_send_message) = create_signal(None::<WebSocketMessage>);
    let (disconnect, set_disconnect) = create_signal(false);

    // Capture global unread_counts context here so we can update badges from the WS callback
    let unread_counts = use_unread_counts_context();

    // Clone the signal to ensure it is shared between closures and the hook
    let set_messages_shared = set_messages.clone();
    let ws_service = std::rc::Rc::new(std::cell::RefCell::new(MessageWsService::new(status)));
    ws_service.borrow_mut().set_on_message({
        let ws_service = ws_service.clone();
        move |msg: WebSocketMessage| {
            // push incoming message into local messages buffer
            set_messages_shared.update(|msgs| {
                msgs.push(msg.clone());
            });

            // If this is a Group NewMessage event, increment the global unread_counts
                    if let WebSocketMessage::Event { event, .. } = &msg {
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

    // Manual connection on mount and cleanup
    let url = WebSocketEndpoints::group_websocket_url(AppConstants::DEFAULT_SERVER_URL, &token);
    ws_service.borrow_mut().connect(&url);

    // Effect for sending messages
    {
        let ws_service = ws_service.clone();
        let set_send_message = set_send_message.clone();
        create_effect(move |_| {
            if let Some(msg) = send_message.get() {
                ws_service.borrow_mut().send(&msg);
                set_send_message.set(None);
            }
        });
    }

    // Effect for manual disconnection
    {
        let ws_service = ws_service.clone();
        let set_disconnect = set_disconnect.clone();
        create_effect(move |_| {
            if disconnect.get() {
                ws_service.borrow_mut().disconnect();
                set_disconnect.set(false);
            }
        });
    }

    // Cleanup on unmount
    on_cleanup(move || {
        ws_service.borrow_mut().disconnect();
    });

    UseGroupMessageWs {
        status: status.read_only(),
        send_message: set_send_message,
        messages,
        disconnect: set_disconnect,
    }
}
