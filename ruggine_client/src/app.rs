use leptos::*;
use crate::router::AppRouter;
use crate::utils::{ThemeProvider, use_theme, Theme};
use crate::components::ToastProvider;

#[component]
pub fn App() -> impl IntoView {
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
    
    // Applica dinamicamente la classe del tema al body/html
    create_effect(move |_| {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(html_element) = document.document_element() {
                let class_name = match theme_ctx.theme.get() {
                    Theme::Dark => "dark",
                    Theme::Light => "",
                };
                let _ = html_element.set_class_name(class_name);
            }
        }
    });
    
    view! {
        <main>
            <AppRouter />
        </main>
    }
}
