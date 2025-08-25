use leptos::*;
use crate::types::WebSocketMessage;
use crate::hooks::use_group_message_ws::{use_group_message_ws, WsStatus, UseGroupMessageWs};

/// Hook che gestisce la logica di lifecycle e join automatica del WebSocket dopo login.
pub fn use_app_group_ws(token: ReadSignal<Option<String>>) -> Option<UseGroupMessageWs> {
    let ws = create_memo(move |_| {
        token.get().clone().map(|t| use_group_message_ws(t))
    });

    create_effect(move |_| {
        if let Some(ws) = ws.get() {
            logging::log!("[WS] Stato connessione: {:?}", ws.status.get());
        } else {
            logging::log!("[WS] Nessuna connessione WebSocket attiva (token mancante o non autenticato)");
        }
    });

    create_effect(move |_| {
        if let Some(ws) = ws.get() {
            let send_message = ws.send_message;
            if let WsStatus::Connecting = ws.status.get() {
                // Non ancora connesso
            } else if let WsStatus::Open = ws.status.get() {
                logging::log!("[WS] Invio join ai gruppi...");
                send_message.set(Some(WebSocketMessage::Request {
                    request_id: uuid::Uuid::new_v4().to_string(),
                    action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                }));
            }
        }
    });

    ws.get()
}
