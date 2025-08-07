use leptos::*;
use crate::components::AppNavbar;

/// Main app layout for authenticated users
#[component]
pub fn AppLayout(children: ChildrenFn) -> impl IntoView {
    view! {
        <div class="h-screen overflow-hidden">
            <AppNavbar />
            
            <main class="h-[calc(100vh-4rem)] overflow-hidden">
                {children()}
            </main>
        </div>
    }
}
