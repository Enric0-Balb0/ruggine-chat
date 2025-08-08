use leptos::*;
use crate::services::auth_service::AuthService;
use crate::services::storage_service::StorageService;
use crate::http::client::ApiClient;
use crate::config::constants::AppConstants;

#[component]
pub fn HomePage() -> impl IntoView {
    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    );

    let user_profile = auth_service.get_current_user();

    view! {
        <div class="h-screen bg-[#f3f2f1] flex overflow-hidden">
            // Left Sidebar (Future: teams, groups list)
            <aside class="w-70 bg-[#f8f8f8] border-r border-[#e1dfdd] flex flex-col">
                <div class="p-4 border-b border-[#e1dfdd]">
                    <h2 class="font-semibold text-[#323130] mb-2">"Team"</h2>
                    <div class="text-sm text-[#605e5c]">
                        "Le tue conversazioni"
                    </div>
                </div>
                
                <div class="flex-1 p-4">
                    <div class="space-y-2">
                        <div class="p-3 bg-white rounded border border-[#e1dfdd] hover:bg-[#f3f2f1] cursor-pointer transition-colors">
                            <div class="font-medium text-[#323130] text-sm">"# Generale"</div>
                            <div class="text-xs text-[#605e5c]">"Chat di gruppo principale"</div>
                        </div>
                        
                        <div class="p-3 rounded hover:bg-[#f3f2f1] cursor-pointer transition-colors">
                            <div class="font-medium text-[#323130] text-sm">"# Sviluppo"</div>
                            <div class="text-xs text-[#605e5c]">"Discussioni tecniche"</div>
                        </div>
                        
                        <div class="p-3 rounded hover:bg-[#f3f2f1] cursor-pointer transition-colors">
                            <div class="font-medium text-[#323130] text-sm">"# Marketing"</div>
                            <div class="text-xs text-[#605e5c]">"Strategie e campagne"</div>
                        </div>
                    </div>
                    
                    <button class="mt-4 flex items-center text-[#6264a7] text-sm hover:bg-[#f3f2f1] w-full p-2 rounded transition-colors">
                        <span class="mr-2">"+"</span>
                        "Aggiungi team"
                    </button>
                </div>
                
                // Bottom status section
                <div class="p-4 border-t border-[#e1dfdd]">
                    <div class="bg-white rounded p-3 border border-[#e1dfdd]">
                        <div class="flex items-center gap-3">
                            <div class="w-3 h-3 bg-[#107c10] rounded-full"></div>
                            <div>
                                <div class="text-sm font-medium text-[#323130]">
                                    {move || user_profile.clone().map_or("Utente".to_string(), |u| u.email)}
                                </div>
                                <div class="text-xs text-[#605e5c]">"Online"</div>
                            </div>
                        </div>
                    </div>
                </div>
            </aside>

            // Main Content Area
            <main class="flex-1 flex flex-col">
                <div class="flex-1 flex items-center justify-center">
                    <div class="text-center max-w-md">
                        <div class="w-16 h-16 bg-[#6264a7] rounded-full flex items-center justify-center mx-auto mb-4">
                            <span class="text-white text-xl font-bold">"💬"</span>
                        </div>
                        <h1 class="text-2xl font-bold text-[#323130] mb-4">
                            "Benvenuto nella Dashboard"
                        </h1>
                        <p class="text-[#605e5c] mb-6">
                            "Seleziona una chat dalla sidebar per iniziare una conversazione, oppure crea un nuovo team."
                        </p>
                        
                        <div class="space-x-3">
                            <button class="bg-[#6264a7] hover:bg-[#464775] text-white px-4 py-2 rounded transition-colors">
                                "Nuova Chat"
                            </button>
                            <button class="bg-white hover:bg-[#f3f2f1] text-[#323130] px-4 py-2 rounded border border-[#e1dfdd] transition-colors">
                                "Invita Membri"
                            </button>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    }
}


#[component]
fn LogoutButton() -> impl IntoView {
    let logout_action = create_action(|_: &()| async move {
        let auth_service = AuthService::new(
            ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
            StorageService::new(),
        );
        
        match auth_service.logout().await {
            Ok(_) => {
                // Ricarica la pagina per attivare il redirect al login
                web_sys::window()
                    .unwrap()
                    .location()
                    .reload()
                    .unwrap();
            }
            Err(e) => {
                leptos::logging::error!("Errore durante il logout: {:?}", e);
            }
        }
    });

    view! {
        <button
            on:click=move |_| logout_action.dispatch(())
            class="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded transition duration-200"
            disabled=move || logout_action.pending().get()
        >
            {move || if logout_action.pending().get() {
                "Disconnessione..."
            } else {
                "Logout"
            }}
        </button>
    }
}
