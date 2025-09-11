use leptos::*;
use gloo_timers;
use crate::types::message::Message;
use crate::types::WebSocketMessage;
use crate::types::message_ws::{ServerEvent, GroupEvent};
use crate::hooks::use_group_message_ws::UseGroupMessageWs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Hook that returns a reactive signal with all deduplicated socket messages, sorted by date.
pub fn use_group_socket_messages(
    group_id: i32,
    ws_ctx: Option<UseGroupMessageWs>,
    local_messages: ReadSignal<Vec<Message>>,
) -> ReadSignal<Vec<Message>> {
    let (all_messages, set_all_messages) = create_signal(Vec::new());

    // Note: unread counts are updated at the websocket hook level (use_group_message_ws).
    // This hook focuses on assembling deduplicated messages for the chat view.

    // Signal to control loop termination
    let (should_continue, set_should_continue) = create_signal(true);
    
    // Additional atomic flag for robust termination control
    let atomic_should_continue = Arc::new(AtomicBool::new(true));

    // Use spawn_local to avoid signal disposal panics
    {
        let ws_ctx_clone = ws_ctx.clone();
        let set_all_messages = set_all_messages.clone();
        let atomic_continue = atomic_should_continue.clone();
        spawn_local(async move {
            leptos::logging::log!("[WS SOCKET MSGS] Starting message polling loop for group {}", group_id);
            loop {
                // Check atomic flag first (most reliable)
                if !atomic_continue.load(Ordering::Relaxed) {
                    leptos::logging::log!("[WS SOCKET MSGS] Atomic flag set to false, breaking loop");
                    break;
                }
                
                // Check if we should continue - safely handle disposed signal
                let should_continue_val = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    should_continue.get_untracked()
                })) {
                    Ok(val) => val,
                    Err(_) => {
                        leptos::logging::log!("[WS SOCKET MSGS] should_continue signal disposed, breaking loop");
                        break;
                    }
                };
                
                if !should_continue_val {
                    leptos::logging::log!("[WS SOCKET MSGS] should_continue signal set to false, breaking loop");
                    break;
                }
                
                let _current_user_id = crate::utils::storage::StorageService::new()
                    .get_user_profile()
                    .map(|u| u.id);
                
                // Safely get WebSocket messages with proper error handling
                let ws_msgs = ws_ctx_clone.as_ref().and_then(|w| {
                    // Use catch_unwind to handle disposed signals
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        w.messages.get_untracked()
                    })) {
                        Ok(msgs) => Some(msgs),
                        Err(_) => {
                            leptos::logging::log!("[WS SOCKET MSGS] Messages signal disposed, stopping loop");
                            // Set atomic flag to stop the loop
                            atomic_continue.store(false, Ordering::Relaxed);
                            // Safely set should_continue to false with error handling
                            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                set_should_continue.set(false)
                            }));
                            None
                        }
                    }
                });
                
                // Safely get local messages
                let local_msgs = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    local_messages.get_untracked()
                })) {
                    Ok(msgs) => msgs,
                    Err(_) => {
                        leptos::logging::log!("[WS SOCKET MSGS] Local messages signal disposed, stopping loop");
                        // Safely set should_continue to false with error handling
                        if let Err(_) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            set_should_continue.set(false)
                        })) {
                            leptos::logging::log!("[WS SOCKET MSGS] set_should_continue signal also disposed, breaking immediately");
                        }
                        break;
                    }
                };
                // Check if we should not continue (safely handle disposed signal)
                if !atomic_continue.load(Ordering::Relaxed) {
                    leptos::logging::log!("[WS SOCKET MSGS] Atomic flag set to false during loop, breaking");
                    break;
                }
                
                let should_continue_val = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    should_continue.get_untracked()
                })) {
                    Ok(val) => val,
                    Err(_) => {
                        leptos::logging::log!("[WS SOCKET MSGS] should_continue signal disposed in loop, breaking");
                        break;
                    }
                };
                
                if !should_continue_val {
                    leptos::logging::log!("[WS SOCKET MSGS] should_continue signal set to false during loop, breaking");
                    break;
                }
                
                let mut all_msgs = local_msgs;
                if let (Some(_ws_ctx), Some(ws_msgs)) = (ws_ctx_clone.as_ref(), ws_msgs) {
                    let mut new_msgs: Vec<Message> = vec![];
                    for ws_msg in ws_msgs.iter() {
                        if let WebSocketMessage::Event { event, .. } = ws_msg {
                            if let ServerEvent::Groups(GroupEvent::NewMessage { message_id, group_id: gid, sender_id, sender_username: _, content, sent_at }) = event {
                                if *gid == group_id {
                                    // Increment only if message is not from current user
                                    // Do NOT increment unread_counts here: the global WS hook
                                    // (`use_group_message_ws`) already handles unread count increments
                                    // to ensure badges update even when chat view is not mounted.
                                    new_msgs.push(Message {
                                        id: *message_id,
                                        content: content.clone(),
                                        sender_id: *sender_id,
                                        group_chat_id: *gid,
                                        sent_at: *sent_at,
                                    });
                                }
                            }
                        }
                    }
                    all_msgs.extend(new_msgs);
                }
                
                use std::collections::HashMap;
                let mut map = HashMap::new();
                for msg in all_msgs {
                    map.insert(msg.id, msg);
                }
                let mut deduped: Vec<_> = map.into_values().collect();
                deduped.sort_by_key(|m| m.sent_at);
                
                // Safely update the signal
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    set_all_messages.set(deduped);
                })) {
                    Ok(_) => {},
                    Err(_) => {
                        leptos::logging::log!("[WS SOCKET MSGS] Set signal disposed, breaking loop");
                        break;
                    }
                }
                
                // Wait before checking again
                gloo_timers::future::TimeoutFuture::new(100).await;
            }
            leptos::logging::log!("[WS SOCKET MSGS] Message polling loop ended for group {}", group_id);
        });
    }

    // Cleanup: stop the loop when the component unmounts
    let atomic_continue_cleanup = atomic_should_continue.clone();
    on_cleanup(move || {
        leptos::logging::log!("[WS SOCKET MSGS] Cleanup called for group {}", group_id);
        // Set atomic flag first (most reliable)
        atomic_continue_cleanup.store(false, Ordering::Relaxed);
        
        // Also try to set the signal (with error handling)
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            set_should_continue.set(false);
        }));
    });

    all_messages
}
