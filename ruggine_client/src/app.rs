// group_ws_contexts removed: WebSocket per-group context now handled differently

use leptos::*;
use crate::router::AppRouter;
use crate::utils::{ThemeProvider, use_theme, Theme, storage::StorageService};
use crate::components::ToastProvider;
use crate::hooks::use_app_group_ws::use_app_group_ws;
use crate::context::auth_context::{provide_auth_context, use_auth_context};
use crate::context::unread_counts_context::provide_unread_counts_context;
use crate::types::WebSocketMessage;

#[component]
pub fn App() -> impl IntoView {
    provide_auth_context();
    provide_unread_counts_context();
    view! {
        <ThemeProvider>
            <ToastProvider>
                <AppContent />
            </ToastProvider>
        </ThemeProvider>
    }
}

#[component]
pub fn AppContent() -> impl IntoView {
    let theme_ctx = use_theme();
    view! {
        <main>
            <AppRouter />
        </main>
    }
}
