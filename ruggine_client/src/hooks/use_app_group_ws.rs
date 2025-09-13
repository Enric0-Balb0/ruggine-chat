use leptos::*;
use gloo_timers;
use crate::types::WebSocketMessage;
use crate::hooks::use_group_message_ws::{use_group_message_ws, UseGroupMessageWs};
use crate::types::message_ws::WsStatus;
use crate::api::ws::global_ws;
use crate::config::endpoints::WebSocketEndpoints;
use crate::config::constants::AppConstants;

/// Hook che gestisce la logica di lifecycle e join automatica del WebSocket dopo login.
pub fn use_app_group_ws(token: ReadSignal<Option<String>>) -> ReadSignal<Option<UseGroupMessageWs>> {
    // Create signal to track the current ws context and recreate it when token changes
    let (ws_ctx, set_ws_ctx) = create_signal::<Option<UseGroupMessageWs>>(None);
    
    // Effect to manage ws_ctx creation/recreation based on token changes
    {
        let set_ws_ctx = set_ws_ctx.clone();
        create_effect(move |_| {
            let current_token = token.get();
            match current_token {
                Some(t) => {
                    // Token available, create new instance
                    let instance = use_group_message_ws(t);
                    // Reset tracking immediately on creation to clear any stale state
                    instance.reset_tracking.set(true);
                    set_ws_ctx.set(Some(instance));
                    leptos::logging::log!("[WS APP] Created new UseGroupMessageWs instance for token");
                },
                None => {
                    // No token, clear instance
                    set_ws_ctx.set(None);
                    leptos::logging::log!("[WS APP] Cleared UseGroupMessageWs instance (no token)");
                }
            }
        });
    }
    
    // Connect only when token goes from None -> Some (login event). We use a
    // separate spawn_local that tracks the token and triggers connect on the shared
    // service. This ensures the socket is opened only at login and not on page
    // refresh/navigation.
    {
        let token_prev = create_rw_signal::<Option<String>>(None);
        let (should_continue, set_should_continue) = create_signal(true);
        let ws_ctx_for_spawn = ws_ctx.clone();
        
    spawn_local(async move {
            while should_continue.get_untracked() {
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
                
                let previous = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    token_prev.get_untracked()
                })) {
                    Ok(val) => val,
                    Err(_) => {
                        leptos::logging::log!("[WS APP] Previous token signal disposed, stopping");
                        break;
                    }
                };
                
                // Debug logging to understand the state transitions
                leptos::logging::log!("[WS APP] Token transition check: previous={:?}, current={:?}", 
                    previous.as_ref().map(|t| &t[..8]), current.as_ref().map(|t| &t[..8]));
                
                // detect transition None -> Some (login)
                if previous.is_none() && current.is_some() {
                    if let Some(tok) = current.clone() {
                        leptos::logging::log!("[WS APP] Login detected! Creating/connecting WebSocket for token {}", &tok[..8]);
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
                        
                        leptos::logging::log!("[WS APP] Current WebSocket status: {:?}", status_value);
                        
                        if status_value != WsStatus::Open {
                            leptos::logging::log!("[WS APP] login detected, connecting shared svc (svc={:p})", svc.as_ptr());
                            // Reset global join tracking before connecting
                            global_ws::reset_join_tracking(&tok);
                            svc.borrow_mut().connect(&url);

                            // After connecting, schedule a join message using the current ws_ctx instance.
                            let ws_ctx_clone_for_join = ws_ctx_for_spawn.clone();
                            let tok_for_join = tok.clone();
                            spawn_local(async move {
                                leptos::logging::log!("[WS APP] Scheduling join message after connection...");
                                // Wait for ws_ctx instance to exist
                                for i in 0..10 {
                                    leptos::logging::log!("[WS APP] Attempt {} to get ws_ctx instance", i + 1);
                                    // Get current ws_ctx value
                                    let current_ws_instance = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        ws_ctx_clone_for_join.get_untracked()
                                    })) {
                                        Ok(val) => val,
                                        Err(_) => {
                                            leptos::logging::log!("[WS APP] ws_ctx signal disposed");
                                            break;
                                        }
                                    };
                                    
                                    if let Some(ws_instance) = current_ws_instance {
                                        leptos::logging::log!("[WS APP] Got ws_ctx instance, waiting for Open status...");
                                        // Wait until Open then send join
                                        for j in 0..10 {
                                            gloo_timers::future::TimeoutFuture::new(200).await;
                                            let status_check = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                                ws_instance.status.get_untracked()
                                            })) {
                                                Ok(val) => val,
                                                Err(_) => {
                                                    leptos::logging::log!("[WS APP] Status check failed on ws_instance, continuing...");
                                                    continue;
                                                }
                                            };
                                            leptos::logging::log!("[WS APP] Status check {}: {:?}", j + 1, status_check);
                                            if status_check == WsStatus::Open {
                                                let request_id = uuid::Uuid::new_v4().to_string();
                                                global_ws::set_join_request_id(&tok_for_join, request_id.clone());
                                                leptos::logging::log!("[WS APP] Sending join message with request_id: {}", request_id);
                                                ws_instance.send_message.set(Some(WebSocketMessage::Request {
                                                    request_id,
                                                    action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                                                }));
                                                leptos::logging::log!("[WS APP] Join message sent after connection (via ws_ctx instance)");
                                                return;
                                            }
                                        }
                                        leptos::logging::log!("[WS APP] Failed to send join: socket never opened after connection");
                                        break;
                                    }
                                    gloo_timers::future::TimeoutFuture::new(300).await;
                                }
                                leptos::logging::log!("[WS APP] Failed to get ws_ctx instance for join");
                            });
                        } else {
                            leptos::logging::log!("[WS APP] login detected, shared svc already open (svc={:p})", svc.as_ptr());
                            // If already open, send join using current ws_ctx instance
                            let current_ws_instance = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                ws_ctx_for_spawn.get_untracked()
                            })) {
                                Ok(val) => val,
                                Err(_) => None,
                            };
                            
                            if let Some(ws_instance) = current_ws_instance {
                                let request_id = uuid::Uuid::new_v4().to_string();
                                global_ws::set_join_request_id(&tok, request_id.clone());
                                leptos::logging::log!("[WS APP] Sending join with existing open socket, request_id: {}", request_id);
                                ws_instance.send_message.set(Some(WebSocketMessage::Request {
                                    request_id,
                                    action: crate::types::ClientAction::Groups(crate::types::GroupAction::Join {}),
                                }));
                                leptos::logging::log!("[WS APP] Join message sent (via ws_ctx instance)");
                            } else {
                                leptos::logging::log!("[WS APP] No ws_ctx instance available for sending join");
                            }
                        }
                    }
                }
                
                // Safely update previous token
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    token_prev.set(current.clone());
                })) {
                    Ok(_) => {},
                    Err(_) => {
                        leptos::logging::log!("[WS APP] Cannot set previous token, signal disposed");
                        break;
                    }
                }
                
                // Wait before checking again
                gloo_timers::future::TimeoutFuture::new(100).await;
            }
            leptos::logging::log!("[WS APP] Connection monitoring loop ended");
        });
        
        // Cleanup: stop the monitoring loop when component unmounts
        on_cleanup(move || {
            set_should_continue.set(false);
        });
    }

    ws_ctx
}
