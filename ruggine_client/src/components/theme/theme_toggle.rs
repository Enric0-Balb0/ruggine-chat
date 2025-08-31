use leptos::*;
use crate::utils::{use_theme, Theme};
use crate::components::LucideIcon;

/// Theme toggle button component
#[component]
pub fn ThemeToggle() -> impl IntoView {
    let theme_ctx = use_theme();
    
    let toggle_theme = {
        let theme_ctx = theme_ctx.clone();
        move |_| {
            theme_ctx.toggle();
        }
    };

    let theme_signal = theme_ctx.theme;

    let tooltip = {
        let theme = theme_signal.clone();
        move || match theme.get() {
            Theme::Light => "Attiva tema scuro",
            Theme::Dark => "Attiva tema chiaro",
        }
    };

    view! {
        <button
            class="w-10 h-10 rounded-full bg-white bg-opacity-20 hover:bg-opacity-30 dark:bg-gray-800 dark:bg-opacity-50 dark:hover:bg-opacity-70 flex items-center justify-center transition-all duration-200 border border-white border-opacity-20"
            on:click=toggle_theme
            title=tooltip
        >
            {move || match theme_signal.get() {
                Theme::Light => view! { <LucideIcon name="moon" size=20 class="text-white" /> },
                Theme::Dark => view! { <LucideIcon name="sun" size=20 class="text-white" /> },
            }}
        </button>
    }
}
