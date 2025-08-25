
use leptos::*;
use crate::router::AppRouter;
use crate::utils::{ThemeProvider, use_theme, Theme, storage::StorageService};
use crate::components::ToastProvider;
use crate::hooks::use_app_group_ws::use_app_group_ws;
use crate::context::auth_context::{provide_auth_context, use_auth_context};
use crate::types::WebSocketMessage;

#[component]
pub fn App() -> impl IntoView {
    // Fornisci il context di autenticazione globale
    provide_auth_context();
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
    let auth_ctx = use_auth_context();
    // Gestione WebSocket centralizzata tramite hook dedicato
    let _ws = use_app_group_ws(auth_ctx.token.read_only());

    view! {
        <main>
            <AppRouter />
        </main>
    }
}
