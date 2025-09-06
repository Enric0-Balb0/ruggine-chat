use leptos::*;
use crate::types::WebSocketMessage;
use crate::hooks::use_group_message_ws::{use_group_message_ws, UseGroupMessageWs};
use crate::types::message_ws::WsStatus;
use crate::api::ws::global_ws;
use crate::config::endpoints::WebSocketEndpoints;
use crate::config::constants::AppConstants;

/// Hook che gestisce la logica di lifecycle e join automatica del WebSocket dopo login.
pub fn use_app_group_ws(token: ReadSignal<Option<String>>) -> Option<UseGroupMessageWs> {
    let ws = create_memo(move |_| {
        token.get().clone().map(|t| use_group_message_ws(t))
    });
    // When the ws context appears or its status becomes Open, send a Join
    create_effect(move |_| {
        if let Some(ws_ctx) = ws.get() {
            // read the status to subscribe to changes
            match ws_ctx.status.get() {
                WsStatus::Open => {
                    // send join when it becomes open
                    ws_ctx.send_message.set(Some(WebSocketMessage::Request {
                        request_id: uuid::Uuid::new_v4().to_string(),
                        action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                    }));
                }
                _ => {}
            }
        }
    });

    // Connect only when token goes from None -> Some (login event). We use a
    // separate effect that tracks the token and triggers connect on the shared
    // service. This ensures the socket is opened only at login and not on page
    // refresh/navigation.
    {
        let token_prev = create_rw_signal::<Option<String>>(None);
        create_effect(move |_| {
            let current = token.get();
            let previous = token_prev.get_untracked();
            // detect transition None -> Some (login)
            if previous.is_none() && current.is_some() {
                if let Some(tok) = current.clone() {
                    // ensure the shared service exists and connect it
                    let (svc, status) = global_ws::get_or_create(&tok);
                    let url = WebSocketEndpoints::group_websocket_url(AppConstants::DEFAULT_SERVER_URL, &tok);
                    if status.get_untracked() != WsStatus::Open {
                        leptos::logging::log!("[WS APP] login detected, connecting shared svc (svc={:p})", svc.as_ptr());
                        svc.borrow_mut().connect(&url);
                    } else {
                        leptos::logging::log!("[WS APP] login detected, shared svc already open (svc={:p})", svc.as_ptr());
                    }
                    // Do not send Join here; the other effect observes the status
                    // and will send Join when the socket becomes Open.
                }
            }
            token_prev.set(current.clone());
        });
    }

    // Observe status changes and log them to help debugging connectivity
    create_effect(move |_| {
        if let Some(ws_ctx) = ws.get() {
            let _status = ws_ctx.status.get();
                
        }
    });

    // Return the memoized value without tracking (reading outside a reactive context intentionally)
    ws.get_untracked()
}
