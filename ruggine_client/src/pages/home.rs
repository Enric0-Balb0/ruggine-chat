use leptos::*;
use crate::components::{AppNavbar, Sidebar, CreateGroupModal, ChatView, use_toast};
use crate::types::group::GroupChatCreateRequest;
use crate::api::facade::RuggineApiClient;
use crate::hooks::{provide_groups_context, use_groups_list};

#[component]
pub fn HomePage() -> impl IntoView {
    // Provide groups context for this page and its children
    let groups_context = provide_groups_context();
    
    // API client instance with authentication from storage
    let api_client_instance = RuggineApiClient::new();
    let toast = use_toast();
    
    // Use the groups hook from context
    let groups_hook = groups_context.groups_hook.clone();
    let groups_list = use_groups_list(&groups_hook);
    let active_group_id = groups_hook.active_group_id;

    // Start the groups refresh once when on the authenticated HomePage.
    // Continuous poller removed (authoritative refreshs happen on mount/open elsewhere).
    groups_context.groups_hook.refresh_groups.dispatch(());
    
    // Get current user for personalized welcome message
    let current_user = api_client_instance.auth_service.get_current_user();
    let welcome_title = if let Some(user) = &current_user {
        user.welcome_message()
    } else {
        "Benvenuto/a in Ruggine".to_string()
    };
    
    // Recupera il token dal storage se esiste e imposta l'autenticazione
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
    let handle_group_create = {
        let toast = toast.clone();
        let groups_hook = groups_hook.clone();
        move |create_request: GroupChatCreateRequest| {
            let api_client = api_client.get();
            let set_is_creating_group = set_is_creating_group;
            let set_is_modal_open = set_is_modal_open;
            let toast = toast.clone();
            let groups_hook = groups_hook.clone();
            let group_name = create_request.name.clone();

            spawn_local(async move {
                set_is_creating_group.set(true);
                
                match api_client.group_service.create_group(create_request).await {
                    Ok(_group) => {
                        toast.success(&format!("Gruppo '{}' creato con successo!", group_name));
                        
                        groups_hook.refresh_groups.dispatch(());
                        
                        // TODO: Reindirizzare al nuovo gruppo creato
                        set_is_modal_open.set(false);
                    }
                    Err(_error) => {
                        toast.error(&format!("Errore nella creazione del gruppo '{}'. Riprova.", group_name));
                    }
                }
                
                set_is_creating_group.set(false);
            });
        }
    };

    view! {
        <>
            <div class="h-screen w-screen bg-white dark:bg-bg-main-dark flex">
                // Left Sidebar - Occupa tutto il lato sinistro dall'alto in basso
                <Sidebar on_create_group_click=handle_create_group_click />

                // Main Content Area - Include navbar + content
                <div class="flex-1 flex flex-col min-h-0">
                    // Top Header/Navbar - Solo nella parte destra
                    <AppNavbar />

                    // Main Content Area
                    <div class="flex-1 bg-white dark:bg-bg-main-dark min-h-0 flex flex-col">
                        {move || {
                            if let Some(active_id) = active_group_id.get() {
                                // Find the selected group in the groups list
                                let groups = groups_list.get();
                                if let Some(selected_group) = groups.iter().find(|g| g.membership.group_chat_id == active_id) {
                                    view! {
                                        <ChatView group_data=selected_group.clone() />
                                    }.into_view()
                                } else {
                                    // Group is selected but not found in list (shouldn't happen)
                                    view! {
                                        <div class="flex items-center justify-center h-full">
                                            <div class="text-center">
                                                <p class="text-text-secondary dark:text-text-secondary-dark">
                                                    "Gruppo non trovato. Seleziona un altro gruppo."
                                                </p>
                                            </div>
                                        </div>
                                    }.into_view()
                                }
                            } else {
                                // No group selected - show welcome screen
                                view! {
                                    <div class="flex items-center justify-center h-full">
                                        <div class="text-center max-w-md">
                                            <div class="w-16 h-16 bg-brand-primary-light rounded-full flex items-center justify-center mx-auto mb-4">
                                                <span class="text-white text-xl font-bold">"💬"</span>
                                            </div>
                                            <h1 class="text-2xl font-bold text-text-primary dark:text-text-primary-dark mb-4">
                                                {welcome_title.clone()}
                                            </h1>
                                            <p class="text-text-secondary dark:text-text-secondary-dark mb-6">
                                                "Seleziona un team dalla sidebar per iniziare una conversazione, oppure crea un nuovo team."
                                            </p>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                        }}
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
