use leptos::*;
use crate::hooks::use_app_group_ws::use_app_group_ws;
use crate::context::auth_context::use_auth_context;

/// Main app layout for authenticated users
#[component]
pub fn AppLayout(children: ChildrenFn) -> impl IntoView {
    let auth_ctx = use_auth_context();
    let token = auth_ctx.token.read_only();
    let ws_ctx = use_app_group_ws(token);
    provide_context(ws_ctx);

    view! {
        <main class="h-screen w-screen overflow-hidden">
            {children()}
        </main>
    }
}
