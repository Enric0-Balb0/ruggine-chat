use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use crate::api::services::AuthService;
use crate::utils::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;

#[component]
pub fn AuthGuard(children: ChildrenFn) -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();

    // Crea un segnale per controllare lo stato di autenticazione
    let (auth_check, set_auth_check) = create_signal(0u32);
    
    let is_authenticated = create_memo(move |_| {
        // Triggera il controllo ogni volta che auth_check cambia
        auth_check.get();
        let auth_service = AuthService::new(
            ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
            StorageService::new(),
        );
        
        // Check if authenticated with valid token
        let authenticated = auth_service.is_authenticated();
        
        // If authenticated but token needs refresh, try to refresh it
        if authenticated && auth_service.needs_token_refresh() {
            leptos::logging::log!("Token needs refresh, attempting refresh...");
            // Spawn async task for token refresh
            spawn_local(async move {
                if let Err(e) = auth_service.refresh_token().await {
                    leptos::logging::warn!("Token refresh failed: {:?}", e);
                    // Clear storage on refresh failure
                    let _ = auth_service.clear_session();
                }
            });
        }
        
        authenticated
    });

    // Check authentication and redirect if needed
    create_effect(move |_| {
        let current_path = location.pathname.get();
        let authenticated = is_authenticated.get();
        
        if !authenticated && current_path != "/login" {
            navigate("/login", Default::default());
        } else if authenticated && current_path == "/login" {
            navigate("/", Default::default());
        }
    });

    // Listen for storage changes to update auth state immediately
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let storage_listener = {
                let set_auth_check = set_auth_check.clone();
                move |_event: web_sys::Event| {
                    set_auth_check.update(|n| *n += 1);
                }
            };
            
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(storage_listener) as Box<dyn FnMut(_)>);
            let _ = window.add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref());
            closure.forget(); // Keep the closure alive
        }
    });

    view! {
        <Show
            when=move || is_authenticated.get()
            fallback=|| view! { <div class="min-h-screen flex items-center justify-center">
                <div class="text-center">
                    <p class="text-gray-600">"Reindirizzamento al login..."</p>
                </div>
            </div> }
        >
            {children()}
        </Show>
    }
}
