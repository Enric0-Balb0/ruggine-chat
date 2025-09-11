use leptos::*;

/// Main app layout for authenticated users
#[component]
pub fn AppLayout(children: ChildrenFn) -> impl IntoView {
    // Il WebSocket context è ora fornito a livello di app principale, non qui
    // La gestione del cleanup è stata spostata nell'app principale

    view! {
        <main class="h-screen w-screen overflow-hidden">
            {children()}
        </main>
    }
}
