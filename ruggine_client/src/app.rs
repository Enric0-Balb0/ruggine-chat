use leptos::*;
use crate::pages::LoginPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="container">
            <LoginPage />
        </main>
    }
}
