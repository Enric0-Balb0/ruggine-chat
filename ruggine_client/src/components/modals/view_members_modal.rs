use leptos::*;
use leptos::wasm_bindgen::JsCast;
use crate::types::user::{UserProfile, UserStatus, UserType, Gender, CurrentAction};
use crate::components::{modals::invite_member_modal::MemberRole, ui::{UserAvatar, LucideIcon}};
use chrono::{DateTime, Utc, NaiveDate};

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

// Static data for demo
fn create_mock_members() -> Vec<GroupMember> {
    vec![
        GroupMember {
            user_profile: UserProfile {
                id: 1,
                email: "mario.rossi@email.com".to_string(),
                first_name: "Mario".to_string(),
                last_name: "Rossi".to_string(),
                username: "mario_rossi".to_string(),
                birthday: NaiveDate::from_ymd_opt(1990, 5, 15).unwrap_or_default(),
                address: "Roma, Italia".to_string(),
                gender: Gender::Male,
                user_type: UserType::EndUser,
                user_status: UserStatus::Active,
                current_action: CurrentAction::Waiting,
                is_online: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: Some(Utc::now()),
            },
            role: MemberRole::Admin,
            joined_at: Utc::now(),
            is_creator: true,
        },
        GroupMember {
            user_profile: UserProfile {
                id: 2,
                email: "giulia.bianchi@email.com".to_string(),
                first_name: "Giulia".to_string(),
                last_name: "Bianchi".to_string(),
                username: "giulia_b".to_string(),
                birthday: NaiveDate::from_ymd_opt(1992, 8, 22).unwrap_or_default(),
                address: "Milano, Italia".to_string(),
                gender: Gender::Female,
                user_type: UserType::EndUser,
                user_status: UserStatus::Active,
                current_action: CurrentAction::Writing,
                is_online: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: Some(Utc::now()),
            },
            role: MemberRole::Member,
            joined_at: Utc::now(),
            is_creator: false,
        },
        GroupMember {
            user_profile: UserProfile {
                id: 3,
                email: "luca.verdi@email.com".to_string(),
                first_name: "Luca".to_string(),
                last_name: "Verdi".to_string(),
                username: "luca_v".to_string(),
                birthday: NaiveDate::from_ymd_opt(1988, 12, 3).unwrap_or_default(),
                address: "Napoli, Italia".to_string(),
                gender: Gender::Male,
                user_type: UserType::EndUser,
                user_status: UserStatus::Active,
                current_action: CurrentAction::Waiting,
                is_online: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: Some(Utc::now()),
            },
            role: MemberRole::Member,
            joined_at: Utc::now(),
            is_creator: false,
        },
        GroupMember {
            user_profile: UserProfile {
                id: 4,
                email: "anna.ferrari@email.com".to_string(),
                first_name: "Anna".to_string(),
                last_name: "Ferrari".to_string(),
                username: "anna_ferrari".to_string(),
                birthday: NaiveDate::from_ymd_opt(1995, 3, 18).unwrap_or_default(),
                address: "Torino, Italia".to_string(),
                gender: Gender::Female,
                user_type: UserType::EndUser,
                user_status: UserStatus::Active,
                current_action: CurrentAction::Writing,
                is_online: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: Some(Utc::now()),
            },
            role: MemberRole::Admin,
            joined_at: Utc::now(),
            is_creator: false,
        },
        GroupMember {
            user_profile: UserProfile {
                id: 5,
                email: "francesco.neri@email.com".to_string(),
                first_name: "Francesco".to_string(),
                last_name: "Neri".to_string(),
                username: "fra_neri".to_string(),
                birthday: NaiveDate::from_ymd_opt(1993, 7, 9).unwrap_or_default(),
                address: "Firenze, Italia".to_string(),
                gender: Gender::Male,
                user_type: UserType::EndUser,
                user_status: UserStatus::Suspended,
                current_action: CurrentAction::Waiting,
                is_online: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login: Some(Utc::now()),
            },
            role: MemberRole::Member,
            joined_at: Utc::now(),
            is_creator: false,
        },
    ]
}

#[component]
pub fn ViewMembersModal(
    #[prop(into)] is_open: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into, optional)] group_name: Option<String>,
) -> impl IntoView {
    // Store group name in a signal to avoid move issues
    let group_name_signal = create_signal(group_name.clone()).0;
    
    // Mock members data
    let members = create_mock_members();
    let members_signal = create_signal(members).0;

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
                            class="bg-white dark:bg-surface-dark shadow-2xl dark:shadow-black/50 border border-border dark:border-border-dark rounded-lg modal-container overflow-hidden"
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
                                    "Membri del gruppo"
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

                            // Group info and stats
                            <div class="mb-6 flex-shrink-0">
                                <div class="p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                                    <div class="flex items-start gap-3">
                                        <LucideIcon name="users" size=20 class="text-blue-500 dark:text-blue-400 mt-0.5" />
                                        <div class="flex-1">
                                            <p class="m-0 text-sm text-text-primary dark:text-text-primary-dark leading-relaxed">
                                                <strong class="font-medium">
                                                    {move || {
                                                        if let Some(ref name) = group_name_signal.get() {
                                                            format!("Membri di \"{}\"", name)
                                                        } else {
                                                            "Membri del gruppo".to_string()
                                                        }
                                                    }}
                                                </strong>
                                                <br/>
                                                <span class="text-text-secondary dark:text-text-secondary-dark">
                                                    {move || format!("{} membri totali - {} online - {} amministratori", 
                                                        total_count.get(), online_count.get(), admin_count.get()
                                                    )}
                                                </span>
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // Members List - scrollable con pattern di registration.rs
                            <div class="mb-6">
                                <div class="max-h-96 overflow-y-auto custom-scrollbar pr-2">
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
                                                            // Status dot posizionato sull'avatar
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
