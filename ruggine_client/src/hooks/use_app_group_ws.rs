use leptos::*;
use crate::types::WebSocketMessage;
use crate::hooks::use_group_message_ws::{use_group_message_ws, UseGroupMessageWs};
use crate::types::message_ws::WsStatus;

/// Hook che gestisce la logica di lifecycle e join automatica del WebSocket dopo login.
pub fn use_app_group_ws(token: ReadSignal<Option<String>>) -> Option<UseGroupMessageWs> {
    let ws = create_memo(move |_| {
        token.get().clone().map(|t| use_group_message_ws(t))
    });

    create_effect(move |_| {
        if let Some(ws) = ws.get() {
            let send_message = ws.send_message;
            if let WsStatus::Open = ws.status.get() {
                send_message.set(Some(WebSocketMessage::Request {
                    request_id: uuid::Uuid::new_v4().to_string(),
                    action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                }));
            }
        }
    });

    let ws_val = ws.get();
    if let Some(ref ws_ctx) = ws_val {
        provide_context(ws_ctx.clone());
    }
    ws_val
}
