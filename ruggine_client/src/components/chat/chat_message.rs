use leptos::*;
use chrono::Local;
use crate::types::message::Message;
// status icons removed: UI should not display ticks for messages
use crate::components::ui::UserAvatar;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    Sent,      // una spunta
    Delivered, // doppia spunta
}

#[component]
pub fn ChatMessage(
    message: Message,
    sender_username: String,
    #[prop(optional, default = MessageStatus::Sent)] status: MessageStatus,
    #[prop(optional, default = false)] is_own: bool,
    // Per avatar: aggiungi opzionalmente nome/cognome se disponibili
    #[prop(optional, default = "".to_string())] sender_name: String,
    #[prop(optional, default = "".to_string())] sender_surname: String,
) -> impl IntoView {
    let time_str = message.sent_at.with_timezone(&Local).format("%H:%M").to_string();
    let bubble_classes = if is_own {
        "bg-blue-100 dark:bg-blue-900 text-right ml-auto border-blue-200 dark:border-blue-700 rounded-lg"
    } else {
        "bg-gray-100 dark:bg-gray-800 text-left mr-auto border-gray-200 dark:border-gray-700 rounded-lg"
    };
    view! {
        <div class=format!(
            "my-2 flex w-full {}",
            if is_own {"flex-row-reverse"} else {"flex-row"}
        )>
            <div class=if is_own {"flex-shrink-0 ml-2"} else {"flex-shrink-0 mr-2"}>
                <UserAvatar name=sender_name.clone() surname=sender_surname.clone() username=sender_username.clone() size="md" />
            </div>
            <div class=format!("px-4 py-2 border shadow-sm {} max-w-[75%] {}",
                bubble_classes,
                if is_own {"text-right"} else {"text-left"}
            )>
                <div class="flex items-center gap-2 mb-1">
                    <span class="font-semibold text-sm text-blue-700 dark:text-blue-300">{sender_username.clone()}</span>
                    <span class="text-xs text-gray-400">{time_str}</span>
                </div>
                <div class="text-base text-gray-800 dark:text-gray-100 whitespace-pre-line">{message.content.clone()}</div>
                // status icons intentionally omitted
            </div>
        </div>
    }
}
