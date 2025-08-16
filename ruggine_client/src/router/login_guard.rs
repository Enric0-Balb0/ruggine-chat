use leptos::*;
use leptos_router::*;
use crate::api::services::AuthService;
use crate::utils::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;

/// Guard per le pagine pubbliche (login, register)
/// Reindirizza alla home se l'utente è già autenticato
#[component]
pub fn PublicGuard(children: ChildrenFn) -> impl IntoView {
    let navigate = use_navigate();
    
    let (auth_service, _) = create_signal(AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    ));
    
    // Controlla se l'utente è autenticato
    let is_authenticated = create_memo(move |_| {
        let service = auth_service.get();
        let authenticated = service.is_authenticated();
        
        authenticated
    });

    // Se autenticato, reindirizza alla home
    create_effect(move |_| {
        if is_authenticated.get() {
            navigate("/", Default::default());
        }
    });

    view! {
        <Show
            when=move || {
                let authenticated = is_authenticated.get();
                !authenticated
            }
            fallback=|| view! { 
                <div class="min-h-screen flex items-center justify-center">
                    <div class="text-center">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-brand-primary mx-auto mb-4"></div>
                        <p class="text-brand-secondary-light">"Reindirizzamento alla dashboard..."</p>
                    </div>
                </div> 
            }
        >
            {children()}
        </Show>
    }
}
