use crate::hooks::fetch_missing_users::fetch_missing_users;
use leptos::*;
use leptos::For;
use leptos::html::Div;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use std::rc::Rc;
use crate::hooks::GroupMembershipWithDetails;
use crate::api::services::GroupMembershipService;
use crate::components::use_toast;
use leptos_router::use_navigate;
use crate::hooks::use_groups_context;
use crate::components::{InviteMemberModal, InviteMemberRequest, MessageInputArea, GroupDetailsModal, LucideIcon};
use crate::hooks::use_group_socket_messages::use_group_socket_messages;
use leptos::use_context;
use crate::components::chat::chat_message::{ChatMessage, MessageStatus};
use crate::types::message::Message;
use crate::utils::storage::StorageService;
use crate::api::client::ApiClient;
use crate::types::user::UserProfile;
use std::collections::HashMap;
use crate::hooks::use_group_user_cache::use_group_user_cache;
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
    // Signal for messages managed by the new hook
    use leptos::use_context;
    use crate::hooks::use_group_message_ws::UseGroupMessageWs;
    let ws_ctx = use_context::<Option<UseGroupMessageWs>>();

    // Local signal for messages sent via REST (immediate)
    let (local_messages, set_local_messages) = create_signal(Vec::<Message>::new());
    // Callback to add a local message
    let set_local_messages_rc = Rc::new(set_local_messages);
    let add_message: Rc<dyn Fn(Message)> = {
        let set_local_messages_rc = Rc::clone(&set_local_messages_rc);
        Rc::new(move |msg: Message| {
            set_local_messages_rc.update(|msgs| msgs.push(msg));
        })
    };
    // Hook that returns all messages (socket + local), deduplicated and sorted
    let messages = use_group_socket_messages(
        group_data.membership.group_chat_id,
        ws_ctx.as_ref().and_then(|w| w.as_ref().cloned()),
        local_messages,
    );
    let (dropdown_state, set_dropdown_state) = create_signal(DropdownState::Closed);
    let (invite_modal_open, set_invite_modal_open) = create_signal(false);
    let (group_details_modal_open, set_group_details_modal_open) = create_signal(false);
    let (leave_modal_open, set_leave_modal_open) = create_signal(false);
    let (leave_error, set_leave_error) = create_signal(None::<String>);
    let (leave_loading, set_leave_loading) = create_signal(false);

    let dropdown_ref = create_node_ref::<Div>();
    
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
                // If already closing, do nothing
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
                style=move || format!("{};padding-bottom:72px;", bg_url.get())
            >
                {move || {
                    let storage_service = StorageService::new();
                    let user_profile = storage_service.get_user_profile();
                    let msgs = messages.get();
                    msgs.iter().enumerate().map(|(_idx, msg)| {
                        let (is_own, sender_username, sender_name, sender_surname) = if let Some(ref user) = user_profile {
                            if msg.sender_id == user.id {
                                (true, user.username.clone(), user.first_name.clone(), user.last_name.clone())
                            } else if let Some(sender) = user_cache.get().get(&msg.sender_id) {
                                (false, sender.username.clone(), sender.first_name.clone(), sender.last_name.clone())
                            } else {
                                (false, "?".to_string(), "".to_string(), "".to_string())
                            }
                        } else {
                            (false, "?".to_string(), "".to_string(), "".to_string())
                        };
                        view! {
                            <ChatMessage
                                message=msg.clone()
                                sender_username=sender_username
                                sender_name=sender_name
                                sender_surname=sender_surname
                                status=MessageStatus::Delivered
                                is_own=is_own
                            />
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
