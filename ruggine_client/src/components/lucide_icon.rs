use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = lucide)]
    fn createIcons(config: &JsValue);
}

/// Lucide Icon component for displaying SVG icons
#[component]
pub fn LucideIcon(
    /// Icon name (e.g., "menu", "sun", "moon", "settings")
    name: &'static str,
    /// Optional CSS classes
    #[prop(optional)] class: &'static str,
    /// Icon size (default: 20)
    #[prop(default = 20)] size: u32,
) -> impl IntoView {
    let icon_class = if class.is_empty() {
        format!("lucide-icon-{}", name)
    } else {
        format!("lucide-icon-{} {}", name, class)
    };

    create_effect(move |_| {
        // Initialize Lucide icons after the DOM is updated
        request_animation_frame(move || {
            let config = js_sys::Object::new();
            js_sys::Reflect::set(&config, &"nameAttr".into(), &"data-lucide".into()).unwrap();
            createIcons(&config.into());
        });
    });

    view! {
        <i 
            data-lucide=name
            class=icon_class
            style=format!("width: {}px; height: {}px;", size, size)
        ></i>
    }
}

/// Common icon sizes as constants
pub mod IconSize {
    pub const SMALL: u32 = 16;
    pub const MEDIUM: u32 = 20;
    pub const LARGE: u32 = 24;
    pub const XLARGE: u32 = 32;
}
