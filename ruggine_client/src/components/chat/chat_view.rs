use leptos::*;
use leptos::html::Div;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
// use web_sys::MutationObserver;
use crate::hooks::GroupMembershipWithDetails;
use crate::api::services::GroupMembershipService;
use crate::components::use_toast;
use leptos_router::use_navigate;
use crate::hooks::use_groups_context;
use crate::components::{InviteMemberModal, InviteMemberRequest, MessageInputArea, GroupDetailsModal, LucideIcon};
use leptos::use_context;
use crate::components::chat::chat_message::{ChatMessage, MessageStatus};
use crate::types::message::Message;
use chrono::{TimeZone, Utc, Datelike, NaiveDate};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChatHeaderAction {
    InviteMembers,
    GroupDetails,
    LeaveGroup,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropdownState {
    Closed,
    Opening,
    Open,
    Closing,
}

#[component]
pub fn ChatView(
    #[prop(into)] group_data: GroupMembershipWithDetails,
) -> impl IntoView {
    let messages = vec![
        (
            Message {
                id: 1,
                content: "Ciao a tutti! Questo è un messaggio di esempio.".to_string(),
                sender_id: 42,
                group_chat_id: group_data.membership.group_chat_id,
                sent_at: Utc.ymd(2025, 8, 24).and_hms(15, 30, 0),
            },
            "alice".to_string(),
            "Alice".to_string(),
            "Rossi".to_string(),
            MessageStatus::Delivered,
            false,
        ),
        (
            Message {
                id: 2,
                content: "Messaggio inviato da me!".to_string(),
                sender_id: 99,
                group_chat_id: group_data.membership.group_chat_id,
                sent_at: Utc.ymd(2025, 8, 24).and_hms(15, 31, 0),
            },
            "io".to_string(),
            "Enrico".to_string(),
            "Bianchi".to_string(),
            MessageStatus::Sent,
            true,
        ),
        (
            Message {
                id: 3,
                content: "Come va il progetto?".to_string(),
                sender_id: 43,
                group_chat_id: group_data.membership.group_chat_id,
                sent_at: Utc.ymd(2025, 8, 24).and_hms(15, 32, 0),
            },
            "marco".to_string(),
            "Marco".to_string(),
            "Verdi".to_string(),
            MessageStatus::Delivered,
            false,
        ),
        (
            Message {
                id: 4,
                content: "Tutto bene! Sto lavorando sulla UI.".to_string(),
                sender_id: 99,
                group_chat_id: group_data.membership.group_chat_id,
                sent_at: Utc.ymd(2025, 8, 25).and_hms(0, 1, 0),
            },
            "io".to_string(),
            "Enrico".to_string(),
            "Bianchi".to_string(),
            MessageStatus::Sent,
            true,
        ),
        (
            Message {
                id: 5,
                content: "Ottimo! Poi fammi vedere il risultato.".to_string(),
                sender_id: 44,
                group_chat_id: group_data.membership.group_chat_id,
                sent_at: Utc.ymd(2025, 8, 25).and_hms(0, 2, 0),
            },
            "sofia".to_string(),
            "Sofia".to_string(),
            "Neri".to_string(),
            MessageStatus::Delivered,
            false,
        ),
        (
            Message {
                id: 6,
                content: "Certo! Appena pronto condivido uno screenshot.".to_string(),
                sender_id: 99,
                group_chat_id: group_data.membership.group_chat_id,
                sent_at: Utc.ymd(2025, 8, 25).and_hms(0, 3, 0),
            },
            "io".to_string(),
            "Enrico".to_string(),
            "Bianchi".to_string(),
            MessageStatus::Sent,
            true,
        ),
    ];
    let mut last_date: Option<NaiveDate> = None;
    let mut message_nodes: Vec<View> = vec![
        view! {
            <div class="text-center text-gray-500 dark:text-gray-200 text-sm italic py-2 bg-gray-50 dark:bg-gray-800 rounded-md border border-gray-200 dark:border-gray-700 mx-auto max-w-[80%] shadow-sm">
                Benvenuto nella chat di gruppo!
            </div>
        }.into_view()
    ];
    for (msg, username, name, surname, status, is_own) in messages {
        let msg_date = msg.sent_at.date_naive();
        if last_date.map_or(true, |d| d != msg_date) {
            let formatted = msg.sent_at.format("%A %d %B %Y").to_string();
            message_nodes.push(
                view! {
                    <div class="text-center text-gray-500 dark:text-gray-200 text-sm italic py-2 bg-gray-50 dark:bg-gray-800 rounded-md border border-gray-200 dark:border-gray-700 mx-auto max-w-[80%] shadow-sm">
                        {formatted}
                    </div>
                }.into_view()
            );
            last_date = Some(msg_date);
        }
        message_nodes.push(
            view! {
                <ChatMessage
                    message=msg.clone()
                    sender_username=username.clone()
                    sender_name=name.clone()
                    sender_surname=surname.clone()
                    status=status.clone()
                    is_own=is_own
                />
            }.into_view()
        );
    }
    let (dropdown_state, set_dropdown_state) = create_signal(DropdownState::Closed);
    let (invite_modal_open, set_invite_modal_open) = create_signal(false);
    let (group_details_modal_open, set_group_details_modal_open) = create_signal(false);
    let (leave_modal_open, set_leave_modal_open) = create_signal(false);
    let (leave_error, set_leave_error) = create_signal(None::<String>);
    let (leave_loading, set_leave_loading) = create_signal(false);

    let dropdown_ref = create_node_ref::<Div>();
    
    // Effect per chiudere il dropdown quando si clicca fuori
    create_effect(move |_| {
        let current_state = dropdown_state.get();
        if matches!(current_state, DropdownState::Open | DropdownState::Opening) {
            let handle_click_outside = move |event: web_sys::Event| {
                if let Some(dropdown_element) = dropdown_ref.get_untracked() {
                    if let Some(target) = event.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                            if !dropdown_element.contains(Some(&element)) {
                                set_dropdown_state.set(DropdownState::Closing);
                                // Dopo l'animazione di chiusura, imposta lo stato a Closed
                                set_timeout(
                                    move || set_dropdown_state.set(DropdownState::Closed),
                                    std::time::Duration::from_millis(150)
                                );
                            }
                        }
                    }
                }
            };
            
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(handle_click_outside) as Box<dyn FnMut(_)>);
            let _ = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });
    
    // Clone values needed for closures
    let group_name = group_data.group_name();
    let group_data_clone = group_data.clone();
    
    // Handle header actions
    let handle_header_action = move |action: ChatHeaderAction| {
        set_dropdown_state.set(DropdownState::Closing);
        set_timeout(
            move || set_dropdown_state.set(DropdownState::Closed),
            std::time::Duration::from_millis(150)
        );
        match action {
            ChatHeaderAction::InviteMembers => {
                set_invite_modal_open.set(true);
                logging::log!("Opening invite modal");
            }
            ChatHeaderAction::GroupDetails => {
                set_group_details_modal_open.set(true);
                logging::log!("Opening group details modal");
            }
            ChatHeaderAction::LeaveGroup => {
                set_leave_modal_open.set(true);
                set_leave_error.set(None);
                logging::log!("Showing leave confirmation");
            }
        }
    };

    // Leave group logic
    use std::rc::Rc;
    let handle_leave_group = {
        let set_leave_modal_open = set_leave_modal_open.clone();
        let set_leave_error = set_leave_error.clone();
        let set_leave_loading = set_leave_loading.clone();
        let group_data = group_data.clone();
        let navigate = use_navigate();
        let toast = use_toast();
        let groups_ctx = use_groups_context();
        Rc::new(move || {
            set_leave_loading.set(true);
            set_leave_error.set(None);
            let navigate = navigate.clone();
            let toast = toast.clone();
            let refresh_groups = groups_ctx.groups_hook.refresh_groups.clone();
            leptos::spawn_local(async move {
                use crate::utils::storage::StorageService;
                use crate::api::client::ApiClient;
                use crate::config::constants::AppConstants;
                let storage_service = StorageService::new();
                let mut http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                if let Some(token_response) = storage_service.get_token() {
                    http_client.set_auth_token(Some(token_response.token));
                }
                let service = GroupMembershipService::new(http_client, storage_service);
                match service.leave_group(group_data.membership.id).await {
                    Ok(_left) => {
                        set_leave_loading.set(false);
                        set_leave_modal_open.set(false);
                        // Refetch sidebar groups
                        refresh_groups.dispatch(());
                        // Show toast
                        toast.success("Hai abbandonato il gruppo con successo!");
                        // Redirect to home
                        navigate("/", Default::default());
                        leptos::logging::log!("Left group successfully");
                    }
                    Err(e) => {
                        set_leave_loading.set(false);
                        set_leave_error.set(Some(format!("Errore: {}", e)));
                    }
                }
            });
        })
    };

    // Handle dropdown toggle
    let handle_dropdown_toggle = move |_| {
        match dropdown_state.get_untracked() {
            DropdownState::Closed => {
                set_dropdown_state.set(DropdownState::Opening);
                set_timeout(
                    move || set_dropdown_state.set(DropdownState::Open),
                    std::time::Duration::from_millis(200)
                );
            }
            DropdownState::Open | DropdownState::Opening => {
                set_dropdown_state.set(DropdownState::Closing);
                set_timeout(
                    move || set_dropdown_state.set(DropdownState::Closed),
                    std::time::Duration::from_millis(150)
                );
            }
            DropdownState::Closing => {
                // Se già in chiusura, non fare nulla
            }
        }
    };

    // Handle invite member
    let handle_invite_member = move |invite_request: InviteMemberRequest| {
        logging::log!("Inviting user: {} with role: {:?}", invite_request.username, invite_request.role);
        // TODO: Implement actual invitation logic
        set_invite_modal_open.set(false);
    };

    // Handle modal close
    let handle_invite_modal_close = move |_| {
        set_invite_modal_open.set(false);
    };

    let handle_group_details_modal_close = move |_| {
        set_group_details_modal_open.set(false);
    };
    
    // Signal reattiva per il background in base al tema
    let (bg_url, set_bg_url) = create_signal(String::new());
    // Funzione per aggiornare il background
    let update_bg = {
        let set_bg_url = set_bg_url.clone();
        move || {
            let is_dark = leptos::window().document().unwrap().document_element().unwrap().class_list().contains("dark");
            if is_dark {
                set_bg_url.set("background-image: url('/public/images/bg-chat-dark.png'); background-size: cover; background-position: center; background-repeat: no-repeat;".to_string());
            } else {
                set_bg_url.set("background-image: url('/public/images/bg-chat-light.png'); background-size: cover; background-position: center; background-repeat: no-repeat;".to_string());
            }
        }
    };
    // Aggiorna subito
    update_bg();
    // Polling reattivo per aggiornare il background quando cambia il tema
    {
        use gloo_timers::callback::Interval;
        let update_bg_cb = update_bg.clone();
        create_effect(move |_| {
            // Aggiorna subito
            update_bg_cb();
            // Poll ogni 300ms
            let interval = Interval::new(300, move || {
                update_bg_cb();
            });
            // Cleanup: ferma il polling quando l'effetto viene droppato
            on_cleanup(move || {
                drop(interval);
            });
        });
    }
    view! {

    <div class="flex flex-col h-full bg-white dark:bg-surface-dark">
            // Chat Header - usando solo Tailwind
            <div class="px-6 py-4 border-b border-gray-200 dark:border-border-dark bg-gray-50 dark:bg-surface-dark flex justify-between items-center">
                <div class="flex-1">
                    <h2 class="text-xl font-semibold text-gray-800 dark:text-text-primary-dark mb-1">
                        {group_name.clone()}
                    </h2>
                    <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-text-secondary-dark">
                        <span>
                            {move || match group_data_clone.group_details.as_ref() {
                                Some(group) => match group.member_count {
                                    Some(count) => format!("{} membri", count),
                                    None => "Membri: N/A".to_string(),
                                },
                                None => "Caricamento...".to_string(),
                            }}
                        </span>
                        <span>"•"</span>
                        <span>"4 online"</span>
                    </div>
                </div>
                <div class="flex items-center gap-2 relative" node_ref=dropdown_ref>
                    <button 
                        class="bg-white dark:bg-surface-dark border border-gray-300 dark:border-gray-400 text-gray-700 dark:text-text-primary-dark px-3 py-1.5 rounded text-xs flex items-center gap-1 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors shadow-sm dark:shadow-gray-800/20"
                        on:click=handle_dropdown_toggle
                    >
                        <span>"Opzioni gruppo"</span>
                        <span>"⋮"</span>
                    </button>
                    <div
                        class=move || {
                            let base_classes = "absolute top-full right-0 mt-1 bg-white dark:bg-surface-dark border border-gray-200 dark:border-border-dark rounded shadow-lg dark:shadow-black/50 z-50 min-w-48 transition-opacity duration-200";
                            match dropdown_state.get() {
                                DropdownState::Closed => format!("{} opacity-0 pointer-events-none", base_classes),
                                DropdownState::Opening => format!("{} opacity-100 animate-dropdown-open pointer-events-auto", base_classes),
                                DropdownState::Open => format!("{} opacity-100 pointer-events-auto", base_classes),
                                DropdownState::Closing => format!("{} opacity-0 animate-dropdown-close pointer-events-none", base_classes),
                            }
                        }
                        style="will-change: opacity, transform;"
                    >
                        <div 
                            class="px-4 py-2 text-sm text-gray-700 dark:text-text-primary-dark hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer flex items-center gap-2 border-b border-gray-100 dark:border-border-dark"
                            on:click=move |_| handle_header_action(ChatHeaderAction::InviteMembers)
                        >
                            <LucideIcon name="user-plus" size=16 />
                            <span>"Invita membri"</span>
                        </div>

                        <div 
                            class="px-4 py-2 text-sm text-gray-700 dark:text-text-primary-dark hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer flex items-center gap-2 border-b border-gray-100 dark:border-border-dark"
                            on:click=move |_| handle_header_action(ChatHeaderAction::GroupDetails)
                        >
                            <LucideIcon name="settings" size=16 />
                            <span>"Dettagli gruppo"</span>
                        </div>
                        <div 
                            class="px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 cursor-pointer flex items-center gap-2"
                            on:click=move |_| handle_header_action(ChatHeaderAction::LeaveGroup)
                        >
                            <LucideIcon name="door-open" size=16 class="text-red-600" />
                            <span>"Abbandona gruppo"</span>
                        </div>
                    </div>
                </div>
            </div>


            // Content area - struttura base chat
            <div class="flex flex-col h-full min-h-0">
                {/* Scrollable message list with padding for input bar */}
                <div
                    class="messages-container custom-scrollbar flex-1 min-h-0 overflow-y-auto px-12 py-4 space-y-4"
                    style=move || format!("{};padding-bottom:72px;", bg_url.get())
                >
                    {message_nodes}
                </div>

                {/* Input bar always visible at the bottom */}
                <div class="shrink-0 bg-inherit z-10">
                    <MessageInputArea />
                </div>
            </div>

            // Invite Member Modal
            <InviteMemberModal
                is_open=invite_modal_open
                on_close=handle_invite_modal_close
                group_chat_id=group_data.membership.group_chat_id
                group_name=group_name.clone()
            />

            // Group Details Modal
            <GroupDetailsModal
                is_open=group_details_modal_open
                on_close=handle_group_details_modal_close
                group_id=group_data_clone.membership.group_chat_id
            />

            // Leave Group Confirmation Modal
            <Show when=move || leave_modal_open.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-40">
                    <div class="bg-white dark:bg-gray-900 rounded-lg shadow-lg p-6 w-full max-w-md">
                        <h3 class="text-lg font-semibold mb-2 text-gray-900 dark:text-white">Sei sicuro di voler abbandonare il gruppo?</h3>
                        <p class="mb-4 text-gray-700 dark:text-gray-300">Questa azione è irreversibile.</p>
                        <Show when=move || leave_error.get().is_some()>
                            <div class="mb-2 text-red-600 dark:text-red-400 text-sm">{move || leave_error.get().unwrap_or_default()}</div>
                        </Show>
                        <div class="flex justify-end gap-2 mt-4">
                            <button class="px-4 py-2 rounded bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600" on:click=move |_| set_leave_modal_open.set(false) disabled=move || leave_loading.get()>
                                Annulla
                            </button>
                            <button class="px-4 py-2 rounded bg-red-600 text-white hover:bg-red-700 disabled:opacity-60" on:click={{
                                let handle_leave_group = handle_leave_group.clone();
                                move |_| (handle_leave_group)()
                            }} disabled=move || leave_loading.get()>
                                {move || if leave_loading.get() { "Abbandono..." } else { "Abbandona" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
