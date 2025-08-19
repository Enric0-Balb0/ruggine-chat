use leptos::*;
use std::collections::HashMap;
use super::toast::{ToastMessage, Toast};

// Global toast context
#[derive(Clone)]
pub struct ToastContext {
    pub toasts: RwSignal<HashMap<String, ToastMessage>>,
}

impl ToastContext {
    pub fn new() -> Self {
        Self {
            toasts: create_rw_signal(HashMap::new()),
        }
    }

    pub fn add_toast(&self, toast: ToastMessage) {
        self.toasts.update(|toasts| {
            toasts.insert(toast.id.clone(), toast);
        });
    }

    pub fn remove_toast(&self, id: &str) {
        self.toasts.update(|toasts| {
            toasts.remove(id);
        });
    }

    pub fn success(&self, message: &str) {
        self.add_toast(ToastMessage::success(message.to_string()));
    }

    pub fn error(&self, message: &str) {
        self.add_toast(ToastMessage::error(message.to_string()));
    }

    pub fn warning(&self, message: &str) {
        self.add_toast(ToastMessage::warning(message.to_string()));
    }

    pub fn info(&self, message: &str) {
        self.add_toast(ToastMessage::info(message.to_string()));
    }

    pub fn clear_all(&self) {
        self.toasts.update(|toasts| {
            toasts.clear();
        });
    }
}

// Context provider component
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let toast_context = ToastContext::new();
    
    provide_context(toast_context.clone());

    view! {
        {children()}
        <ToastContainer />
    }
}

// Container that renders all active toasts
#[component]
pub fn ToastContainer() -> impl IntoView {
    let toast_context = expect_context::<ToastContext>();

    view! {
        <div class="fixed inset-0 pointer-events-none z-50">
            <div class="absolute top-4 right-4 space-y-2">
                <For
                    each=move || {
                        toast_context.toasts.get().into_iter().collect::<Vec<_>>()
                    }
                    key=|(id, _)| id.clone()
                    children=move |(_id, toast)| {
                        let toast_context = toast_context.clone();
                        view! {
                            <div class="pointer-events-auto">
                                <Toast
                                    toast=toast
                                    on_close=move |removed_id: String| {
                                        toast_context.remove_toast(&removed_id);
                                    }
                                />
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

// Hook to use toast context
pub fn use_toast() -> ToastContext {
    expect_context::<ToastContext>()
}
