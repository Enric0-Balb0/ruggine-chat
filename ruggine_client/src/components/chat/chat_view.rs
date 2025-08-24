use leptos::*;
use leptos::html::Div;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
// use web_sys::MutationObserver;
use crate::hooks::GroupMembershipWithDetails;
use crate::components::{InviteMemberModal, InviteMemberRequest, MessageInputArea, GroupDetailsModal, LucideIcon};
use leptos::use_context;
use crate::components::chat::chat_message::{ChatMessage, MessageStatus};
use crate::types::message::Message;
use chrono::{TimeZone, Utc};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChatHeaderAction {
    InviteMembers,
    ViewMembers,
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
    let (dropdown_state, set_dropdown_state) = create_signal(DropdownState::Closed);
    let (invite_modal_open, set_invite_modal_open) = create_signal(false);
    let (view_members_modal_open, set_view_members_modal_open) = create_signal(false);
    
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
            ChatHeaderAction::ViewMembers => {
                set_view_members_modal_open.set(true);
                logging::log!("Opening members modal");
            }
            ChatHeaderAction::GroupDetails => {
                logging::log!("Opening group details modal");
            }
            ChatHeaderAction::LeaveGroup => {
                logging::log!("Showing leave confirmation");
            }
        }
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

    let handle_view_members_modal_close = move |_| {
        set_view_members_modal_open.set(false);
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
                            on:click=move |_| handle_header_action(ChatHeaderAction::ViewMembers)
                        >
                            <LucideIcon name="users" size=16 />
                            <span>"Visualizza membri"</span>
                        </div>
                        <div 
                            class="px-4 py-2 text-sm text-gray-700 dark:text-text-primary-dark hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer flex items-center gap-2 border-b border-gray-100 dark:border-border-dark"
                            on:click=move |_| handle_header_action(ChatHeaderAction::GroupDetails)
                        >
                            <LucideIcon name="settings" size=16 />
                            <span>"Impostazioni gruppo"</span>
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
            <div class="flex-1 flex flex-col">
                {/* Qui andranno i messaggi */}
                <div
                    class="flex-1 overflow-y-auto px-12 py-4 space-y-4"
                    style=move || bg_url.get()
                >
                    {/* Esempio messaggio di sistema */}
                    <div class="text-center text-gray-500 text-xs italic py-2 bg-gray-50 dark:bg-gray-800 rounded-md border border-gray-200 dark:border-gray-700 mx-auto max-w-[80%] shadow-sm">
                        Benvenuto nella chat di gruppo!
                    </div>
                    {/* Messaggi statici di esempio */}
                    <ChatMessage
                        message=Message {
                            id: 1,
                            content: "Ciao a tutti! Questo è un messaggio di esempio.".to_string(),
                            sender_id: 42,
                            group_chat_id: group_data.membership.group_chat_id,
                            sent_at: Utc.ymd(2025, 8, 24).and_hms(15, 30, 0),
                        }
                        sender_username="alice".to_string()
                        sender_name="Alice".to_string()
                        sender_surname="Rossi".to_string()
                        status=MessageStatus::Delivered
                        is_own=false
                    />
                    <ChatMessage
                        message=Message {
                            id: 2,
                            content: "Messaggio inviato da me!".to_string(),
                            sender_id: 99,
                            group_chat_id: group_data.membership.group_chat_id,
                            sent_at: Utc.ymd(2025, 8, 24).and_hms(15, 31, 0),
                        }
                        sender_username="io".to_string()
                        sender_name="Enrico".to_string()
                        sender_surname="Bianchi".to_string()
                        status=MessageStatus::Sent
                        is_own=true
                    />
                </div>

                <MessageInputArea />

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
                is_open=view_members_modal_open
                on_close=handle_view_members_modal_close
                group_id=group_data_clone.membership.group_chat_id
            />
        </div>
    }
}
