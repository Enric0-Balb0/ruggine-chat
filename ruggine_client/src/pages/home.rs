use leptos::*;
use crate::components::{AppNavbar, Sidebar, CreateGroupModal};
use crate::types::group::GroupChatCreateRequest;
use crate::api::facade::RuggineApiClient;

#[component]
pub fn HomePage() -> impl IntoView {
    // API client instance with authentication from storage
    let api_client_instance = RuggineApiClient::new();
    
    // Recupera il token dal storage se esiste
    if let Some(token_response) = api_client_instance.storage_service.get_token() {
        api_client_instance.http_client.set_auth_token(Some(token_response.token));
    }
    
    let api_client = create_rw_signal(api_client_instance);
    
    // Modal state - gestito a livello della HomePage
    let (is_modal_open, set_is_modal_open) = create_signal(false);

    // Loading state for group creation
    let (is_creating_group, set_is_creating_group) = create_signal(false);

    // Handle create group button click from sidebar
    let handle_create_group_click = move |_| {
        set_is_modal_open.set(true);
    };

    // Handle modal close
    let handle_modal_close = move |_| {
        set_is_modal_open.set(false);
    };

    // Handle group creation
    let handle_group_create = move |create_request: GroupChatCreateRequest| {
        let api_client = api_client.get();
        let set_is_creating_group = set_is_creating_group;
        let set_is_modal_open = set_is_modal_open;

        spawn_local(async move {
            set_is_creating_group.set(true);
            
            match api_client.group_service.create_group(create_request).await {
                Ok(_group) => {
                    // TODO: Aggiornare la lista dei gruppi nella sidebar
                    // TODO: Reindirizzare al nuovo gruppo creato
                    set_is_modal_open.set(false);
                }
                Err(_error) => {
                    // TODO: Mostrare errore all'utente (toast, modal, etc.)
                    // For now, keep the modal open so user can retry
                }
            }
            
            set_is_creating_group.set(false);
        });
    };

    view! {
        <>
            <div class="h-screen w-screen overflow-hidden bg-white dark:bg-bg-main-dark flex">
                // Left Sidebar - Occupa tutto il lato sinistro dall'alto in basso
                <Sidebar on_create_group_click=handle_create_group_click />

                // Main Content Area - Include navbar + content
                <div class="flex-1 flex flex-col">
                    // Top Header/Navbar - Solo nella parte destra
                    <AppNavbar />

                    // Main Content Area
                    <div class="flex-1 bg-white dark:bg-bg-main-dark flex items-center justify-center">
                        <div class="text-center max-w-md">
                            <div class="w-16 h-16 bg-brand-primary-light rounded-full flex items-center justify-center mx-auto mb-4">
                                <span class="text-white text-xl font-bold">"💬"</span>
                            </div>
                            <h1 class="text-2xl font-bold text-text-primary dark:text-text-primary-dark mb-4">
                                "Benvenuto in Ruggine"
                            </h1>
                            <p class="text-text-secondary dark:text-text-secondary-dark mb-6">
                                "Seleziona un team dalla sidebar per iniziare una conversazione, oppure crea un nuovo team."
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            // Create Group Modal - renderizzato a livello HomePage per evitare problemi di posizionamento
            <CreateGroupModal 
                is_open=is_modal_open
                on_close=handle_modal_close
                on_create=handle_group_create
                is_loading=is_creating_group
            />
        </>
    }
}
