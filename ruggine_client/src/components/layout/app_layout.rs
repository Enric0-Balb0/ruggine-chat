use leptos::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use crate::hooks::use_app_group_ws::use_app_group_ws;
use crate::context::auth_context::use_auth_context;

/// Main app layout for authenticated users
#[component]
pub fn AppLayout(children: ChildrenFn) -> impl IntoView {
    let auth_ctx = use_auth_context();
    let token = auth_ctx.token.read_only();
    let ws_ctx = use_app_group_ws(token);
    // app layout: ws_ctx presence handled
    // provide a clone so the local `ws_ctx` can still be used below
    provide_context(ws_ctx.clone());

    // Register a beforeunload handler so that when the user closes the tab/window
    // we attempt to synchronously close all shared websockets. This will send
    // the websocket close frames so the server can log leave/unsubscribe/close.
    if let Some(win) = web_sys::window() {
        // capture the leptos read-only token signal so closures can read the
        // current token at unload time and call disconnect_for_token(token)
        let token_for_closure = token.clone();
        // beforeunload: common earliest hook
        let on_before = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            if let Some(t) = token_for_closure.get() {
                crate::api::ws::global_ws::disconnect_for_token(&t);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = win.add_event_listener_with_callback("beforeunload", on_before.as_ref().unchecked_ref());
        on_before.forget();

        // pagehide: fired on some browsers/platforms when navigating away or closing
        let token_for_pagehide = token_for_closure.clone();
        let on_pagehide = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            if let Some(t) = token_for_pagehide.get() {
                crate::api::ws::global_ws::disconnect_for_token(&t);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = win.add_event_listener_with_callback("pagehide", on_pagehide.as_ref().unchecked_ref());
        on_pagehide.forget();

        // unload: fallback
        let token_for_unload = token_for_closure.clone();
        let on_unload = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            if let Some(t) = token_for_unload.get() {
                crate::api::ws::global_ws::disconnect_for_token(&t);
            }
        }) as Box<dyn FnMut(_)>);
        let _ = win.add_event_listener_with_callback("unload", on_unload.as_ref().unchecked_ref());
        on_unload.forget();
    }

    // Close socket on logout (token -> None) and on component cleanup
    {
        let token = token.clone();
        create_effect(move |_| {
            // when token becomes None, disconnect the shared service for previous token
            if token.get().is_none() {
                if let Some(prev_token) = auth_ctx.token.get() {
                    crate::api::ws::global_ws::disconnect_for_token(&prev_token);
                }
            }
        });
    }

    on_cleanup(move || {
        // attempt to disconnect all sockets when the app layout unmounts
        crate::api::ws::global_ws::disconnect_all();
    });

    // Server handles WebSocket disconnection on client unload; no client-side
    // beforeunload handler is needed anymore.

    view! {
        <main class="h-screen w-screen overflow-hidden">
            {children()}
        </main>
    }
}
