use leptos::*;
use crate::api::MessageWsService;
use crate::config::endpoints::WebSocketEndpoints;
use crate::config::constants::AppConstants;
use crate::types::WebSocketMessage;
use crate::types::message_ws::WsStatus;

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

    // Clone the signal to ensure it is shared between closures and the hook
    let set_messages_shared = set_messages.clone();
    let ws_service = std::rc::Rc::new(std::cell::RefCell::new(MessageWsService::new(status)));
    ws_service.borrow_mut().set_on_message(move |msg: WebSocketMessage| {
        set_messages_shared.update(|msgs| {
            msgs.push(msg.clone());
        });
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
