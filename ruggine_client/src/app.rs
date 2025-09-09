
use leptos::*;
use crate::router::AppRouter;
use crate::utils::{ThemeProvider, use_theme};
use crate::components::ToastProvider;
use crate::context::auth_context::provide_auth_context;
use crate::context::unread_counts_context::provide_unread_counts_context;
use crate::hooks::{RememberMeRefreshProvider, use_remember_me_init};

#[component]
pub fn App() -> impl IntoView {
    provide_auth_context();
    provide_unread_counts_context();
    
    view! {
        <ThemeProvider>
            <ToastProvider>
                <RememberMeRefreshProvider>
                    <AppContent />
                </RememberMeRefreshProvider>
            </ToastProvider>
        </ThemeProvider>
    }
}

#[component]
pub fn AppContent() -> impl IntoView {
    let _theme_ctx = use_theme();
    
    // Inizializza Remember Me all'avvio dell'app
    use_remember_me_init();

    // WebSocket context is created and provided by `AppLayout` for authenticated routes.

    view! {
        <main>
            <AppRouter />
        </main>
    }
}
