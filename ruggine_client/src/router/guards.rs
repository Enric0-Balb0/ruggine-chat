use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;
use crate::context::auth_context::use_auth_context;
use crate::utils::StorageService;

#[component]
pub fn AuthGuard(children: ChildrenFn) -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();
    let auth_ctx = use_auth_context();

    // Controlla se l'utente è autenticato basandosi sul context
    let is_authenticated = create_memo(move |_| {
        let has_token = auth_ctx.token.get().is_some();
        let has_profile = auth_ctx.user_profile.get().is_some();
        
        // Un utente è considerato autenticato se ha almeno un token
        // Il profilo potrebbe ancora essere in caricamento con Remember Me
        let authenticated = has_token;
        
        log::info!("AuthGuard::is_authenticated: has_token={}, has_profile={}, authenticated={}", 
            has_token, has_profile, authenticated);
        
        authenticated
    });

    // Gestisce la navigazione quando non autenticato
    create_effect(move |_| {
        let current_path = location.pathname.get();
        let authenticated = is_authenticated.get();
        
        // Se non autenticato e non siamo già sulla pagina di login, reindirizza
        if !authenticated && current_path != "/login" {
            log::info!("AuthGuard: Utente non autenticato, reindirizzo a /login");
            navigate("/login", Default::default());
        }
    });

    // Listen for storage changes per sincronizzare con modifiche esterne
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let storage_listener = {
                let auth_ctx = auth_ctx.clone();
                move |_event: web_sys::Event| {
                    // Ricarica i dati dal storage quando cambia
                    let storage = StorageService::new();
                    if let Some(token_response) = storage.get_token() {
                        auth_ctx.token.set(Some(token_response.token));
                    } else {
                        auth_ctx.token.set(None);
                    }
                    
                    if let Some(profile) = storage.get_user_profile() {
                        auth_ctx.user_profile.set(Some(profile));
                    } else {
                        auth_ctx.user_profile.set(None);
                    }
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
            fallback=move || {
                view! { 
                    <div class="fixed inset-0 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm flex items-center justify-center z-50">
                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 text-center max-w-sm mx-4">
                            <div class="animate-pulse rounded-full h-8 w-8 bg-brand-primary/20 mx-auto mb-4"></div>
                            <p class="text-gray-600 dark:text-gray-300 text-sm">"Reindirizzamento al login..."</p>
                        </div>
                    </div> 
                }
            }
        >
            {children()}
        </Show>
    }
}
