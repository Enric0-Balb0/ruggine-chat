use leptos::*;
use crate::components::{UserAvatar, ThemeToggle};
use crate::api::services::AuthService;
use crate::utils::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;

/// Main app navbar for authenticated users (based on UI mock)
#[component]
pub fn AppNavbar() -> impl IntoView {
    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    );

    // Recupera i dati dell'utente corrente
    let user_profile = auth_service.get_current_user();

    view! {
        <header class="h-16 bg-brand-primary dark:bg-brand-primary-dark text-white flex items-center justify-between pl-2 pr-6 border-b border-border dark:border-border-dark flex-shrink-0">
            <div class="flex items-center gap-4">
                <img 
                    src="public/logos/logo-full-white.png" 
                    alt="Ruggine" 
                    class="h-10 w-auto"
                />
            </div>

            <div class="flex items-center gap-3">
                <ThemeToggle />
                {match user_profile {
                    Some(user) => {
                        // Dividi il full_name in first_name e last_name
                        let name_parts: Vec<&str> = user.full_name.split_whitespace().collect();
                        let first_name = name_parts.first().unwrap_or(&"User").to_string();
                        let last_name = name_parts.get(1).unwrap_or(&"Default").to_string();
                        // Usa l'email come username temporaneo (manca username nel DTO)
                        let username = user.email.split('@').next().unwrap_or("user").to_string();
                        
                        view! {
                            <UserAvatar 
                                name=first_name 
                                surname=last_name 
                                username=username 
                                size="md" 
                            />
                        }.into_view()
                    },
                    None => view! {
                        <UserAvatar 
                            name="User".to_string() 
                            surname="Default".to_string() 
                            username="user".to_string() 
                            size="md" 
                        />
                    }.into_view(),
                }}
            </div>
        </header>
    }
}
