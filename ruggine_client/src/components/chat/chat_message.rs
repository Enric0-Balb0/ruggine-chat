use leptos::*;
use chrono::Local;
use crate::types::message::Message;
// status icons removed: UI should not display ticks for messages
use crate::components::ui::UserAvatar;
use gloo_timers::callback::Timeout;
use std::collections::HashSet;
use std::cell::RefCell;

thread_local! {
    static ANIMATED_MESSAGE_IDS: RefCell<HashSet<i32>> = RefCell::new(HashSet::new());
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    Sent,      // una spunta
    Delivered, // doppia spunta
}

#[component]
pub fn ChatMessage(
    message: Message,
    sender_username: String,
    #[prop(optional, default = MessageStatus::Sent)] _status: MessageStatus,
    #[prop(optional, default = false)] is_own: bool,
    // Per avatar: aggiungi opzionalmente nome/cognome se disponibili
    #[prop(optional, default = "".to_string())] sender_name: String,
    #[prop(optional, default = "".to_string())] sender_surname: String,
    // True se questo messaggio continua un gruppo dello stesso mittente (quindi niente margine top e niente avatar/nome se show_sender=false)
    #[prop(optional, default = false)] continued: bool,
    // Se mostrare avatar + username (solo il primo del gruppo)
    #[prop(optional, default = true)] show_sender: bool,
) -> impl IntoView {
    let time_str = message.sent_at.with_timezone(&Local).format("%H:%M").to_string();
    // Cloni separati per evitare move tra le diverse closure generate dal macro view!
    let avatar_username = sender_username.clone();
    let header_username = sender_username.clone();
    let compact_time = time_str.clone();
    let _header_time = time_str.clone();
    let _fallback_time = time_str.clone();
    // Evita warning se l'originale non viene più usato direttamente
    let _original_sender_username = sender_username;
    // entrance animation only the first time a given message id is mounted
    let already_animated = ANIMATED_MESSAGE_IDS.with(|set| set.borrow().contains(&message.id));
    let (entered, set_entered) = create_signal(already_animated);
    if !already_animated {
        // Delay leggermente più alto per assicurare che il primo frame (opacity 0, translate) venga dipinto
        // prima di passare allo stato finale e quindi la transizione sia percepibile.
        let msg_id = message.id;
        let (is_mounted, set_is_mounted) = create_signal(true);
        
        // Cleanup quando il componente viene smontato
        on_cleanup(move || {
            set_is_mounted.set(false);
        });
        
        Timeout::new(80, move || {
            // Solo aggiorna se il componente è ancora montato
            if is_mounted.get_untracked() {
                set_entered.set(true);
                ANIMATED_MESSAGE_IDS.with(|set| { set.borrow_mut().insert(msg_id); });
            }
        }).forget();
    }
    let bubble_classes = if is_own {
        "bg-blue-100 dark:bg-blue-900 text-right ml-auto border-blue-200 dark:border-blue-700 rounded-lg"
    } else {
        "bg-gray-100 dark:bg-gray-800 text-left mr-auto border-gray-200 dark:border-gray-700 rounded-lg"
    };
    // Remove the small bottom margin on the header/time when this message continues a group
    let header_margin = if continued { "" } else { "mb-1" };
    // Wrapper base flex row (reverse per messaggi propri) + top alignment.
    // Non usiamo più padding interno per compensare l'avatar; usiamo sempre uno spacer come fallback simmetrico.
    // Wrapper base flex row (reverse for own messages) + top alignment.
    // We use a small spacer when avatar is hidden to keep alignment symmetric.
    let base = if is_own { "flex w-full items-start flex-row-reverse" } else { "flex w-full items-start flex-row" };

    // Remove per-message margin logic - now handled entirely by CSS .chat-msg rules

    // determine time position classes depending on ownership
    let time_pos = if is_own { "left-2 text-left" } else { "right-2 text-right" };

    // Add extra lateral padding on the side where the timestamp will sit so
    // very short messages don't place the time directly under the content.
    let bubble_padding = if is_own { "pl-10 pr-3 pt-1.5 pb-6" } else { "pl-3 pr-10 pt-1.5 pb-6" };
    // small horizontal offset for own messages to shift the column slightly to the right
    // apply it to the root container (so avatar + bubble move together) and increase a bit
    let sent_offset = if is_own { "translate-x-3" } else { "" };

    view! {
        <div class=move || {
            // root classes: add a stable marker `chat-msg` and `continued` when applicable
            let mut classes = String::new();
            classes.push_str("chat-msg ");
            if continued { classes.push_str("continued "); }
            classes.push_str(base);
            // apply horizontal offset to the whole message row for own messages
            if !sent_offset.is_empty() { classes.push_str(" "); classes.push_str(sent_offset); }
            // spacing is now controlled entirely by CSS .chat-msg and .chat-msg.continued rules
            
            if entered.get() {
                format!("{} opacity-100 translate-y-0 transition-all duration-200 ease-out", classes)
            } else {
                format!("{} opacity-0 translate-y-2 transition-all duration-200 ease-out", classes)
            }
        }>
            <Show
                when=move || show_sender
                fallback=move || {
                    if !show_sender {
                        // Spacer simmetrico leggermente ridotto (w-9) per diminuire lo spazio rispetto all'avatar reale
                        if is_own {
                            view! { <div class="flex-shrink-0 ml-2 w-8"/> }
                        } else {
                            view! { <div class="flex-shrink-0 mr-2 w-8"/> }
                        }
                    } else {
                        view! { <div class="w-0 h-0"/> }
                    }
                }
            >
                { // Do not render avatar for own messages; show the spacer instead to keep alignment
                    if is_own {
                        if is_own {
                            view! { <div class="flex-shrink-0 ml-2 w-8"/> }
                        } else {
                            view! { <div class="flex-shrink-0 mr-2 w-8"/> }
                        }
                    } else {
                        view! { <div class="flex-shrink-0 mr-2"><UserAvatar name=sender_name.clone() surname=sender_surname.clone() username=avatar_username.clone() size="md" /></div> }
                    }
                }
            </Show>
            <div class=format!("{} {} border shadow-sm max-w-[75%] {} relative transition-transform transition-opacity duration-200 ease-out",
                bubble_classes,
                bubble_padding,
                if is_own {"text-right"} else {"text-left"}
            )>
                <Show
                    when=move || show_sender
                    fallback=move || {
                        let hm = header_margin;
                        view! { <div class=hm></div> }
                    }
                >
                    {let username_header = header_username.clone(); let hm = header_margin;
                        // Do not render username for messages sent by the current user; keep spacing consistent.
                        if is_own {
                            view! { <div class=hm></div> }
                        } else {
                            view! { <div class=format!("flex items-center gap-2 {}", hm)>
                                <span class="font-semibold text-sm text-blue-700 dark:text-blue-300">{username_header}</span>
                            </div> }
                        }
                    }
                </Show>
                <div class="text-base text-gray-800 dark:text-gray-100 whitespace-pre-line">{message.content.clone()}</div>
                // timestamp positioned inside the bubble corner
                <div class=move || format!("absolute bottom-1 {} text-[11px] text-gray-400 px-2 py-0.5 pointer-events-none", time_pos)>
                    {compact_time}
                </div>
                // status icons intentionally omitted
            </div>
        </div>
    }
}
