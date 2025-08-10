use leptos::*;
use crate::services::{use_theme, Theme};

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

    let icon = {
        let theme = theme_ctx.theme;
        move || match theme.get() {
            Theme::Light => "🌙", // Moon for switching to dark
            Theme::Dark => "☀️",  // Sun for switching to light
        }
    };

    let tooltip = {
        let theme = theme_ctx.theme;
        move || match theme.get() {
            Theme::Light => "Attiva tema scuro",
            Theme::Dark => "Attiva tema chiaro",
        }
    };

    view! {
        <button
            class="w-8 h-8 rounded-full bg-white bg-opacity-10 hover:bg-opacity-20 flex items-center justify-center transition-colors"
            on:click=toggle_theme
            title=tooltip
        >
            <span class="text-sm">
                {icon}
            </span>
        </button>
    }
}
