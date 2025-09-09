use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use gloo_timers::future::TimeoutFuture;
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
    
    // Forza il primo controllo all'avvio con un piccolo delay per permettere all'auto-login di completarsi
    create_effect(move |_| {
        if !initial_check_done.get() {
            let set_auth_check_counter = set_auth_check_counter.clone();
            let set_initial_check_done = set_initial_check_done.clone();
            let auth_service = auth_service.get();
            spawn_local(async move {
                // Controlla se Remember Me è attivo e potrebbe fare auto-login
                let storage = StorageService::new();
                if storage.is_remember_me_active() && !auth_service.is_authenticated() {
                    // Se Remember Me è attivo ma non siamo autenticati, aspetta massimo 2 secondi per l'auto-login
                    log::info!("AuthGuard: Remember Me attivo, aspetto auto-login...");
                    
                    // Controlla ogni 100ms per un massimo di 2 secondi se l'utente si è autenticato
                    let mut checks = 0;
                    while checks < 20 && !auth_service.is_authenticated() {
                        gloo_timers::future::TimeoutFuture::new(100).await;
                        checks += 1;
                    }
                    
                    if auth_service.is_authenticated() {
                        log::info!("AuthGuard: Auto-login completato!");
                    } else {
                        log::info!("AuthGuard: Auto-login timeout, procedo con controllo normale");
                    }
                } else {
                    // Altrimenti aspetta solo 100ms per evitare flickering
                    gloo_timers::future::TimeoutFuture::new(100).await;
                }
                set_auth_check_counter.update(|n| *n += 1);
                set_initial_check_done.set(true);
            });
        }
    });
    
    let is_authenticated = create_memo(move |_| {
        // Triggera il controllo ogni volta che il counter cambia
        let _counter = auth_check_counter.get();
        let service = auth_service.get();
        let _current_path = location.pathname.get();
        
        // Check if authenticated with valid token
        let authenticated = service.is_authenticated();
        
        // Debug logging
        let storage = StorageService::new();
        let has_token = storage.get_token().is_some();
        let has_profile = storage.get_user_profile().is_some();
        let remember_me_active = storage.is_remember_me_active();
        
        log::info!("AuthGuard::is_authenticated: risultato={}, has_token={}, has_profile={}, remember_me_active={}", 
            authenticated, has_token, has_profile, remember_me_active);
        
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
                    // Invece di una pagina bianca, mostra un overlay discreto
                    view! { 
                        <div class="fixed inset-0 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm flex items-center justify-center z-50">
                            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 text-center max-w-sm mx-4">
                                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-brand-primary mx-auto mb-4"></div>
                                <p class="text-gray-600 dark:text-gray-300 text-sm">"Verifica autenticazione..."</p>
                            </div>
                        </div> 
                    }
                } else if !authenticated {
                    // Navigate to login immediatamente
                    let navigate = use_navigate();
                    create_effect(move |_| {
                        navigate("/login", Default::default());
                    });
                    
                    view! { 
                        <div class="fixed inset-0 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm flex items-center justify-center z-50">
                            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 text-center max-w-sm mx-4">
                                <div class="animate-pulse rounded-full h-8 w-8 bg-brand-primary/20 mx-auto mb-4"></div>
                                <p class="text-gray-600 dark:text-gray-300 text-sm">"Reindirizzamento..."</p>
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
