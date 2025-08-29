use leptos::*;
use leptos::wasm_bindgen::JsCast;
use crate::types::user::{UserProfile, UserStatus, UserType, Gender};
use crate::api::services::UserService;
use crate::components::{ UserAvatar, LucideIcon};
use crate::types::invitation::MemberRole;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct GroupMember {
    pub user_profile: UserProfile,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
    pub is_creator: bool,
}

impl GroupMember {
    pub fn display_name(&self) -> String {
        format!("{} {}", self.user_profile.first_name, self.user_profile.last_name)
    }

    pub fn status_color(&self) -> &'static str {
        if self.user_profile.is_online {
            "bg-green-500"
        } else {
            "bg-gray-400"
        }
    }

    pub fn role_badge_color(&self) -> &'static str {
        match self.role {
            MemberRole::Admin => "bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-300",
            MemberRole::Member => "bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-300",
        }
    }
}


#[component]
pub fn GroupDetailsModal(
    #[prop(into)] is_open: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] group_id: i32,
) -> impl IntoView {
    let (members_signal, set_members_signal) = create_signal(Vec::<GroupMember>::new());
    let (group_signal, set_group_signal) = create_signal(None::<crate::types::group::GroupChat>);

    let storage_service = crate::utils::storage::StorageService::new();
    let http_client = crate::api::client::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
    if let Some(token_response) = storage_service.get_token() {
        http_client.set_auth_token(Some(token_response.token));
    }
    let membership_service = crate::api::services::GroupMembershipService::new(http_client.clone(), storage_service.clone());
    let user_service = UserService::new(http_client.clone(), storage_service.clone());
    let group_service = crate::api::services::GroupChatService::new(http_client, storage_service);

    create_effect(move |_| {
        if is_open.get() {
            let group_id = group_id;
            let set_members_signal = set_members_signal.clone();
            let set_group_signal = set_group_signal.clone();
            let membership_service = membership_service.clone();
            let user_service = user_service.clone();
            let group_service = group_service.clone();
            spawn_local(async move {
                // Fetch group info
                match group_service.get_group_by_id(&group_id.to_string()).await {
                    Ok(group) => set_group_signal.set(Some(group)),
                    Err(_) => set_group_signal.set(None),
                }
                // Fetch memberships
                match membership_service.get_by_group_chat_id(&group_id.to_string()).await {
                    Ok(memberships) => {
                        let mut group_members = Vec::with_capacity(memberships.len());
                        let mut tasks = Vec::with_capacity(memberships.len());
                        for m in memberships {
                            let user_service = user_service.clone();
                            let user_id = m.user_id;
                            let role = m.role;
                            let joined_at = m.joined_at;
                            tasks.push(async move {
                                let user_profile = match user_service.get_user_by_id(&user_id.to_string()).await {
                                    Ok(profile) => UserProfile {
                                        id: user_id,
                                        email: profile.email,
                                        first_name: profile.first_name,
                                        last_name: profile.last_name,
                                        username: profile.username,
                                        birthday: chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                                        address: String::new(),
                                        gender: Gender::Other,
                                        user_type: UserType::EndUser,
                                        user_status: UserStatus::Active,
                                        current_action: crate::types::membership::CurrentAction::Waiting,
                                        is_online: false,
                                        created_at: chrono::Utc::now(),
                                        updated_at: chrono::Utc::now(),
                                        last_login: None,
                                    },
                                    Err(_) => UserProfile {
                                        id: user_id,
                                        email: String::new(),
                                        first_name: String::from(""),
                                        last_name: String::from(""),
                                        username: String::from(""),
                                        birthday: chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                                        address: String::new(),
                                        gender: Gender::Other,
                                        user_type: UserType::EndUser,
                                        user_status: UserStatus::Active,
                                        current_action: crate::types::membership::CurrentAction::Waiting,
                                        is_online: false,
                                        created_at: chrono::Utc::now(),
                                        updated_at: chrono::Utc::now(),
                                        last_login: None,
                                    }
                                };
                                GroupMember {
                                    user_profile,
                                    role,
                                    joined_at,
                                    is_creator: false,
                                }
                            });
                        }
                        let results = futures::future::join_all(tasks).await;
                        group_members.extend(results);
                        set_members_signal.set(group_members);
                    }
                    Err(_e) => {
                        set_members_signal.set(Vec::new());
                    }
                }
            });
        }
    });

    // Animation states
    let (is_visible, set_is_visible) = create_signal(false);
    let (is_animating_in, set_is_animating_in) = create_signal(false);

    // Handle modal open/close with animations
    create_effect(move |_| {
        let is_modal_open = is_open.get();
        
        if is_modal_open {
            set_is_visible.set(true);
            set_timeout(
                move || {
                    set_is_animating_in.set(true);
                },
                std::time::Duration::from_millis(10),
            );
        } else {
            set_is_animating_in.set(false);
            set_timeout(
                move || {
                    set_is_visible.set(false);
                },
                std::time::Duration::from_millis(250),
            );
        }
    });

    // Handle close
    let handle_close = move |_| {
        on_close.call(());
    };

    // Handle backdrop click
    let handle_backdrop_click = move |e: web_sys::MouseEvent| {
        if let Some(target) = e.target() {
            if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                if element.class_list().contains("modal-backdrop") {
                    handle_close(());
                }
            }
        }
    };

    // Calculate stats
    let online_count = create_memo(move |_| {
        members_signal.get().iter().filter(|m| m.user_profile.is_online).count()
    });
    let admin_count = create_memo(move |_| {
        members_signal.get().iter().filter(|m| m.role == MemberRole::Admin).count()
    });
    let total_count = create_memo(move |_| members_signal.get().len());

    view! {
        {move || {
            if is_visible.get() {
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
                            class="bg-white dark:bg-surface-dark shadow-2xl pb-8 dark:shadow-black/50 border border-border dark:border-border-dark rounded-lg modal-container overflow-hidden"
                            style=move || {
                                let base_style = "width: 100%; max-width: 600px; margin: 0 20px; padding: 24px; display: flex; flex-direction: column; max-height: 80vh;";
                                if is_animating_in.get() {
                                    format!("{}transform: scale(1) translateY(0px); opacity: 1; transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);", base_style)
                                } else {
                                    format!("{}transform: scale(0.9) translateY(-20px); opacity: 0; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);", base_style)
                                }
                            }
                        >
                            // Header
                            <div class="flex items-center justify-between mb-4 flex-shrink-0">
                                <h2 class="text-xl font-semibold text-text-primary dark:text-text-primary-dark m-0">
                                    {move || group_signal.get().as_ref().map(|g| g.name.clone()).unwrap_or_else(|| "Dettagli gruppo".to_string())}
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

                            // Group description
                            <Show when=move || group_signal.get().is_some()>
                                <div class="mb-4 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                                    <div class="text-text-primary dark:text-text-primary-dark text-base">
                                        {move || group_signal.get().as_ref().map(|g| g.description.clone()).unwrap_or_default()}
                                    </div>
                                </div>
                            </Show>

                            // Group info and stats
                            <div class="mb-6 flex-shrink-0">
                                <div class="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                                    <div class="flex items-start gap-3">
                                        <LucideIcon name="users" size=20 class="text-blue-500 dark:text-blue-400 mt-0.5" />
                                        <div class="flex-1">
                                            <p class="m-0 text-sm text-text-primary dark:text-text-primary-dark leading-relaxed">
                                                <strong class="font-medium">
                                                    {move || {
                                                        if let Some(ref group) = group_signal.get() {
                                                            format!("Membri di \"{}\"", group.name)
                                                        } else {
                                                            "Membri del gruppo".to_string()
                                                        }
                                                    }}
                                                </strong>
                                                <br/>
                                                <span class="text-text-secondary dark:text-text-secondary-dark">
                                                    {move || format!("{} membri - {} online - {} admin", 
                                                        total_count.get(), online_count.get(), admin_count.get()
                                                    )}
                                                </span>
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // Members List - scrollable
                            <div class="mb-6">
                                <div class="max-h-96 overflow-y-auto custom-scrollbar mb-8 pr-2">
                                    <div class="space-y-3">
                                        {move || {
                                            members_signal.get().into_iter().map(|member| {
                                                view! {
                                                    <div class="flex items-center gap-4 p-4 bg-white dark:bg-surface-dark border border-border dark:border-border-dark rounded-lg hover:shadow-md dark:hover:shadow-black/20 transition-all duration-200">
                                                        // Avatar con status dot usando UserAvatar component
                                                        <div class="relative flex-shrink-0">
                                                            <UserAvatar 
                                                                name={member.user_profile.first_name.clone()}
                                                                surname={member.user_profile.last_name.clone()}
                                                                username={member.user_profile.username.clone()}
                                                                size="xl"
                                                            />
                                                            <div class=format!("absolute top-8 -right-1 w-4 h-4 {} rounded-full border-2 border-white dark:border-surface-dark", member.status_color())></div>
                                                        </div>
                                                    
                                                    // User info
                                                    <div class="flex-1 min-w-0">
                                                        <div class="flex items-center gap-3 mb-1">
                                                            <h3 class="font-semibold text-text-primary dark:text-text-primary-dark text-base truncate">
                                                                {member.display_name()}
                                                            </h3>
                                                            <div class=format!("px-2 py-1 rounded-full text-xs font-medium {}", member.role_badge_color())>
                                                                {member.role.display_name()}
                                                            </div>
                                                            {if member.is_creator {
                                                                view! {
                                                                    <div class="px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-300">
                                                                        "Creatore"
                                                                    </div>
                                                                }.into_view()
                                                            } else {
                                                                view! {}.into_view()
                                                            }}
                                                        </div>
                                                        <div class="flex items-center gap-4 text-sm text-text-secondary dark:text-text-secondary-dark">
                                                            <span class="flex items-center gap-1">
                                                                <span>"@"</span>
                                                                <span class="font-mono text-text-primary dark:text-text-primary-dark">{member.user_profile.username.clone()}</span>
                                                            </span>
                                                        </div>
                                                    </div>
                                                    
                                                    // Actions dropdown placeholder
                                                    <div class="flex items-center gap-2">
                                                        <button class="p-2 text-text-secondary dark:text-text-secondary-dark hover:text-text-primary dark:hover:text-text-primary-dark hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors">
                                                            <LucideIcon name="more-horizontal" size=16 />
                                                        </button>
                                                    </div>
                                                </div>
                                            }.into_view()
                                        }).collect::<Vec<_>>()
                                    }}
                                    </div>
                                </div>
                            </div>

                            // Footer with actions
                            <div class="flex justify-between items-center pt-4 border-t border-border dark:border-border-dark flex-shrink-0">
                                <div class="text-sm text-text-secondary dark:text-text-secondary-dark">
                                    {move || format!("Visualizzando {} di {} membri", total_count.get(), total_count.get())}
                                </div>
                                <button
                                    type="button"
                                    class="px-4 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark font-medium cursor-pointer transition-colors hover:bg-gray-50 dark:hover:bg-gray-700"
                                    on:click=move |_| handle_close(())
                                >
                                    "Chiudi"
                                </button>
                            </div>
                            
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }
        }}
    }
}