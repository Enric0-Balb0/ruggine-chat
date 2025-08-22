use leptos::*;
use crate::components::{GroupItem, CreateGroupButton, ShowInvitesButton};
use crate::components::modals::ShowInvitesModal;
use crate::api::services::invitation::InvitationService;
use crate::types::Invitation;
use crate::hooks::{use_groups_context, use_groups_list, use_groups_loading, use_groups_error};

/// Sidebar component for the main app layout
#[component]
pub fn Sidebar(
    #[prop(into)] on_create_group_click: Callback<()>,
) -> impl IntoView {
    // Use groups context from provider
    let groups_context = use_groups_context();
    let groups_hook = groups_context.groups_hook;
    let groups_list = use_groups_list(&groups_hook);
    let is_loading = use_groups_loading(&groups_hook);
    let error = use_groups_error(&groups_hook);

    // Load groups on mount
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            if let Some(storage) = window.local_storage().ok().flatten() {
                if let Ok(Some(_token)) = storage.get_item("ruggine_auth_token") {
                    logging::log!("Token found, fetching groups...");
                    groups_hook.refresh_groups.dispatch(());
                } else {
                    logging::log!("No token found, skipping groups fetch");
                }
            }
        }
    });

    // Handle group selection
    let set_active_group = groups_hook.set_active_group;
    let active_group_id = groups_hook.active_group_id;
    
    let handle_group_click = move |group_id: i32| {
        set_active_group.set(Some(group_id));
        logging::log!("Selected group: {}", group_id);
    };


    // Stato per apertura modal inviti
    let (show_invites_modal, set_show_invites_modal) = create_signal(false);
    let (invites, set_invites) = create_signal(Vec::<Invitation>::new());
    let (is_loading_invites, set_is_loading_invites) = create_signal(false);

    let handle_create_click = move |_| {
        on_create_group_click.call(());
    };

    let handle_show_invites_click = move |_| {
        set_show_invites_modal.set(true);
        set_is_loading_invites.set(true);
        // Fetch inviti async
        spawn_local(async move {
            let storage_service = crate::utils::storage::StorageService::new();
            let mut http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
            if let Some(token_response) = storage_service.get_token() {
                http_client.set_auth_token(Some(token_response.token));
            }
            let service = InvitationService::new(http_client, storage_service);
            match service.get_user_invitations().await {
                Ok(list) => set_invites.set(list),
                Err(e) => {
                    logging::error!("Errore caricamento inviti: {:?}", e);
                    set_invites.set(Vec::new());
                }
            }
            set_is_loading_invites.set(false);
        });
    };

    let handle_close_invites_modal = move |_| {
        set_show_invites_modal.set(false);
    };

    view! {
        <div class="w-[280px] bg-bg-sidebar dark:bg-bg-sidebar-dark border-r border-border dark:border-border-dark flex flex-col h-full">
            {/* Sezione gruppi scrollabile */}
            <div class="flex-1 p-4 flex flex-col">
                <div class="text-xs font-semibold text-text-secondary dark:text-text-secondary-dark uppercase tracking-wide mb-3">
                    "Team e Gruppi"
                </div>
                <div class="flex-1 overflow-y-auto max-h-[340px] pr-1">
                    {move || {
                        if let Some(_error_msg) = error.get() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-4 space-y-2">
                                    <div class="text-sm text-red-500">
                                        "Errore nel caricamento"
                                    </div>
                                    <button 
                                        class="text-xs text-brand-primary-light hover:underline"
                                        on:click=move |_| groups_hook.refresh_groups.dispatch(())
                                    >
                                        "Riprova"
                                    </button>
                                </div>
                            }.into_view()
                        } else {
                            let groups = groups_list.get();
                            if groups.is_empty() && !is_loading.get() {
                                view! {
                                    <div class="flex items-center justify-center py-4">
                                        <div class="text-sm text-text-secondary dark:text-text-secondary-dark">
                                            "Nessun gruppo trovato"
                                        </div>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="space-y-1">
                                        {groups.into_iter().enumerate().map(|(index, group_data)| {
                                            let is_active = active_group_id.get() == Some(group_data.membership.group_chat_id);
                                            view! {
                                                <div 
                                                    class="animate-fade-in-up"
                                                    style=format!("animation-delay: {}ms", index * 100)
                                                >
                                                    <GroupItem 
                                                        group_data=group_data
                                                        is_active=is_active 
                                                        on_click=handle_group_click
                                                    />
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_view()
                            }
                        }
                    }}
                </div>
            </div>

            {/* Sezione fissa in fondo: Inviti e Crea gruppo */}
            <div class="p-4 border-t border-border dark:border-border-dark bg-bg-sidebar dark:bg-bg-sidebar-dark flex flex-col gap-2">
                {/* Bottone Crea gruppo */}
                <CreateGroupButton on_create_click=handle_create_click />
                {/* Bottone Inviti */}
                <ShowInvitesButton on_show_invites_click=handle_show_invites_click pending_count={invites.get().iter().filter(|i| i.status.to_string() == "pending").count()} />
            </div>

            {/* Modal Inviti */}
            <ShowInvitesModal
                is_open=show_invites_modal
                on_close=handle_close_invites_modal
                invites=invites
                set_invites=set_invites
                is_loading=is_loading_invites
            />
        </div>

    }
}
