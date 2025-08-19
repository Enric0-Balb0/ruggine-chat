use leptos::*;
use crate::utils::{use_theme, Theme};
use crate::components::{LucideIcon, IconSize};

/// Theme slider toggle component for dropdown menus
#[component]
pub fn ThemeSlider() -> impl IntoView {
    let theme_ctx = use_theme();
    
    let toggle_theme = {
        let theme_ctx = theme_ctx.clone();
        move |_| {
            theme_ctx.toggle();
        }
    };

    let is_dark = {
        let theme = theme_ctx.theme;
        move || matches!(theme.get(), Theme::Dark)
    };

    view! {
        <div class="flex items-center justify-between w-full">
            <div class="flex items-center gap-2">
                {move || if is_dark() { 
                    view! { <LucideIcon name="moon" size=IconSize::MEDIUM /> }
                } else { 
                    view! { <LucideIcon name="sun" size=IconSize::MEDIUM /> }
                }}
                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                    "Tema Scuro"
                </span>
            </div>
            
            // Custom slider toggle with inline styles
            <button
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 cursor-pointer"
                style=move || {
                    if is_dark() {
                        "background-color: #2563eb;" // blue-600
                    } else {
                        "background-color: #d1d5db;" // gray-300
                    }
                }
                on:click=toggle_theme
            >
                <span
                    class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200"
                    style=move || {
                        if is_dark() {
                            "transform: translateX(1.75rem);" // Move to right
                        } else {
                            "transform: translateX(0.125rem);" // Stay on left
                        }
                    }
                />
            </button>
        </div>
    }
}
