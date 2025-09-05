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

    // Stati separati per gestire meglio il flusso
    let (auth_check_counter, set_auth_check_counter) = create_signal(0u32);
    let (initial_check_done, set_initial_check_done) = create_signal(false);
    let (auth_service, _) = create_signal(AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    ));
    
    // Forza il primo controllo all'avvio
    create_effect(move |_| {
        if !initial_check_done.get() {
            set_auth_check_counter.update(|n| *n += 1);
            set_initial_check_done.set(true);
        }
    });
    
    let is_authenticated = create_memo(move |_| {
        // Triggera il controllo ogni volta che il counter cambia
        let _counter = auth_check_counter.get();
        let service = auth_service.get();
        let _current_path = location.pathname.get();
        
        // Check if authenticated with valid token
        let authenticated = service.is_authenticated();
        
        authenticated
    });

    // Gestisce il refresh del token separatamente
    create_effect(move |_| {
        let service = auth_service.get();
        let authenticated = is_authenticated.get();
        
        if authenticated && service.needs_token_refresh() {
            let service_clone = service.clone();
            let set_auth_check_counter_clone = set_auth_check_counter.clone();
            
            spawn_local(async move {
                match service_clone.refresh_token().await {
                    Ok(_) => {
                        set_auth_check_counter_clone.update(|n| *n += 1);
                    }
                    Err(_) => {
                        let _ = service_clone.clear_session();
                        set_auth_check_counter_clone.update(|n| *n += 1);
                    }
                }
            });
        }
    });

    // Gestisce la navigazione
    create_effect(move |_| {
        let current_path = location.pathname.get();
        let authenticated = is_authenticated.get();
        let initial_done = initial_check_done.get();
        
        // Solo dopo che il controllo iniziale è stato fatto
        if initial_done {
            // Evita loop di navigazione
            if !authenticated && current_path != "/login" {
                navigate("/login", Default::default());
            }
        }
    });

    // Listen for storage changes to update auth state immediately
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let storage_listener = {
                let set_auth_check_counter = set_auth_check_counter.clone();
                move |_event: web_sys::Event| {
                    set_auth_check_counter.update(|n| *n += 1);
                }
            };
            
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(storage_listener) as Box<dyn FnMut(_)>);
            let _ = window.add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref());
            closure.forget(); // Keep the closure alive
        }
    });

    view! {
        <Show
            when=move || {
                let initial_done = initial_check_done.get();
                let authenticated = is_authenticated.get();
                
                initial_done && authenticated
            }
            fallback=move || {
                let initial_done = initial_check_done.get();
                let authenticated = is_authenticated.get();
                
                if !initial_done {
                    view! { 
                        <div class="min-h-screen flex items-center justify-center">
                            <div class="text-center">
                                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-brand-primary mx-auto mb-4"></div>
                                <p class="text-gray-600">"Controllo autenticazione..."</p>
                            </div>
                        </div> 
                    }
                } else if !authenticated {
                    // Navigate to login
                    let navigate = use_navigate();
                    navigate("/login", Default::default());
                    
                    view! { 
                        <div class="min-h-screen flex items-center justify-center">
                            <div class="text-center">
                                <p class="text-gray-600">"Reindirizzamento al login..."</p>
                            </div>
                        </div> 
                    }
                } else {
                    view! { <div></div> }
                }
            }
        >
            {children()}
        </Show>
    }
}
