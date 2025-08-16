use leptos::*;

/// Circle loader component with customizable size and colors
#[component]
pub fn CircleLoader(
    /// Size of the loader: "sm", "md", "lg", "xl"
    #[prop(default = "md".to_string())]
    size: String,
    /// Optional text to show below the loader
    #[prop(optional)]
    text: Option<String>,
    /// Color scheme: "primary", "secondary", "white"
    #[prop(default = "primary".to_string())]
    color: String,
) -> impl IntoView {
    // Size classes
    let size_class = match size.as_str() {
        "sm" => "w-4 h-4",
        "md" => "w-6 h-6", 
        "lg" => "w-8 h-8",
        "xl" => "w-12 h-12",
        _ => "w-6 h-6", // default to md
    };

    // Color classes
    let color_class = match color.as_str() {
        "primary" => "border-brand-primary-light",
        "secondary" => "border-text-secondary dark:border-text-secondary-dark",
        "white" => "border-white",
        _ => "border-brand-primary-light", // default to primary
    };

    // Text size based on loader size
    let text_size_class = match size.as_str() {
        "sm" => "text-xs",
        "md" => "text-sm",
        "lg" => "text-base", 
        "xl" => "text-lg",
        _ => "text-sm",
    };

    view! {
        <div class="flex flex-col items-center justify-center space-y-3">
            // Spinning circle with improved animation
            <div class={format!("loader-smooth rounded-full border-2 border-transparent border-t-current {}", size_class)}>
                <div class={format!("rounded-full border-2 border-transparent {}", color_class)}></div>
            </div>
            
            // Optional text with subtle pulse animation
            {text.map(|t| view! {
                <div class={format!("text-center font-medium text-text-secondary dark:text-text-secondary-dark loader-pulse {}", text_size_class)}>
                    {t}
                </div>
            })}
        </div>
    }
}

/// Simple inline spinner without text
#[component]
pub fn InlineSpinner(
    #[prop(default = "sm".to_string())]
    size: String,
    #[prop(default = "primary".to_string())] 
    color: String,
) -> impl IntoView {
    let size_class = match size.as_str() {
        "xs" => "w-3 h-3 border",
        "sm" => "w-4 h-4 border-2",
        "md" => "w-5 h-5 border-2",
        _ => "w-4 h-4 border-2",
    };

    let color_class = match color.as_str() {
        "primary" => "border-brand-primary-light border-t-transparent",
        "secondary" => "border-text-secondary border-t-transparent dark:border-text-secondary-dark",
        "white" => "border-white border-t-transparent",
        _ => "border-brand-primary-light border-t-transparent",
    };

    view! {
        <div class={format!("loader-smooth rounded-full {size_class} {color_class}")}></div>
    }
}

/// Button with integrated loader for async actions
#[component]
pub fn LoadingButton(
    /// Button text when not loading
    children: Children,
    /// Whether the button is in loading state
    #[prop(into)]
    loading: Signal<bool>,
    /// Optional loading text
    #[prop(optional)]
    loading_text: Option<String>,
    /// Button click handler
    #[prop(into)]
    on_click: Callback<ev::MouseEvent>,
    /// Button style class
    #[prop(default = "btn-primary".to_string())]
    class: String,
    /// Disabled state
    #[prop(default = false)]
    disabled: bool,
) -> impl IntoView {
    view! {
        <button 
            class={format!("{} relative", class)}
            disabled=move || loading.get() || disabled
            on:click=move |ev| on_click.call(ev)
        >
            // Content - hidden when loading
            <span class=move || if loading.get() { "opacity-0" } else { "opacity-100" }>
                {children()}
            </span>
            
            // Loading state
            {move || if loading.get() {
                view! {
                    <div class="absolute inset-0 flex items-center justify-center space-x-2">
                        <InlineSpinner size="xs".to_string() color="white".to_string() />
                        {loading_text.as_ref().map(|text| view! {
                            <span class="text-sm">{text}</span>
                        })}
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }}
        </button>
    }
}
