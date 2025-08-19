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

    let icon_name = {
        let theme = theme_ctx.theme;
        move || match theme.get() {
            Theme::Light => "moon", // Moon for switching to dark
            Theme::Dark => "sun",  // Sun for switching to light
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
            class="w-10 h-10 rounded-full bg-white bg-opacity-20 hover:bg-opacity-30 dark:bg-gray-800 dark:bg-opacity-50 dark:hover:bg-opacity-70 flex items-center justify-center transition-all duration-200 border border-white border-opacity-20"
            on:click=toggle_theme
            title=tooltip
        >
            <LucideIcon name=icon_name() size=20 class="text-white" />
        </button>
    }
}
