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

    create_effect(move |_| {
    // Force reactivity by always reading the signals
        let ws_msgs = ws_ctx.as_ref().map(|w| w.messages.get());
        let local_msgs = local_messages.get();
        let _ = &ws_msgs;
        let _ = &local_msgs;
        let mut all_msgs = local_msgs;
        if let (Some(_ws_ctx), Some(ws_msgs)) = (ws_ctx.as_ref(), ws_msgs) {
            let mut new_msgs: Vec<Message> = ws_msgs.iter().filter_map(|ws_msg| {
                if let WebSocketMessage::Event { event, .. } = ws_msg {
                    if let ServerEvent::Groups(GroupEvent::NewMessage { message_id, group_id: gid, sender_id, sender_username, content, sent_at }) = event {
                        if *gid == group_id {
                            Some(Message {
                                id: *message_id,
                                content: content.clone(),
                                sender_id: *sender_id,
                                group_chat_id: *gid,
                                sent_at: *sent_at,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect();
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
    });

    all_messages
}
