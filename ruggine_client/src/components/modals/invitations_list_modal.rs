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

    // Load received invites only
    let load_received_invites = move || {
        let set_local_invites = set_local_invites.clone();
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
                log::info!("ShowInvitesModal: fetched invites count = {}", all_invites.len());
                let current_user_id = crate::utils::storage::StorageService::new()
                    .get_user_profile()
                    .map(|u| u.id);
                log::info!("ShowInvitesModal: current_user_id from storage = {:?}", current_user_id);
                // Only received invites (exclude self-invites)
                let filtered = all_invites.into_iter()
                    .filter(|inv| inv.to_user_id == current_user_id.unwrap_or(-1) && inv.from_user_id != inv.to_user_id)
                    .collect::<Vec<_>>();
                let local_len = filtered.len();
                set_local_invites.set(filtered);
                log::info!("ShowInvitesModal: local_invites set length = {}", local_len);
            }
        });
    };

    // Initial load for received invites
    load_received_invites();

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
                    // refresh local view with received invites only
                    set_local_invites.set(cloned.into_iter().filter(|inv| inv.to_user_id == crate::utils::storage::StorageService::new().get_user_profile().map(|u| u.id).unwrap_or(-1) && inv.from_user_id != inv.to_user_id).collect());
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
                    // refresh local view with received invites only
                    set_local_invites.set(cloned.into_iter().filter(|inv| inv.to_user_id == crate::utils::storage::StorageService::new().get_user_profile().map(|u| u.id).unwrap_or(-1) && inv.from_user_id != inv.to_user_id).collect());
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
                        <div class="divide-y divide-border dark:divide-border-dark overflow-hidden" style=move || {
                            // make the invites list scroll when it grows beyond a reasonable number
                            let list = local_invites.get();
                            if list.len() > 6 {
                                "max-height: 340px; overflow-y: auto;".to_string()
                            } else {
                                "max-height: none; overflow-y: visible;".to_string()
                            }
                        }>
                            {move || {
                                let list = local_invites.get();
                                let on_accept_cb = on_accept.clone();
                                let on_reject_cb = on_reject.clone();
                                if list.is_empty() {
                                    view! {
                                        <div class="py-8 text-center text-text-secondary dark:text-text-secondary-dark">
                                            "Nessun invito ricevuto."
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <ul class="space-y-0">
                                            {list.into_iter().map(|inv| {
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
                                                view! {
                                                    <li class="py-4 flex items-center gap-4">
                                                        <div class="flex-1 min-w-0">
                                                            <div class="font-medium text-text-primary dark:text-text-primary-dark">{group}</div>
                                                            <div class="text-xs text-text-secondary dark:text-text-secondary-dark mt-1">Inviato il {sent_at} | Ruolo : <span class="font-semibold">{role}</span></div>
                                                        </div>
                                                        <div class="flex gap-2">
                                                            {if is_pending {
                                                                // invitation is pending -> show actionable buttons
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
                                                                // not pending: show status label
                                                                view! {
                                                                    <span class="px-3 py-1 text-xs rounded bg-gray-300 dark:bg-gray-700 text-gray-600 dark:text-gray-300 cursor-default">
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