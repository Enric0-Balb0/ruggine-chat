use leptos::*;
use wasm_bindgen::JsCast;
use crate::components::{GroupItem, CreateGroupButton, ShowInvitesButton, OnlineUsersCounter};
use crate::components::modals::ShowInvitesModal;
use crate::api::services::invitation::InvitationService;
use crate::types::Invitation;
use crate::hooks::{use_groups_context, use_groups_list, use_groups_loading, use_groups_error};
use crate::context::unread_counts_context::use_unread_counts_context;
use crate::utils::error_recovery::NetworkOperation;

use crate::hooks::use_global_group_unread_ws;

/// Sidebar component for the main app layout
#[component]
pub fn Sidebar(
    #[prop(into)] on_create_group_click: Callback<()>,
) -> impl IntoView {
    let groups_context = use_groups_context();
    let groups_hook = groups_context.groups_hook;
    let groups_list = use_groups_list(&groups_hook);
    let is_loading = use_groups_loading(&groups_hook);
    let error = use_groups_error(&groups_hook);

    // Use unread counts context
    let unread_counts = use_unread_counts_context();

    // Lightweight memo for possible future debug; not rendered by default.
    let _debug_unread = create_memo(move |_| unread_counts.get().clone());

    // Hook globale per badge: aggiorna i badge in tempo reale anche fuori dalla chat
    // Passa sempre la lista aggiornata di group_id
    let group_ids = create_memo(move |_| {
        groups_list.get().iter().map(|g| g.membership.group_chat_id).collect::<Vec<_>>()
    });
    // Initialize global badge hook with current group ids (no-op if not implemented)
    // Also, perform an immediate authoritative refresh of unread counts for each group
    // when the sidebar mounts / the groups list changes so badges are correct on startup.
    create_effect(move |_| {
        let ids = group_ids.get();
        use_global_group_unread_ws(ids.clone());

        // Fire a one-off authoritative refresh for each group id (async)
        let ids_to_refresh = ids.clone();
        spawn_local(async move {
            for gid in ids_to_refresh.into_iter() {
                let _ = crate::context::unread_counts_context::refresh_unread_for_group(gid).await;
            }
        });
    });

    // Load groups on mount
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            if let Some(storage) = window.local_storage().ok().flatten() {
                if let Ok(Some(_token)) = storage.get_item("ruggine_auth_token") {
                    
                    groups_hook.refresh_groups.dispatch(());
                } else {
                    
                }
            }
        }
    });

    // Handle group selection
    let set_active_group = groups_hook.set_active_group;
    let active_group_id = groups_hook.active_group_id;
    
    let handle_group_click = move |group_id: i32| {
        set_active_group.set(Some(group_id));
    // Force a refresh of unread counts for this group when user opens it (quick reconcile)
    let gid = group_id;
    spawn_local(async move {
        let _ = crate::context::unread_counts_context::refresh_unread_for_group(gid).await;
    });
    };


    // Stato per apertura modal inviti
    let (show_invites_modal, set_show_invites_modal) = create_signal(false);
    // Local invites state (used by the modal props) — keep in sync with global context
    let (invites, set_invites) = create_signal(Vec::<Invitation>::new());
    let invitations_ctx = crate::context::invitations_context::use_invitations_context();
    // Use the global invitations pending count for the badge
    let pending_invites_count = crate::context::invitations_context::use_pending_invitations_count_context();
    let (is_loading_invites, set_is_loading_invites) = create_signal(false);

    // Auto-refresh degli inviti ogni 30 secondi per aggiornare il counter
    let auto_refresh_invites = move || {
        spawn_local(async move {
            let storage_service = crate::utils::storage::StorageService::new();
            
            // Controlla se abbiamo ancora un token valido
            if let Some(token_response) = storage_service.get_token() {
                let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                http_client.set_auth_token(Some(token_response.token));
                let service = InvitationService::new(http_client, storage_service);
                
                if let Some(invitations) = service.get_user_invitations()
                    .with_auto_retry("auto refresh invitations").await {
                    set_invites.set(invitations);
                } else {
                    // Se la chiamata fallisce (es. 401), potrebbe significare che il token è scaduto
                    logging::warn!("Failed to refresh invitations - token may be expired");
                }
            } else {
                // Nessun token disponibile, ferma gli auto-refresh
                logging::log!("No token available, skipping invitations refresh");
            }
        });
    };

    // Effettua il primo caricamento degli inviti al mount (one-shot)
    create_effect(move |_| {
        // Solo se abbiamo un token
        let storage_service = crate::utils::storage::StorageService::new();
        if storage_service.get_token().is_some() {
            auto_refresh_invites();
        }
    });

    let handle_create_click = move |_| {
        on_create_group_click.call(());
    };

    let handle_show_invites_click = move |_| {
        let storage_service = crate::utils::storage::StorageService::new();
        
        // Controlla se abbiamo un token valido prima di aprire la modale
        if let Some(token_response) = storage_service.get_token() {
            set_show_invites_modal.set(true);
            set_is_loading_invites.set(true);
            // Pre-fill modal with current context invitations so WS-updates are visible immediately
            set_invites.set(invitations_ctx.get());
            
            // Fetch inviti async
            spawn_local(async move {
                let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
                http_client.set_auth_token(Some(token_response.token));
                let service = InvitationService::new(http_client, storage_service);
                
                let invitations = service.get_user_invitations()
                    .with_auto_retry("load user invitations").await
                    .unwrap_or_else(|| {
                        logging::error!("Failed to load invitations after retries");
                        Vec::new()
                    });
                set_invites.set(invitations);
                set_is_loading_invites.set(false);
            });
        } else {
            logging::warn!("No valid token available, cannot load invitations");
        }
    };

    let handle_close_invites_modal = move |_| {
        set_show_invites_modal.set(false);
    };


    view! {
    <div class="w-[320px] bg-bg-sidebar dark:bg-bg-sidebar-dark border-r border-border dark:border-border-dark flex flex-col h-full">
            {/* Sezione gruppi scrollabile */}
            <div class="flex-1 p-4 flex flex-col">
                <div class="text-xs font-semibold text-text-secondary dark:text-text-secondary-dark uppercase tracking-wide mb-3">
                    "Team e Gruppi"
                </div>
                <div class="flex-1 overflow-y-auto max-h-[340px] pr-1">
                    
                    {move || {
                        if let Some(_err_msg) = error.get() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-4 px-3 space-y-3 text-center">
                                    <div class="p-3 rounded-full bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-300">
                                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12A9 9 0 113 12a9 9 0 0118 0z"></path>
                                        </svg>
                                    </div>
                                    <div class="text-sm font-medium text-text-primary dark:text-text-primary-dark">
                                        "Impossibile caricare i gruppi"
                                    </div>
                                    <div class="text-xs text-text-secondary dark:text-text-secondary-dark leading-relaxed">
                                        "Controlla la connessione e riprova. Se il problema persiste, prova a riavviare l'app."
                                    </div>
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
                                            let group_id = group_data.membership.group_chat_id;
                                            let is_active = active_group_id.get() == Some(group_id);
                                            // Create a per-group memo so the badge depends only on this group's unread count
                                            let unread_memo = create_memo(move |_| {
                                                unread_counts.get().get(&group_id).cloned().unwrap_or(0)
                                            });
                                            let unread_for_group = unread_memo.get();
                                            view! {
                                                <div 
                                                    class="animate-fade-in-up"
                                                    style=format!("animation-delay: {}ms", index * 100)
                                                >
                                                    <GroupItem 
                                                        group_data=group_data
                                                        is_active=is_active 
                                                        on_click=handle_group_click
                                                        unread_count={unread_for_group as i32}
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
            {move || {
                view! {
                    <div class="border-t border-border dark:border-border-dark bg-bg-sidebar dark:bg-bg-sidebar-dark flex flex-col">
                        <div class="p-4 flex flex-col gap-2">
                            {/* Bottone Crea gruppo */}
                            <CreateGroupButton on_create_click=handle_create_click />
                            {/* Bottone Inviti */}
                            <ShowInvitesButton on_show_invites_click=handle_show_invites_click pending_count={pending_invites_count.get()} />
                        </div>
                        {/* Contatore utenti online */}
                        <OnlineUsersCounter />
                    </div>
                }.into_view()
            }}

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
