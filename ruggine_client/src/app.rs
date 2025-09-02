
use leptos::*;
use crate::router::AppRouter;
use crate::utils::{ThemeProvider, use_theme};
use crate::components::ToastProvider;
use crate::context::auth_context::provide_auth_context;
use crate::context::unread_counts_context::provide_unread_counts_context;

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
    use crate::context::auth_context::use_auth_context;

    // WebSocket context is created and provided by `AppLayout` for authenticated routes.

    view! {
        <main>
            <AppRouter />
        </main>
    }
}
