use leptos::*;

#[component]
pub fn MessageInputArea() -> impl IntoView {
    let (message_text, set_message_text) = create_signal(String::new());
    
    let handle_input = move |ev| {
        let value = event_target_value(&ev);
        set_message_text.set(value);
    };
    
    let handle_send = move |_| {
        let message = message_text.get().trim().to_string();
        if !message.is_empty() {
            // TODO: Implement message sending
            logging::log!("Sending: {}", message);
            set_message_text.set(String::new());
        }
    };
    
    let is_disabled = move || message_text.get().trim().is_empty();
    
    view! {
        <div class="px-6 py-4 border-t border-gray-200 dark:border-border-dark bg-gray-50 dark:bg-gray-900/20">
            <div class="relative bg-white dark:bg-surface-dark border border-gray-200 dark:border-border-dark rounded overflow-hidden">
                <textarea
                    class="w-full border-none outline-none focus:outline-none focus:ring-0 focus:border-0 
                    px-4 pr-12 py-3 text-sm resize-none min-h-[20px] max-h-[100px] 
                    bg-transparent text-gray-900 dark:text-text-primary-dark 
                    placeholder-gray-400 dark:placeholder-gray-500
                    focus:shadow-none"
                    placeholder="Scrivi un messaggio..."
                    rows="1-8"
                    prop:value=move || message_text.get()
                    on:input=handle_input
                />
                <button
                    class=move || {
                        if is_disabled() {
                            "absolute right-2 top-1/2 -translate-y-1/2 w-8 h-8 rounded flex items-center justify-center text-sm bg-gray-200 dark:bg-gray-700 text-gray-400 dark:text-gray-500 cursor-not-allowed"
                        } else {
                            "absolute right-2 top-1/2 -translate-y-1/2 w-8 h-8 rounded flex items-center justify-center text-sm bg-brand-primary dark:bg-brand-primary-dark text-white hover:bg-brand-secondary-light dark:hover:bg-brand-secondary-dark cursor-pointer"
                        }
                    }
                    on:click=handle_send
                    disabled=is_disabled
                >
                    "→"
                </button>
            </div>
        </div>
    }
}