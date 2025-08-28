use leptos::*;
use crate::types::WebSocketMessage;
use crate::hooks::use_group_message_ws::{use_group_message_ws, UseGroupMessageWs};
use crate::types::message_ws::WsStatus;

/// Hook che gestisce la logica di lifecycle e join automatica del WebSocket dopo login.
pub fn use_app_group_ws(token: ReadSignal<Option<String>>) -> Option<UseGroupMessageWs> {
    let ws = create_memo(move |_| {
        token.get().clone().map(|t| use_group_message_ws(t))
    });
    // Log token presence and when a WS context is created
    create_effect(move |_| {
        match token.get() {
            Some(t) => leptos::logging::log!("[WS-APP] token present (len={}), creating/using WS memo", t.len()),
            None => leptos::logging::log!("[WS-APP] no token available - WS will not be created"),
        }
        if let Some(ws_ctx) = ws.get() {
            leptos::logging::log!("[WS-APP] use_group_message_ws created (status={:?})", ws_ctx.status.get());
            // If already open, send join request
            if let WsStatus::Open = ws_ctx.status.get() {
                ws_ctx.send_message.set(Some(WebSocketMessage::Request {
                    request_id: uuid::Uuid::new_v4().to_string(),
                    action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                }));
            }
        }
    });

    // Observe status changes and log them to help debugging connectivity
    create_effect(move |_| {
        if let Some(ws_ctx) = ws.get() {
            let status = ws_ctx.status.get();
            leptos::logging::log!("[WS-APP][STATUS] group WS status changed: {:?}", status);
        }
    });

    // Return the memoized value (do not provide context here; AppLayout will provide an Option)
    ws.get()
}
