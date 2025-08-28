use crate::context::unread_counts_context::use_unread_counts_context;
use leptos::*;
use leptos::html::Div;
// use wasm_bindgen::JsCast; // già importato sopra
use web_sys::{Element, HtmlDivElement};
// use wasm_bindgen::JsCast; // già importato sopra
// Importa il trait per get_bounding_client_rect
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use crate::hooks::GroupMembershipWithDetails;
use crate::api::services::GroupMembershipService;
use crate::components::use_toast;
use leptos_router::use_navigate;
use crate::hooks::use_groups_context;
use crate::components::{InviteMemberModal, InviteMemberRequest, MessageInputArea, GroupDetailsModal, LucideIcon};
use crate::hooks::use_group_socket_messages::use_group_socket_messages;
use crate::hooks::use_group_initial_messages::use_group_initial_messages;
use crate::components::chat::chat_message::{ChatMessage, MessageStatus};
use crate::types::message::Message;
use crate::utils::storage::StorageService;
use crate::hooks::use_group_user_cache::use_group_user_cache;


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
    // Signal for messages managed by the new hook
    use leptos::use_context;
    use crate::hooks::use_group_message_ws::UseGroupMessageWs;
    let ws_ctx = use_context::<Option<UseGroupMessageWs>>();
    // Signal per messaggi locali inviati via input
    let (local_messages, set_local_messages) = create_signal(Vec::<Message>::new());
    let set_local_messages_rc = Rc::new(set_local_messages);
    let unread_counts = use_unread_counts_context();
    let group_id_for_update = group_data.membership.group_chat_id;
    let add_message: Rc<dyn Fn(Message)> = {
        let set_local_messages_rc = Rc::clone(&set_local_messages_rc);
        let unread_counts = unread_counts.clone();
        let group_id = group_id_for_update;
        Rc::new(move |msg: Message| {
            set_local_messages_rc.update(|msgs| msgs.push(msg.clone()));
            // After sending, call update_message_read_at and decrement the counter using clone-set
            let unread_counts = unread_counts.clone();
            let msg_id = msg.id;
            leptos::spawn_local(async move {
                use crate::api::services::message::MessageService;
                use crate::config::constants::AppConstants;
                use crate::utils::storage::StorageService;
                use crate::api::client::ApiClient;
                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                let storage_service = StorageService::new();
                if let Some(token) = storage_service.get_token() {
                    http_client.set_auth_token(Some(token.token));
                }
                let message_service = MessageService::new(http_client, storage_service);
                let now = chrono::Utc::now().to_rfc3339();
                if message_service.update_message_read_at(msg_id, now).await.is_ok() {
                    // Use clone-modify-set to avoid in-place mutation of the RwSignal map
                    let mut cloned = unread_counts.get().clone();
                    let entry = cloned.entry(group_id).or_insert(0);
                    if *entry > 0 {
                        *entry -= 1;
                    }
                    unread_counts.set(cloned);
                }
            });
        })
    };
    let (initial_messages, initial_loading, initial_error) = use_group_initial_messages(group_data.membership.group_chat_id, 50);
    let user_cache = use_group_user_cache(group_data.membership.group_chat_id);

    // diagnostic block removed

    let ws_messages = use_group_socket_messages(
        group_data.membership.group_chat_id,
        ws_ctx.as_ref().and_then(|w| w.as_ref().cloned()),
        local_messages,
    );

    // Use unread counts context for badge decrement
    let unread_counts = use_unread_counts_context();

    // Call this after a successful update_message_read_at for a message in this group
    fn decrement_unread_for_group(unread_counts: &leptos::RwSignal<std::collections::HashMap<i32, u32>>, group_id: i32) {
        unread_counts.update(|map| {
            let entry = map.entry(group_id).or_insert(0);
            if *entry > 0 {
                *entry -= 1;
            }
        });
    }
    let messages = create_memo(move |_| {
        let mut all_msgs = Vec::new();
        all_msgs.extend(initial_messages.get());
        all_msgs.extend(ws_messages.get());
        all_msgs.extend(local_messages.get());
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for msg in all_msgs {
            map.insert(msg.id, msg);
        }
        let mut deduped: Vec<_> = map.into_values().collect();
        deduped.sort_by_key(|m| m.sent_at);
        deduped
    });
    let (dropdown_state, set_dropdown_state) = create_signal(DropdownState::Closed);
    let (invite_modal_open, set_invite_modal_open) = create_signal(false);
    let (group_details_modal_open, set_group_details_modal_open) = create_signal(false);
    let (leave_modal_open, set_leave_modal_open) = create_signal(false);
    let (leave_error, set_leave_error) = create_signal(None::<String>);
    let (leave_loading, set_leave_loading) = create_signal(false);

    let dropdown_ref = create_node_ref::<Div>();
    let messages_container_ref = create_node_ref::<Div>();
    // Funzione per verificare se un elemento è visibile nel container scrollabile
    fn is_element_in_viewport(container: &HtmlDivElement, element: &Element) -> bool {
    let container_rect = container.get_bounding_client_rect();
    let elem_rect = element.get_bounding_client_rect();
    elem_rect.top() < container_rect.bottom() && elem_rect.bottom() > container_rect.top()
    }

    // Effetto: aggiungi event listener su scroll per update read_at
    // Effetto: scroll automatico all'ultimo messaggio dopo il caricamento
    {
        let messages = messages.clone();
        let messages_container_ref = messages_container_ref.clone();
        create_effect(move |_| {
            let msgs = messages.get();
            if msgs.is_empty() {
                return;
            }
            // Prendi l'ultimo messaggio (il più recente)
            let last_msg = msgs.last().unwrap();
            if let Some(container) = messages_container_ref.get() {
                let doc = web_sys::window().unwrap().document().unwrap();
                if let Some(last_elem) = doc.get_element_by_id(&format!("msg-{}", last_msg.id)) {
                    // Scrolla il container in modo che l'ultimo messaggio sia visibile
                    let _ = last_elem.scroll_into_view_with_bool(true);
                }
            }
        });
    }
    {
        use crate::api::services::message::MessageService;
        use crate::config::constants::AppConstants;
        use crate::utils::storage::StorageService;
        use crate::api::client::ApiClient;
        let messages = messages.clone();
        let messages_container_ref = messages_container_ref.clone();
        create_effect(move |_| {
            if let Some(container) = messages_container_ref.get() {
                let container_clone = container.clone();
                let messages = messages.clone();
                let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    let doc = web_sys::window().unwrap().document().unwrap();
                    let msg_ids: Vec<i32> = messages.get().iter().map(|m| m.id).collect();
                    for msg_id in msg_ids {
                        if let Some(elem) = doc.get_element_by_id(&format!("msg-{}", msg_id)) {
                            if is_element_in_viewport(&container_clone, &elem) {
                                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                                let storage_service = StorageService::new();
                                if let Some(token) = storage_service.get_token() {
                                    http_client.set_auth_token(Some(token.token));
                                }
                                let message_service = MessageService::new(http_client, storage_service);
                                let now = chrono::Utc::now().to_rfc3339();
                                leptos::spawn_local(async move {
                                    let _ = message_service.update_message_read_at(msg_id, now).await;
                                });
                            }
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                let _ = container.add_event_listener_with_callback("scroll", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        });
    }
    
    // Effect to close the dropdown when clicking outside
    create_effect(move |_| {
        let current_state = dropdown_state.get();
        if matches!(current_state, DropdownState::Open | DropdownState::Opening) {
            let handle_click_outside = move |event: web_sys::Event| {
                if let Some(dropdown_element) = dropdown_ref.get_untracked() {
                    if let Some(target) = event.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                            if !dropdown_element.contains(Some(&element)) {
                                set_dropdown_state.set(DropdownState::Closing);
                                // After the closing animation, set the state to Closed
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
                
            }
            ChatHeaderAction::GroupDetails => {
                set_group_details_modal_open.set(true);
                
            }
            ChatHeaderAction::LeaveGroup => {
                set_leave_modal_open.set(true);
                set_leave_error.set(None);
                
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
                let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
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
                        // left group
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
                // If already closing, do nothing
            }
        }
    };

    // Handle invite member
    let handle_invite_member = move |invite_request: InviteMemberRequest| {
        
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
    
    // Reactive signal for the background based on the theme
    let (bg_url, set_bg_url) = create_signal(String::new());
    // Function to update the background
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
    // Update immediately
    update_bg();
    // Reactive polling to update the background when the theme changes
    {
        use gloo_timers::callback::Interval;
        let update_bg_cb = update_bg.clone();
        create_effect(move |_| {
            // Update immediately
            update_bg_cb();
            // Poll every 300ms
            let interval = Interval::new(300, move || {
                update_bg_cb();
            });
            // Cleanup: stop polling when the effect is dropped
            on_cleanup(move || {
                drop(interval);
            });
        });
    

    // User cache: populated on chat open with all group members
    let user_cache = use_group_user_cache(group_data.membership.group_chat_id);

    view! {
    <div class="flex flex-col h-full bg-white dark:bg-surface-dark">
        
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
    // Content area - base chat structure
        <div class="flex flex-col h-full min-h-0">
            
            <div
                class="messages-container custom-scrollbar flex-1 min-h-0 overflow-y-auto px-12 py-4 space-y-4"
                node_ref=messages_container_ref
                style=move || format!("{};padding-bottom:24px;", bg_url.get())
            >
                {move || {
                    let msgs = messages.get();
                    let users = user_cache.get();
                    let storage_service = StorageService::new();
                    let user_profile = storage_service.get_user_profile();
                    msgs.iter().map(|msg| {
                        let (is_own, sender_username, sender_name, sender_surname) = if let Some(ref user) = user_profile {
                            if msg.sender_id == user.id {
                                (true, user.username.clone(), user.first_name.clone(), user.last_name.clone())
                            } else if let Some(sender) = users.get(&msg.sender_id) {
                                (false, sender.username.clone(), sender.first_name.clone(), sender.last_name.clone())
                            } else {
                                (false, "?".to_string(), "".to_string(), "".to_string())
                            }
                        } else {
                            (false, "?".to_string(), "".to_string(), "".to_string())
                        };
                        view! {
                            <div id={format!("msg-{}", msg.id)}>
                                <ChatMessage
                                    message=msg.clone()
                                    sender_username=sender_username
                                    sender_name=sender_name
                                    sender_surname=sender_surname
                                    status=MessageStatus::Delivered
                                    is_own=is_own
                                />
                            </div>
                        }
                    }).collect_view()
                }}
                <Show when=move || messages.get().is_empty()>
                    <div class="text-center text-gray-500 dark:text-gray-200 text-sm italic py-2 bg-gray-50 dark:bg-gray-800 rounded-md border border-gray-200 dark:border-gray-700 mx-auto max-w-[80%] shadow-sm">
                        Nessun messaggio ancora. Inizia la conversazione!
                    </div>
                </Show>
            </div>
            
            <div class="shrink-0 bg-inherit z-10">
                {move || {
                    use leptos::use_context;
                    let ws_ctx = use_context::<Option<crate::hooks::use_group_message_ws::UseGroupMessageWs>>();
                    view! {
                        <MessageInputArea
                            ws_ctx=ws_ctx.flatten()
                            group_id=group_data.membership.group_chat_id
                            on_message_sent=add_message.clone()
                        />
                    }
                }}
            </div>
        </div>
        
        <InviteMemberModal
            is_open=invite_modal_open
            on_close=handle_invite_modal_close
            group_chat_id=group_data.membership.group_chat_id
            group_name=group_name.clone()
        />
        
        <GroupDetailsModal
            is_open=group_details_modal_open
            on_close=handle_group_details_modal_close
            group_id=group_data_clone.membership.group_chat_id
        />
        
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
}
