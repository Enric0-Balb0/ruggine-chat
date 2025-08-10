use leptos::*;
use leptos_router::*;
use crate::services::auth_service::AuthService;
use crate::services::storage_service::StorageService;
use crate::http::client::ApiClient;
use crate::config::constants::AppConstants;

/// Guard per le pagine pubbliche (login, register)
/// Reindirizza alla home se l'utente è già autenticato
#[component]
pub fn PublicGuard(children: ChildrenFn) -> impl IntoView {
    let navigate = use_navigate();
    
    // Controlla se l'utente è autenticato
    let is_authenticated = create_memo(move |_| {
        let auth_service = AuthService::new(
            ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
            StorageService::new(),
        );
        auth_service.is_authenticated()
    });

    // Se autenticato, reindirizza alla home
    create_effect(move |_| {
        if is_authenticated.get() {
            navigate("/", Default::default());
        }
    });

    view! {
        <Show
            when=move || !is_authenticated.get()
            fallback=|| view! { 
                <div class="min-h-screen flex items-center justify-center">
                    <div class="text-center">
                        <p class="text-brand-secondary-light">"Reindirizzamento alla dashboard..."</p>
                    </div>
                </div> 
            }
        >
            {children()}
        </Show>
    }
}
