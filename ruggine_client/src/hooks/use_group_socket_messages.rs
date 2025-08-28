use leptos::*;
use crate::types::message::Message;
use crate::types::WebSocketMessage;
use crate::types::message_ws::{ServerEvent, GroupEvent};
use crate::hooks::use_group_message_ws::UseGroupMessageWs;

/// Hook that returns a reactive signal with all deduplicated socket messages, sorted by date.
pub fn use_group_socket_messages(
    group_id: i32,
    ws_ctx: Option<UseGroupMessageWs>,
    local_messages: ReadSignal<Vec<Message>>,
) -> ReadSignal<Vec<Message>> {
    let (all_messages, set_all_messages) = create_signal(Vec::new());

    // Note: unread counts are updated at the websocket hook level (use_group_message_ws).
    // This hook focuses on assembling deduplicated messages for the chat view.

    create_effect(move |_| {
        let current_user_id = crate::utils::storage::StorageService::new()
            .get_user_profile()
            .map(|u| u.id);
        let ws_msgs = ws_ctx.as_ref().map(|w| w.messages.get());
        let local_msgs = local_messages.get();
        let mut all_msgs = local_msgs;
        if let (Some(_ws_ctx), Some(ws_msgs)) = (ws_ctx.as_ref(), ws_msgs) {
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
        set_all_messages.set(deduped);
    // Effect updates all_messages for the chat view; kept minimal for production.
    });

    all_messages
}
