use leptos::*;
use crate::services::auth_service::AuthService;
use crate::services::storage_service::StorageService;
use crate::http::client::ApiClient;
use crate::config::constants::AppConstants;

/// Main app navbar for authenticated users (based on UI mock)
#[component]
pub fn AppNavbar() -> impl IntoView {
    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    );

    let handle_logout = {
        let auth_service = auth_service.clone();
        move |_| {
            let auth_service = auth_service.clone();
            spawn_local(async move {
                let _ = auth_service.logout().await;
                // Navigation will be handled by AuthGuard automatically
            });
        }
    };

    view! {
        <header class="bg-[#464775] text-white flex items-center justify-between px-6 py-3 border-b border-[#e1dfdd]">
            <div class="flex items-center gap-4">
                <img 
                    src="public/logos/nav-logo.png" 
                    alt="Ruggine   " 
                    class="h-8 w-auto"
                />
                <div>
                    <h1 class="text-base font-semibold">
                        "Ruggine Chat"
                    </h1>
                    <p class="text-sm opacity-90">
                        "Dashboard"
                    </p>
                </div>
            </div>

            <div class="flex items-center gap-3">
                <button class="bg-white bg-opacity-10 hover:bg-opacity-20 px-3 py-1.5 rounded text-xs transition-colors">
                    "Nuova Chat"
                </button>
                
                <button class="bg-white bg-opacity-10 hover:bg-opacity-20 px-3 py-1.5 rounded text-xs transition-colors">
                    "Impostazioni"
                </button>

                <div class="w-8 h-8 bg-[#6264a7] rounded-full flex items-center justify-center text-xs font-bold cursor-pointer">
                    "U"
                </div>

                <button 
                    on:click=handle_logout
                    class="bg-white bg-opacity-10 hover:bg-opacity-20 px-3 py-1.5 rounded text-xs transition-colors"
                >
                    "Logout"
                </button>
            </div>
        </header>
    }
}
