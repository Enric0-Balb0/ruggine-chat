use leptos::*;

/// Main app layout for authenticated users
#[component]
pub fn AppLayout(children: ChildrenFn) -> impl IntoView {
    view! {
        <main class="h-screen w-screen overflow-hidden">
            {children()}
        </main>
    }
}
