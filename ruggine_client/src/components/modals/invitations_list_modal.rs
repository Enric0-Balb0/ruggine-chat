use wasm_bindgen::JsCast;
use leptos::*;
use crate::types::Invitation;
use crate::api::services::group::GroupChatService;
use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::config::constants::AppConstants;
use crate::components::LucideIcon;
use crate::hooks::groups_provider::use_groups_context;
use crate::components::ui::feedback::use_toast;
use crate::utils::error_recovery::NetworkOperation;

#[component]
pub fn ShowInvitesModal(
    #[prop(into)] is_open: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] invites: ReadSignal<Vec<Invitation>>,
    #[prop(into)] set_invites: WriteSignal<Vec<Invitation>>,
    #[prop(into, optional)] is_loading: Option<ReadSignal<bool>>,
    #[prop(into, optional)] on_accept: Option<Callback<i32>>,
    #[prop(into, optional)] on_reject: Option<Callback<i32>>,
) -> impl IntoView {
    let (is_visible, set_is_visible) = create_signal(false);
    let (is_animating_in, set_is_animating_in) = create_signal(false);

    // Gestione animazioni apertura/chiusura
    create_effect(move |_| {
        let open = is_open.get();
        if open {
            set_is_visible.set(true);
            set_timeout(move || set_is_animating_in.set(true), std::time::Duration::from_millis(10));
        } else {
            set_is_animating_in.set(false);
            set_timeout(move || set_is_visible.set(false), std::time::Duration::from_millis(250));
        }
    });


    let (group_names, set_group_names) = create_signal(std::collections::HashMap::<i32, String>::new());

    create_effect(move |_| {
        let invites_list = invites.get();
        let mut missing_group_ids = vec![];
        let names_map = group_names.get();
        for inv in &invites_list {
            if !names_map.contains_key(&inv.group_chat_id) {
                missing_group_ids.push(inv.group_chat_id);
            }
        }
        if !missing_group_ids.is_empty() {
            let mut names_map = names_map.clone();
            let storage_service = StorageService::new();
            let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
            if let Some(token_response) = storage_service.get_token() {
                http_client.set_auth_token(Some(token_response.token));
            }
            let group_service = GroupChatService::new(http_client, storage_service);
            spawn_local(async move {
                for group_id in missing_group_ids {
                    let group_id_str = group_id.to_string();
                    if let Some(group) = group_service.get_group_by_id(&group_id_str)
                        .with_auto_retry("load group name").await {
                        names_map.insert(group_id, group.name);
                    }
                }
                set_group_names.set(names_map);
            });
        }
    });

    // Local copy of received invites only
    let (local_invites, set_local_invites) = create_signal(Vec::<Invitation>::new());
    let (all_invites, set_all_invites) = create_signal(Vec::<Invitation>::new());
    
    // Tab selection state
    let (active_tab, set_active_tab) = create_signal("pending"); // "pending" or "history"

    // Load received invites only
    let load_received_invites = move || {
        let set_local_invites = set_local_invites.clone();
        let set_all_invites = set_all_invites.clone();
        spawn_local(async move {
            let storage_service = StorageService::new();
            let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
            if let Some(token_response) = storage_service.get_token() {
                http_client.set_auth_token(Some(token_response.token));
            }
            let invitation_service = crate::api::services::invitation::InvitationService::new(http_client, storage_service);
            if let Some(all_invites) = invitation_service.get_user_invitations()
                .with_auto_retry("load user invitations").await {
                // diagnostic logs: see how many invites returned and which user id we read from storage
                leptos::logging::log!("ShowInvitesModal: fetched invites count = {}", all_invites.len());
                
                let current_user_id = crate::utils::storage::StorageService::new()
                    .get_user_profile()
                    .map(|u| u.id);
                leptos::logging::log!("ShowInvitesModal: current_user_id from storage = {:?}", current_user_id);
                
                // Debug: log all invites before filtering
                for inv in &all_invites {
                    leptos::logging::log!("ShowInvitesModal: invite {} - from_user_id: {}, to_user_id: {}, status: {}", 
                        inv.id, inv.from_user_id, inv.to_user_id, inv.status);
                }
                
                // Filter pending invites
                let pending_filtered = all_invites.clone().into_iter()
                    .filter(|inv| {
                        if let Some(user_id) = current_user_id {
                            let is_for_me = inv.to_user_id == user_id;
                            let not_self_invite = inv.from_user_id != inv.to_user_id;
                            let is_pending = inv.status == crate::types::invitation::InvitationStatus::Pending;
                            is_for_me && not_self_invite && is_pending
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<_>>();
                
                // Filter all received invites (for history) - exclude pending
                let all_filtered = all_invites.into_iter()
                    .filter(|inv| {
                        if let Some(user_id) = current_user_id {
                            let is_for_me = inv.to_user_id == user_id;
                            let not_self_invite = inv.from_user_id != inv.to_user_id;
                            let is_not_pending = inv.status != crate::types::invitation::InvitationStatus::Pending;
                            is_for_me && not_self_invite && is_not_pending
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<_>>();
                
                let pending_len = pending_filtered.len();
                let all_len = all_filtered.len();
                leptos::logging::log!("ShowInvitesModal: pending_invites length = {}, all_invites length = {}", pending_len, all_len);
                
                set_local_invites.set(pending_filtered);
                set_all_invites.set(all_filtered);
            } else {
                leptos::logging::log!("ShowInvitesModal: failed to load invitations");
            }
        });
    };

    // Set up polling to refresh invites when modal is open
    {
        let load_fn = load_received_invites.clone();
        create_effect(move |_| {
            if is_open.get() {
                leptos::logging::log!("ShowInvitesModal: Modal opened, loading invites immediately");
                // Load immediately when modal opens
                load_fn();
                
                // Then start polling every 10 seconds
                let load_fn_poll = load_fn.clone();
                spawn_local(async move {
                    loop {
                        crate::utils::sleep_ms(10000).await; // Wait 10 seconds
                        leptos::logging::log!("ShowInvitesModal: Polling for invitation updates");
                        load_fn_poll();
                    }
                });
            }
        });
    }

    let groups_ctx = use_groups_context();
    let handle_close = move |_| {
        // Refetch gruppi quando si chiude il modal
        groups_ctx.groups_hook.refresh_groups.dispatch(());
        on_close.call(());
    };

    let handle_backdrop_click = move |e: web_sys::MouseEvent| {
        if let Some(target) = e.target() {
            let element = target.unchecked_into::<web_sys::HtmlElement>();
            if element.class_list().contains("modal-backdrop") {
                handle_close(());
            }
        }
    };

    // Funzione per accettare un invito
    let set_invites_signal = set_invites.clone();
    let toast = use_toast();
    // Track pending invite actions to disable buttons per-invite
    let (pending_invites, set_pending_invites) = create_signal(std::collections::HashSet::<i32>::new());
    // prepare clones for closures to avoid move-after-use
    let toast_for_accept = toast.clone();
    let toast_for_reject = toast.clone();
    let set_pending_for_accept = set_pending_invites.clone();
    let set_pending_for_reject = set_pending_invites.clone();
    let on_accept_cb_clone = on_accept.clone();
    let on_reject_cb_clone = on_reject.clone();
    let handle_accept_invite = Callback::new(move |invitation_id: i32| {
        let set_invites_signal = set_invites_signal.clone();
        let toast = toast_for_accept.clone();
        let set_pending = set_pending_for_accept.clone();
        let on_accept_cb = on_accept_cb_clone.clone();
        spawn_local(async move {
            // mark pending
            set_pending.update(|s| { s.insert(invitation_id); });
            let storage_service = StorageService::new();
            let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
            if let Some(token_response) = storage_service.get_token() {
                http_client.set_auth_token(Some(token_response.token));
            }
            let invitation_service = crate::api::services::invitation::InvitationService::new(http_client, storage_service);
            let req = crate::types::invitation::InvitationUpdateRequest {
                status: crate::types::invitation::InvitationStatus::Accepted,
                invitation_id,
            };
            if let Some(_res) = invitation_service.update_invitation_status(&req)
                .with_auto_retry("accept invitation").await {
                // Aggiorna la lista inviti dopo l'accettazione
                if let Some(new_list) = invitation_service.get_user_invitations()
                    .with_auto_retry("reload invitations").await {
                    // clone before moving into set to allow creating filtered local list
                    let cloned = new_list.clone();
                    set_invites_signal.set(new_list);
                    // refresh both pending and all invites lists
                    let current_user_id = crate::utils::storage::StorageService::new()
                        .get_user_profile()
                        .map(|u| u.id);
                    
                    let all_filtered = cloned.clone().into_iter().filter(|inv| {
                        if let Some(user_id) = current_user_id {
                            inv.to_user_id == user_id && inv.from_user_id != inv.to_user_id
                        } else {
                            false
                        }
                    }).collect::<Vec<_>>();
                    
                    let pending_filtered = cloned.into_iter().filter(|inv| {
                        if let Some(user_id) = current_user_id {
                            inv.to_user_id == user_id && 
                            inv.from_user_id != inv.to_user_id && 
                            inv.status == crate::types::invitation::InvitationStatus::Pending
                        } else {
                            false
                        }
                    }).collect::<Vec<_>>();
                    
                    set_local_invites.set(pending_filtered);
                    set_all_invites.set(all_filtered);
                }
                // Toast di successo
                toast.success("Invito accettato! Ora fai parte del gruppo.");
                // notify optional external handler
                if let Some(cb) = on_accept.as_ref() {
                    cb.call(invitation_id);
                }
            }
            // clear pending
            set_pending.update(|s| { s.remove(&invitation_id); });
        });
    });

    // Funzione per rifiutare un invito (gestita internamente qui)
    let set_invites_signal_rej = set_invites.clone();
    let handle_reject_invite_internal = Callback::new(move |invitation_id: i32| {
        let set_invites_signal = set_invites_signal_rej.clone();
        let toast = toast_for_reject.clone();
        let set_pending = set_pending_for_reject.clone();
        let on_reject_cb = on_reject_cb_clone.clone();
        spawn_local(async move {
            set_pending.update(|s| { s.insert(invitation_id); });
            let storage_service = StorageService::new();
            let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
            if let Some(token_response) = storage_service.get_token() {
                http_client.set_auth_token(Some(token_response.token));
            }
            let invitation_service = crate::api::services::invitation::InvitationService::new(http_client, storage_service);
            let req = crate::types::invitation::InvitationUpdateRequest {
                status: crate::types::invitation::InvitationStatus::Rejected,
                invitation_id,
            };
            if let Some(_res) = invitation_service.update_invitation_status(&req)
                .with_auto_retry("reject invitation").await {
                if let Some(new_list) = invitation_service.get_user_invitations()
                    .with_auto_retry("reload invitations").await {
                    let cloned = new_list.clone();
                    set_invites_signal.set(new_list);
                    // refresh both pending and all invites lists
                    let current_user_id = crate::utils::storage::StorageService::new()
                        .get_user_profile()
                        .map(|u| u.id);
                    
                    let all_filtered = cloned.clone().into_iter().filter(|inv| {
                        if let Some(user_id) = current_user_id {
                            inv.to_user_id == user_id && inv.from_user_id != inv.to_user_id
                        } else {
                            false
                        }
                    }).collect::<Vec<_>>();
                    
                    let pending_filtered = cloned.into_iter().filter(|inv| {
                        if let Some(user_id) = current_user_id {
                            inv.to_user_id == user_id && 
                            inv.from_user_id != inv.to_user_id && 
                            inv.status == crate::types::invitation::InvitationStatus::Pending
                        } else {
                            false
                        }
                    }).collect::<Vec<_>>();
                    
                    set_local_invites.set(pending_filtered);
                    set_all_invites.set(all_filtered);
                }
                toast.success("Invito rifiutato.");
                // Call external callback if present (for side-effects like refresh)
                if let Some(cb) = on_reject_cb.as_ref() {
                    cb.call(invitation_id);
                }
            } else {
                toast.error("Errore durante il rifiuto dell'invito.");
            }
            set_pending.update(|s| { s.remove(&invitation_id); });
        });
    });

    view! {
        {move || if is_visible.get() {
            view! {
                <div 
                    class="fixed inset-0 z-50 flex items-center justify-center modal-backdrop"
                    style=move || {
                        if is_animating_in.get() {
                            "background-color: rgba(0, 0, 0, 0.65); backdrop-filter: blur(4px); opacity: 1; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);"
                        } else {
                            "background-color: rgba(0, 0, 0, 0); backdrop-filter: blur(0px); opacity: 0; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);"
                        }
                    }
                    on:click=handle_backdrop_click
                >
                    <div 
                        class="bg-white dark:bg-surface-dark shadow-2xl dark:shadow-black/50 border border-border dark:border-border-dark rounded-lg modal-container"
                        style=move || {
                            let base_style = "width: 100%; max-width: 540px; margin: 0 20px; padding: 24px;";
                            if is_animating_in.get() {
                                format!("{}transform: scale(1) translateY(0px); opacity: 1; transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);", base_style)
                            } else {
                                format!("{}transform: scale(0.9) translateY(-20px); opacity: 0; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);", base_style)
                            }
                        }
                    >
                        <div class="flex items-center justify-between mb-4">
                            <h2 class="text-xl font-semibold text-text-primary dark:text-text-primary-dark m-0">
                                "Inviti Ricevuti"
                            </h2>
                            <button
                                type="button"
                                class="bg-transparent border-none text-text-secondary dark:text-text-secondary-dark cursor-pointer p-2 rounded-lg transition-all duration-200 w-9 h-9 flex items-center justify-center hover:bg-red-50 hover:text-red-600 hover:scale-110 dark:hover:bg-red-900/20 dark:hover:text-red-400 group"
                                on:click=move |_| handle_close(())
                            >
                                <svg 
                                    class="w-5 h-5 transition-transform duration-300 group-hover:rotate-90"
                                    fill="none" 
                                    stroke="currentColor" 
                                    viewBox="0 0 24 24"
                                >
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        </div>
                        <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                            <div class="flex items-start gap-3">
                                <LucideIcon name="lightbulb" size=20 class="text-blue-500 dark:text-blue-400 mt-0.5" />
                                <div class="flex-1">
                                    <p class="m-0 text-sm text-blue-800 dark:text-blue-200 leading-relaxed">
                                        <strong class="font-medium">Gestisci i tuoi inviti ricevuti</strong>
                                        <br/>
                                        Visualizza, accetta o rifiuta gli inviti ai gruppi che hai ricevuto. Puoi vedere lo stato di ogni invito, quando è stato inviato e il ruolo che ti è stato assegnato nel gruppo.
                                    </p>
                                </div>
                            </div>
                        </div>
                        
                        // Tab navigation
                        <div class="flex border-b border-border dark:border-border-dark mb-4">
                            <button
                                class=move || {
                                    let base = "px-4 py-2 text-sm font-medium transition-all duration-200 border-b-2 ";
                                    if active_tab.get() == "pending" {
                                        format!("{}text-blue-600 dark:text-blue-400 border-blue-600 dark:border-blue-400", base)
                                    } else {
                                        format!("{}text-text-secondary dark:text-text-secondary-dark border-transparent hover:text-text-primary dark:hover:text-text-primary-dark", base)
                                    }
                                }
                                on:click=move |_| set_active_tab.set("pending")
                            >
                                {move || {
                                    let pending_count = local_invites.get().len();
                                    if pending_count > 0 {
                                        format!("In attesa ({})", pending_count)
                                    } else {
                                        "In attesa".to_string()
                                    }
                                }}
                            </button>
                            <button
                                class=move || {
                                    let base = "px-4 py-2 text-sm font-medium transition-all duration-200 border-b-2 ";
                                    if active_tab.get() == "history" {
                                        format!("{}text-blue-600 dark:text-blue-400 border-blue-600 dark:border-blue-400", base)
                                    } else {
                                        format!("{}text-text-secondary dark:text-text-secondary-dark border-transparent hover:text-text-primary dark:hover:text-text-primary-dark", base)
                                    }
                                }
                                on:click=move |_| set_active_tab.set("history")
                            >
                                {move || {
                                    let total_count = all_invites.get().len();
                                    format!("Cronologia ({})", total_count)
                                }}
                            </button>
                        </div>
                        <div class="divide-y divide-border dark:divide-border-dark overflow-hidden" style=move || {
                            // make the invites list scroll when it grows beyond a reasonable number
                            let current_list = if active_tab.get() == "pending" { 
                                local_invites.get() 
                            } else { 
                                all_invites.get() 
                            };
                            if current_list.len() > 6 {
                                "max-height: 340px; overflow-y: auto;".to_string()
                            } else {
                                "max-height: none; overflow-y: visible;".to_string()
                            }
                        }>
                            {move || {
                                let current_list = if active_tab.get() == "pending" { 
                                    local_invites.get() 
                                } else { 
                                    all_invites.get() 
                                };
                                let tab = active_tab.get();
                                let on_accept_cb = on_accept.clone();
                                let on_reject_cb = on_reject.clone();
                                
                                if current_list.is_empty() {
                                    let empty_message = if tab == "pending" {
                                        "Nessun invito in attesa."
                                    } else {
                                        "Nessun invito ricevuto."
                                    };
                                    view! {
                                        <div class="py-8 text-center text-text-secondary dark:text-text-secondary-dark">
                                            {empty_message}
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <ul class="space-y-0">
                                            {current_list.into_iter().map(|inv| {
                                                let id = inv.id;
                                                let group = group_names.get().get(&inv.group_chat_id).cloned().unwrap_or_else(|| format!("Gruppo {}", inv.group_chat_id));
                                                let status = inv.status.clone();
                                                let sent_at = inv.created_at.format(" %d/%m/%Y %H:%M ").to_string();
                                                let is_pending = status.to_string() == "pending";
                                                // check if an action (accept/reject) for this invitation is currently pending
                                                let is_action_pending = pending_invites.get().contains(&id);
                                                let on_accept_cb = on_accept_cb.clone();
                                                let on_reject_cb = on_reject_cb.clone();
                                                let role = match &inv.role_at_join {
                                                    crate::types::invitation::MemberRole::Admin => " Admin",
                                                    crate::types::invitation::MemberRole::Member => " Membro",
                                                };
                                                
                                                // Show responded date if available
                                                let responded_info = if let Some(responded_at) = inv.responded_at {
                                                    format!(" | Risposto il {}", responded_at.format("%d/%m/%Y %H:%M"))
                                                } else {
                                                    String::new()
                                                };
                                                
                                                view! {
                                                    <li class="py-4 flex items-center gap-4">
                                                        <div class="flex-1 min-w-0">
                                                            <div class="font-medium text-text-primary dark:text-text-primary-dark">{group}</div>
                                                            <div class="text-xs text-text-secondary dark:text-text-secondary-dark mt-1">
                                                                Inviato il {sent_at}{responded_info} | Ruolo : <span class="font-semibold">{role}</span>
                                                            </div>
                                                        </div>
                                                        <div class="flex gap-2">
                                                            {if is_pending && tab == "pending" {
                                                                // invitation is pending in the pending tab -> show actionable buttons
                                                                if is_action_pending {
                                                                    view! {
                                                                        <>
                                                                            <button class="px-3 py-1 text-xs rounded bg-green-500 text-white hover:bg-green-600 transition-colors opacity-60 cursor-not-allowed" disabled=true>
                                                                                "Accetta"
                                                                            </button>
                                                                            <button class="px-3 py-1 text-xs rounded bg-red-500 text-white hover:bg-red-600 transition-colors opacity-60 cursor-not-allowed" disabled=true>
                                                                                "Rifiuta"
                                                                            </button>
                                                                        </>
                                                                    }.into_view()
                                                                } else {
                                                                    view! {
                                                                        <>
                                                                            <button class="px-3 py-1 text-xs rounded bg-green-500 text-white hover:bg-green-600 transition-colors" on:click=move |_| handle_accept_invite.call(id)>
                                                                                "Accetta"
                                                                            </button>
                                                                            <button class="px-3 py-1 text-xs rounded bg-red-500 text-white hover:bg-red-600 transition-colors" on:click=move |_| handle_reject_invite_internal.call(id)>
                                                                                "Rifiuta"
                                                                            </button>
                                                                        </>
                                                                    }.into_view()
                                                                }
                                                            } else {
                                                                // in history tab or not pending: show status label
                                                                let status_class = match status {
                                                                    crate::types::invitation::InvitationStatus::Accepted => "bg-green-100 dark:bg-green-700/20 text-green-800 dark:text-green-200",
                                                                    crate::types::invitation::InvitationStatus::Rejected => "bg-red-100 dark:bg-red-700/20 text-red-800 dark:text-red-200",
                                                                    _ => "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300",
                                                                };
                                                                view! {
                                                                    <span class=format!("px-3 py-1 text-xs rounded cursor-default {}", status_class)>
                                                                        {status.display_name()}
                                                                    </span>
                                                                }.into_view()
                                                            }}
                                                        </div>
                                                    </li>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </ul>
                                    }.into_view()
                                }
                            }}
                        </div>
                    </div>
                </div>
            }.into_view() 
            } else {
                view! { <></> }.into_view()
            }
        }
    }
}