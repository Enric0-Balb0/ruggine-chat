use leptos::*;
use gloo_timers;
use crate::types::WebSocketMessage;
use crate::hooks::use_group_message_ws::{use_group_message_ws, UseGroupMessageWs};
use crate::types::message_ws::WsStatus;
use crate::api::ws::global_ws;
use crate::config::endpoints::WebSocketEndpoints;
use crate::config::constants::AppConstants;

/// Hook che gestisce la logica di lifecycle e join automatica del WebSocket dopo login.
pub fn use_app_group_ws(token: ReadSignal<Option<String>>) -> Option<UseGroupMessageWs> {
    // Get initial token value without tracking
    let initial_token = token.get_untracked();
    
    // Create WS context only if we have a token
    let ws_ctx = initial_token.map(|t| use_group_message_ws(t));
    
    // Connect only when token goes from None -> Some (login event). We use a
    // separate spawn_local that tracks the token and triggers connect on the shared
    // service. This ensures the socket is opened only at login and not on page
    // refresh/navigation.
    {
        let token_prev = create_rw_signal::<Option<String>>(None);
        spawn_local(async move {
            loop {
                // Safely get token values without tracking
                let current = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    token.get_untracked()
                })) {
                    Ok(val) => val,
                    Err(_) => {
                        // Signal was disposed, stop the loop
                        leptos::logging::log!("[WS APP] Token signal disposed, stopping connection monitoring");
                        break;
                    }
                };
                
                let previous = token_prev.get_untracked();
                
                // detect transition None -> Some (login)
                if previous.is_none() && current.is_some() {
                    if let Some(tok) = current.clone() {
                        // ensure the shared service exists and connect it
                        let (svc, status) = global_ws::get_or_create(&tok);
                        let url = WebSocketEndpoints::group_websocket_url(AppConstants::DEFAULT_SERVER_URL, &tok);
                        
                        // Safely check status without tracking
                        let status_value = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            status.get_untracked()
                        })) {
                            Ok(val) => val,
                            Err(_) => {
                                leptos::logging::log!("[WS APP] Status signal disposed, assuming closed");
                                WsStatus::Closed
                            }
                        };
                        
                        if status_value != WsStatus::Open {
                            leptos::logging::log!("[WS APP] login detected, connecting shared svc (svc={:p})", svc.as_ptr());
                            svc.borrow_mut().connect(&url);
                            
                            // After connecting, schedule a join message with safer approach
                            let tok_for_join = tok.clone();
                            spawn_local(async move {
                                // Wait for connection to establish
                                for _ in 0..10 {
                                    gloo_timers::future::TimeoutFuture::new(500).await;
                                    
                                    // Try to get fresh WS context - always succeeds since use_group_message_ws returns UseGroupMessageWs
                                    let fresh_ws = use_group_message_ws(tok_for_join.clone());
                                    let fresh_ws = use_group_message_ws(tok_for_join.clone());
                                    // Safely check status
                                    let status_check = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        fresh_ws.status.get_untracked()
                                    })) {
                                        Ok(val) => val,
                                        Err(_) => {
                                            leptos::logging::log!("[WS APP] Status check failed, continuing...");
                                            continue;
                                        }
                                    };
                                    
                                    if status_check == WsStatus::Open {
                                        fresh_ws.send_message.set(Some(WebSocketMessage::Request {
                                            request_id: uuid::Uuid::new_v4().to_string(),
                                            action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                                        }));
                                        leptos::logging::log!("[WS APP] Join message sent after connection");
                                        break;
                                    }
                                }
                            });
                        } else {
                            leptos::logging::log!("[WS APP] login detected, shared svc already open (svc={:p})", svc.as_ptr());
                            // If already open, send join immediately using fresh WS context
                            let fresh_ws = use_group_message_ws(tok.clone());
                            let fresh_ws = use_group_message_ws(tok.clone());
                            fresh_ws.send_message.set(Some(WebSocketMessage::Request {
                                request_id: uuid::Uuid::new_v4().to_string(),
                                action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                            }));
                            leptos::logging::log!("[WS APP] Join message sent (already connected)");
                        }
                    }
                }
                token_prev.set(current.clone());
                
                // Wait before checking again
                gloo_timers::future::TimeoutFuture::new(100).await;
            }
        });
    }

    ws_ctx
}
