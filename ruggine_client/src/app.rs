
use leptos::*;
use wasm_bindgen::JsCast;
use crate::router::AppRouter;
use crate::utils::{ThemeProvider, use_theme};
use crate::components::ToastProvider;
use crate::context::auth_context::{provide_auth_context, use_auth_context};
use crate::context::unread_counts_context::provide_unread_counts_context;
use crate::context::invitations_context::provide_invitations_context;
use crate::hooks::use_app_group_ws::use_app_group_ws;

#[component]
pub fn App() -> impl IntoView {
    provide_auth_context();
    provide_unread_counts_context();
    provide_invitations_context();
    
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
    let _theme_ctx = use_theme();
    let auth_ctx = use_auth_context();
    let token = auth_ctx.token.read_only();

    // Inizializza il WebSocket globalmente per tutta l'app (una sola volta)
    let ws_ctx = use_app_group_ws(token);
    provide_context(ws_ctx);

    // Populate invitations context once when we have a token so pending_count is correct
    create_effect(move |_| {
        if token.get().is_some() {
            leptos::logging::log!("[APP] token available, calling refresh_invitations()");
            spawn_local(async move {
                let _ = crate::context::invitations_context::refresh_invitations().await;
            });
        }
    });

    // Register a beforeunload handler so that when the user closes the tab/window
    // we attempt to synchronously close all shared websockets. This will send
    // the websocket close frames so the server can log leave/unsubscribe/close.
    if let Some(win) = web_sys::window() {
        // capture the leptos read-only token signal so closures can read the
        // current token at unload time and call disconnect_for_token(token)
        let token_for_closure = token.clone();
        // beforeunload: common earliest hook
        let on_before = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            if let Some(t) = token_for_closure.get() {
                crate::api::ws::global_ws::disconnect_for_token(&t);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = win.add_event_listener_with_callback("beforeunload", on_before.as_ref().unchecked_ref());
        on_before.forget();

        // pagehide: fired on some browsers/platforms when navigating away or closing
        let token_for_pagehide = token_for_closure.clone();
        let on_pagehide = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            if let Some(t) = token_for_pagehide.get() {
                crate::api::ws::global_ws::disconnect_for_token(&t);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = win.add_event_listener_with_callback("pagehide", on_pagehide.as_ref().unchecked_ref());
        on_pagehide.forget();

        // unload: fallback
        let token_for_unload = token_for_closure.clone();
        let on_unload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            if let Some(t) = token_for_unload.get() {
                crate::api::ws::global_ws::disconnect_for_token(&t);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = win.add_event_listener_with_callback("unload", on_unload.as_ref().unchecked_ref());
        on_unload.forget();
    }

    // Close socket on logout (token -> None)
    {
        let token = token.clone();
        let (prev_token, set_prev_token) = create_signal::<Option<String>>(None);
        
        create_effect(move |_| {
            let current_token = token.get();
            let previous_token = prev_token.get_untracked();
            
            // If token goes from Some -> None (logout), disconnect the previous token's socket
            if current_token.is_none() && previous_token.is_some() {
                if let Some(token_to_disconnect) = previous_token {
                    leptos::logging::log!("[APP CLEANUP] Disconnecting WebSocket for token on logout");
                    crate::api::ws::global_ws::disconnect_for_token(&token_to_disconnect);
                }
            }
            
            // Update previous token for next comparison
            set_prev_token.set(current_token.clone());
        });
    }

    on_cleanup(move || {
        // attempt to disconnect all sockets when the app unmounts
        crate::api::ws::global_ws::disconnect_all();
    });

    view! {
        <main>
            <AppRouter />
        </main>
    }
}
