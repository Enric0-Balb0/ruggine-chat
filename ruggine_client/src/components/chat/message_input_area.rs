use leptos::*;
use std::rc::Rc;
use crate::hooks::use_group_message_ws::UseGroupMessageWs;
use crate::api::services::message::MessageService;
use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::types::message::{TextMessageCreateRequest, Message};
use crate::config::constants::AppConstants;

#[component]
pub fn MessageInputArea(
    ws_ctx: Option<UseGroupMessageWs>,
    #[prop(optional)] group_id: Option<i32>,
    #[prop(optional)] on_message_sent: Option<Rc<dyn Fn(Message)>>,
) -> impl IntoView {
    let (message_text, set_message_text) = create_signal(String::new());

    let group_id = group_id.unwrap_or_default();

    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        set_message_text.set(value);
    };

    let message_text_signal = message_text.clone();
    let sending = create_rw_signal(false);

    // Extract send logic into an Rc-wrapped function so it can be called
    // from both the send button and the textarea key handler.
    let handle_send_fn: Rc<dyn Fn()> = {
        let on_message_sent = on_message_sent;
        let set_message_text = set_message_text.clone();
        Rc::new(move || {
            let message = message_text_signal.get().trim().to_string();
            if !message.is_empty() {
                let storage_service = StorageService::new();
                let token = storage_service.get_token().map(|t| t.token);
                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                http_client.set_auth_token(token);
                let message_service = MessageService::new(http_client, storage_service);
                let req = TextMessageCreateRequest {
                    content: message.clone(),
                    group_chat_id: group_id,
                };
                sending.set(true);
                let on_message_sent_cb = on_message_sent.clone();
                let set_message_text = set_message_text.clone();
                leptos::spawn_local(async move {
                    match message_service.create_message(&req).await {
                        Ok(new_msg) => {
                            if let Some(cb) = on_message_sent_cb {
                                cb(new_msg.clone());
                            }
                        }
                        Err(_e) => {
                            // Optionally handle error
                        }
                    }
                    sending.set(false);
                    set_message_text.set(String::new());
                });
            }
        })
    };



    // Clone ws_ctx for each closure to avoid move errors
    let _ws_ctx_for_class = ws_ctx.clone();
    // Clone handle_send_fn for use in multiple closures below
    let handle_send_for_key = handle_send_fn.clone();
    let handle_send_for_click = handle_send_fn.clone();
    view! {
        <div class="px-6 py-4 border-t border-gray-200 dark:border-border-dark bg-gray-50 dark:bg-gray-900/20">
            <div class="relative flex items-center bg-white dark:bg-surface-dark border border-gray-200 dark:border-border-dark rounded overflow-hidden min-h-[44px]">
                <textarea
                    class="flex-1 border-none outline-none focus:outline-none focus:ring-0 focus:border-0 \
                    px-4 pr-12 py-3 text-sm resize-none min-h-[20px] max-h-[100px] \
                    bg-transparent text-gray-900 dark:text-text-primary-dark \
                    placeholder-gray-400 dark:placeholder-gray-500 focus:shadow-none \
                    flex items-center"
                    style="resize: none;"
                    placeholder="Scrivi un messaggio..."
                    rows="1-8"
                    prop:value=move || message_text.get()
                    on:input=handle_input
            on:keydown=move |ev: web_sys::KeyboardEvent| {
                        // Send on Enter, allow Shift+Enter for newline
                        if ev.key() == "Enter" && !ev.shift_key() {
                            ev.prevent_default();
                (handle_send_for_key)();
                        }
                    }
                />
                <button
                    class=move || {
                        if message_text.get().trim().is_empty() || sending.get() {
                            "absolute right-2 top-1/2 -translate-y-1/2 w-8 h-8 rounded flex items-center justify-center text-sm bg-gray-200 dark:bg-gray-700 text-gray-400 dark:text-gray-500 cursor-not-allowed"
                        } else {
                            "absolute right-2 top-1/2 -translate-y-1/2 w-8 h-8 rounded flex items-center justify-center text-sm bg-brand-primary dark:bg-brand-primary-dark text-white hover:bg-brand-secondary-light dark:hover:bg-brand-secondary-dark cursor-pointer"
                        }
                    }
                    on:click=move |ev| {
                        // keep button click behavior (ignore the event)
                        let _ = ev;
                        (handle_send_for_click)();
                    }
                    disabled=move || message_text.get().trim().is_empty() || sending.get()
                >
                    "→"
                </button>
            </div>
        </div>
    }
}
