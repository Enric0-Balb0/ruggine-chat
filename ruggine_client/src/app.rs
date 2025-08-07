use leptos::*;
use crate::router::AppRouter;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main>
            <AppRouter />
        </main>
    }
}
