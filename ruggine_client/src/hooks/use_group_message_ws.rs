use leptos::*;
use crate::api::MessageWsService;
use crate::config::endpoints::WebSocketEndpoints;
use crate::config::constants::AppConstants;
use crate::types::WebSocketMessage;

#[derive(Debug, Clone, PartialEq)]
pub enum WsStatus {
    Connecting,
    Open,
    Closed,
    Error(String),
}

#[derive(Clone)]
pub struct UseGroupMessageWs {
    pub status: ReadSignal<WsStatus>,
    pub send_message: WriteSignal<Option<WebSocketMessage>>,
    pub messages: ReadSignal<Vec<WebSocketMessage>>,
    pub disconnect: WriteSignal<bool>,
}

pub fn use_group_message_ws(token: String) -> UseGroupMessageWs {
    let (status, set_status) = create_signal(WsStatus::Connecting);
    let (messages, set_messages) = create_signal(Vec::new());
    let (send_message, set_send_message) = create_signal(None::<WebSocketMessage>);
    let (disconnect, set_disconnect) = create_signal(false);

    // Use a RefCell to hold the service instance
    let ws_service = std::rc::Rc::new(std::cell::RefCell::new(MessageWsService::new()));

    // Connect on mount
    {
        let ws_service = ws_service.clone();
        let set_status = set_status.clone();
        let url = WebSocketEndpoints::group_websocket_url(AppConstants::DEFAULT_SERVER_URL, &token);
        create_effect(move |_| {
            ws_service.borrow_mut().connect(&url);
            set_status.set(WsStatus::Connecting);
            let ws_service_cleanup = ws_service.clone();
            let set_status_cleanup = set_status.clone();
            on_cleanup(move || {
                ws_service_cleanup.borrow_mut().disconnect();
                set_status_cleanup.set(WsStatus::Closed);
            });
        });
    }

    // Send message effect
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

    // Disconnect effect
    {
        let ws_service = ws_service.clone();
        let set_status = set_status.clone();
        let set_disconnect = set_disconnect.clone();
        create_effect(move |_| {
            if disconnect.get() {
                ws_service.borrow_mut().disconnect();
                set_status.set(WsStatus::Closed);
                set_disconnect.set(false);
            }
        });
    }

    UseGroupMessageWs {
        status,
        send_message: set_send_message,
        messages,
        disconnect: set_disconnect,
    }
}
