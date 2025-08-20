use leptos::*;
use leptos::html::Div;
use wasm_bindgen::JsCast;
use crate::hooks::GroupMembershipWithDetails;
use crate::components::{InviteMemberModal, InviteMemberRequest, ViewMembersModal, LucideIcon};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChatHeaderAction {
    InviteMembers,
    ViewMembers,
    GroupDetails,
    LeaveGroup,
}

#[component]
pub fn ChatView(
    #[prop(into)] group_data: GroupMembershipWithDetails,
) -> impl IntoView {
    let (dropdown_open, set_dropdown_open) = create_signal(false);
    let (invite_modal_open, set_invite_modal_open) = create_signal(false);
    let (view_members_modal_open, set_view_members_modal_open) = create_signal(false);
    
    let dropdown_ref = create_node_ref::<Div>();
    
    // Effect per chiudere il dropdown quando si clicca fuori
    create_effect(move |_| {
        let dropdown_is_open = dropdown_open.get();
        if dropdown_is_open {
            let handle_click_outside = move |event: web_sys::Event| {
                if let Some(dropdown_element) = dropdown_ref.get() {
                    if let Some(target) = event.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                            if !dropdown_element.contains(Some(&element)) {
                                set_dropdown_open.set(false);
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
        set_dropdown_open.set(false);
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
    
    view! {
        <div class="flex flex-col h-full bg-white dark:bg-surface-dark">
            // Chat Header - usando solo Tailwind
            <div class="px-6 py-4 border-b border-gray-200 dark:border-border-dark bg-gray-50 dark:bg-surface-dark flex justify-between items-center">
                <div class="flex-1">
                    <h2 class="text-xl font-semibold text-gray-800 dark:text-text-primary-dark mb-1">
                        {group_name.clone()}
                    </h2>
                    <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-text-secondary-dark">
                        <div class="w-2 h-2 bg-green-500 rounded-full"></div>
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
                        <span>"Ultimo accesso ora"</span>
                    </div>
                </div>
                <div class="flex items-center gap-2 relative" node_ref=dropdown_ref>
                    <button 
                        class="bg-white dark:bg-surface-dark border border-gray-300 dark:border-gray-400 text-gray-700 dark:text-text-primary-dark px-3 py-1.5 rounded text-xs flex items-center gap-1 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors shadow-sm dark:shadow-gray-800/20"
                        on:click=move |_| set_dropdown_open.update(|open| *open = !*open)
                    >
                        <span>"Opzioni gruppo"</span>
                        <span>"⋮"</span>
                    </button>
                    <div 
                        class=move || {
                            let base_classes = "absolute top-full right-0 mt-1 bg-white dark:bg-surface-dark border border-gray-200 dark:border-border-dark rounded shadow-lg dark:shadow-black/50 z-50 min-w-48";
                            if dropdown_open.get() {
                                format!("{} block", base_classes)
                            } else {
                                format!("{} hidden", base_classes)
                            }
                        }
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

            // Content area - per ora vuoto
            <div class="flex-1 bg-white dark:bg-surface-dark">
                // Area vuota per ora - in futuro conterrà i messaggi
            </div>

            // Invite Member Modal
            <InviteMemberModal
                is_open=invite_modal_open
                on_close=handle_invite_modal_close
                on_invite=handle_invite_member
                group_name=group_name.clone()
            />

            // View Members Modal
            <ViewMembersModal
                is_open=view_members_modal_open
                on_close=handle_view_members_modal_close
                group_name=group_name.clone()
            />
        </div>
    }
}
