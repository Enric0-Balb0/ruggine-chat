use leptos::*;
use wasm_bindgen::prelude::*;

/// Theme management service
#[derive(Clone, Debug, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => Theme::Dark,
            _ => Theme::Light,
        }
    }
}

/// Theme service for managing light/dark theme
#[derive(Clone)]
pub struct ThemeService;

impl ThemeService {
    pub fn new() -> Self {
        Self
    }

    /// Get current theme from localStorage or system preference
    pub fn get_theme(&self) -> Theme {
        // Try to get from localStorage first
        if let Some(stored_theme) = self.get_stored_theme() {
            return Theme::from_str(&stored_theme);
        }
        
        // Fallback to system preference
        self.get_system_theme()
    }

    /// Set theme and persist to localStorage
    pub fn set_theme(&self, theme: Theme) {
        // Store in localStorage
        self.store_theme(&theme);
        
        // Apply to DOM
        self.apply_theme(&theme);
    }

    /// Toggle between light and dark theme
    pub fn toggle_theme(&self) -> Theme {
        let current = self.get_theme();
        let new_theme = match current {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
        
        self.set_theme(new_theme.clone());
        new_theme
    }

    /// Initialize theme on app startup
    pub fn initialize(&self) {
        let theme = self.get_theme();
        self.apply_theme(&theme);
    }

    // Private methods
    #[cfg(target_arch = "wasm32")]
    fn get_stored_theme(&self) -> Option<String> {
        use web_sys::window;
        let storage = window()?.local_storage().ok()??;
        storage.get_item("ruggine-theme").ok()?
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_stored_theme(&self) -> Option<String> {
        None // In tests, return None
    }

    #[cfg(target_arch = "wasm32")]
    fn store_theme(&self, theme: &Theme) {
        use web_sys::window;
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item("ruggine-theme", theme.as_str());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn store_theme(&self, _theme: &Theme) {
        // In tests, no-op
    }

    #[cfg(target_arch = "wasm32")]
    fn get_system_theme(&self) -> Theme {
        // Per semplicità, ritorniamo il tema chiaro come default
        // In futuro si può implementare il rilevamento tramite CSS media query
        Theme::Light
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_system_theme(&self) -> Theme {
        Theme::Light // In tests, default to light
    }

    #[cfg(target_arch = "wasm32")]
    fn apply_theme(&self, theme: &Theme) {
        use web_sys::{window, Element};
        
        if let Some(document) = window().and_then(|w| w.document()) {
            if let Some(html_element) = document.document_element() {
                let class_list = html_element.class_list();
                
                // Remove existing theme classes
                let _ = class_list.remove_1("light");
                let _ = class_list.remove_1("dark");
                
                // Add new theme class
                let _ = class_list.add_1(theme.as_str());
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_theme(&self, _theme: &Theme) {
        // In tests, no-op
    }
}

impl Default for ThemeService {
    fn default() -> Self {
        Self::new()
    }
}

/// Leptos context for theme management
#[derive(Clone)]
pub struct ThemeContext {
    pub theme: ReadSignal<Theme>,
    pub set_theme: WriteSignal<Theme>,
    pub theme_service: ThemeService,
}

impl ThemeContext {
    pub fn new() -> Self {
        let theme_service = ThemeService::new();
        let initial_theme = theme_service.get_theme();
        
        let (theme, set_theme) = create_signal(initial_theme);
        
        // Initialize theme on creation
        theme_service.initialize();
        
        Self {
            theme,
            set_theme,
            theme_service,
        }
    }
    
    pub fn toggle(&self) {
        let new_theme = self.theme_service.toggle_theme();
        self.set_theme.set(new_theme);
    }
}

/// Hook to use theme context
pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>()
        .expect("ThemeContext must be provided")
}

/// Theme provider component that provides theme context to children
#[component]
pub fn ThemeProvider(children: Children) -> impl IntoView {
    let theme_ctx = ThemeContext::new();
    
    provide_context(theme_ctx);
    
    children()
}
