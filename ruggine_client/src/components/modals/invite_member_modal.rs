use leptos::*;
use leptos::wasm_bindgen::JsCast;
use crate::components::LucideIcon;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemberRole {
    Member,
    Admin,
}

impl MemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberRole::Member => "member",
            MemberRole::Admin => "admin",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            MemberRole::Member => "Membro",
            MemberRole::Admin => "Amministratore",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MemberRole::Member => "Può partecipare alle conversazioni",
            MemberRole::Admin => "Può gestire il gruppo e invitare altri membri",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InviteMemberRequest {
    pub username: String,
    pub role: MemberRole,
}

#[component]
pub fn InviteMemberModal(
    #[prop(into)] is_open: ReadSignal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_invite: Callback<InviteMemberRequest>,
    #[prop(into, optional)] is_loading: Option<ReadSignal<bool>>,
    #[prop(into, optional)] group_name: Option<String>,
) -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (selected_role, set_selected_role) = create_signal(MemberRole::Member);
    let (error_message, set_error_message) = create_signal(Option::<String>::None);

    // Store group name in a signal to avoid move issues
    let group_name_signal = create_signal(group_name.clone()).0;

    // Use external loading state if provided, otherwise use internal state
    let (_internal_is_inviting, _set_internal_is_inviting) = create_signal(false);
    let is_inviting = if let Some(external_loading) = is_loading {
        external_loading
    } else {
        _internal_is_inviting.into()
    };

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

    // Form validation
    let is_form_valid = create_memo(move |_| {
        let username = username.get();
        !username.trim().is_empty() && username.len() >= 3 && username.len() <= 50
    });

    // Handle form submission
    let handle_submit = move |_| {
        if !is_form_valid.get() || is_inviting.get() {
            return;
        }

        let username_value = username.get().trim().to_string();

        // Validate username length
        if username_value.len() < 3 {
            set_error_message.set(Some("L'username deve essere almeno di 3 caratteri".to_string()));
            return;
        }

        if username_value.len() > 50 {
            set_error_message.set(Some("L'username non può superare i 50 caratteri".to_string()));
            return;
        }

        // Basic username validation (alphanumeric and underscore)
        if !username_value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            set_error_message.set(Some("L'username può contenere solo lettere, numeri, _ e -".to_string()));
            return;
        }

        set_error_message.set(None);

        let invite_request = InviteMemberRequest {
            username: username_value,
            role: selected_role.get(),
        };

        on_invite.call(invite_request);
    };

    // Handle close
    let handle_close = move |_| {
        // Reset form
        set_username.set(String::new());
        set_selected_role.set(MemberRole::Member);
        set_error_message.set(None);
        
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

    // Character count
    let username_count = create_memo(move |_| username.get().len());

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
                            class="bg-white dark:bg-surface-dark shadow-2xl dark:shadow-black/50 border border-border dark:border-border-dark rounded-lg modal-container"
                            style=move || {
                                let base_style = "width: 100%; max-width: 480px; margin: 0 20px; padding: 24px;";
                                if is_animating_in.get() {
                                    format!("{}transform: scale(1) translateY(0px); opacity: 1; transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);", base_style)
                                } else {
                                    format!("{}transform: scale(0.9) translateY(-20px); opacity: 0; transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);", base_style)
                                }
                            }
                        >
                            // Header
                            <div class="flex items-center justify-between mb-4">
                                <h2 class="text-xl font-semibold text-text-primary dark:text-text-primary-dark m-0">
                                    "Invita Nuovo Membro"
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

                            // Group info and description
                            <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                                <div class="flex items-start gap-3">
                                    <LucideIcon name="user-plus" size=20 class="text-blue-500 dark:text-blue-400 mt-0.5" />
                                    <div class="flex-1">
                                        <p class="m-0 text-sm text-blue-800 dark:text-blue-200 leading-relaxed">
                                            <strong class="font-medium">
                                                {move || {
                                                    if let Some(ref name) = group_name_signal.get() {
                                                        format!("Invita un nuovo membro in \"{}\"", name)
                                                    } else {
                                                        "Invita un nuovo membro".to_string()
                                                    }
                                                }}
                                            </strong>
                                            <br/>
                                            "Cerca l'utente per username e seleziona il suo ruolo nel gruppo."
                                        </p>
                                    </div>
                                </div>
                            </div>

                            // Error message
                            {move || error_message.get().map(|msg| view! {
                                <div 
                                    class="mb-5 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg"
                                    style="transform: translateY(0px); opacity: 1; animation: slideInError 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);"
                                >
                                    <p class="m-0 text-sm text-red-800 dark:text-red-200 flex items-center gap-2">
                                        <LucideIcon name="alert-triangle" size=16 class="text-red-600 dark:text-red-400" />
                                        {msg}
                                    </p>
                                </div>
                            })}

                            // Form
                            <form 
                                on:submit=move |e| {
                                    e.prevent_default();
                                    handle_submit(());
                                }
                            >
                                // Username Field
                                <div class="mb-6">
                                    <label 
                                        for="username" 
                                        class="block text-sm font-semibold text-text-primary dark:text-text-primary-dark mb-2"
                                    >
                                        "Username dell'utente"
                                    </label>
                                    <div class="relative">
                                        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                            <LucideIcon name="at-sign" size=16 class="text-gray-400" />
                                        </div>
                                        <input
                                            type="text"
                                            id="username"
                                            class="w-full pl-10 pr-3 py-3 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark focus:border-transparent transition-colors hover:border-accent/50 dark:hover:border-accent-dark/50"
                                            placeholder="es. mario_rossi"
                                            prop:value=move || username.get()
                                            on:input=move |e| {
                                                set_username.set(event_target_value(&e));
                                                set_error_message.set(None);
                                            }
                                            maxlength="50"
                                            required
                                        />
                                    </div>
                                    <div class="flex justify-between mt-2 text-xs">
                                        <span class="text-text-secondary dark:text-text-secondary-dark">
                                            "Solo lettere, numeri, _ e -"
                                        </span>
                                        <span class=move || {
                                            let count = username_count.get();
                                            let base_class = "font-medium transition-colors duration-200 ";
                                            if count > 45 {
                                                format!("{}text-red-500", base_class)
                                            } else if count > 35 {
                                                format!("{}text-orange-500", base_class)
                                            } else {
                                                format!("{}text-text-secondary dark:text-text-secondary-dark", base_class)
                                            }
                                        }>
                                            {move || format!("{}/50", username_count.get())}
                                        </span>
                                    </div>
                                </div>

                                // Role Selection
                                <div class="mb-6">
                                    <label class="block text-sm font-semibold text-text-primary dark:text-text-primary-dark mb-3">
                                        "Ruolo nel gruppo"
                                    </label>
                                    <div class="space-y-3">
                                        // Member Role Option
                                        <div 
                                            class=move || {
                                                let base_classes = "p-4 border rounded-lg cursor-pointer transition-all duration-200 ";
                                                if selected_role.get() == MemberRole::Member {
                                                    format!("{}border-accent dark:border-accent-dark bg-accent/5 dark:bg-accent-dark/5", base_classes)
                                                } else {
                                                    format!("{}border-border dark:border-border-dark bg-white dark:bg-surface-dark hover:border-accent/50 dark:hover:border-accent-dark/50", base_classes)
                                                }
                                            }
                                            on:click=move |_| set_selected_role.set(MemberRole::Member)
                                        >
                                            <div class="flex items-start gap-3">
                                                <div class=move || {
                                                    let base_classes = "w-4 h-4 rounded-full border-2 mt-0.5 transition-colors duration-200 ";
                                                    if selected_role.get() == MemberRole::Member {
                                                        format!("{}border-accent dark:border-accent-dark bg-accent dark:bg-accent-dark", base_classes)
                                                    } else {
                                                        format!("{}border-gray-300 dark:border-gray-600", base_classes)
                                                    }
                                                }>
                                                    {move || if selected_role.get() == MemberRole::Member {
                                                        view! {
                                                            <div class="w-2 h-2 bg-white rounded-full m-0.5"></div>
                                                        }.into_view()
                                                    } else {
                                                        view! {}.into_view()
                                                    }}
                                                </div>
                                                <div class="flex-1">
                                                    <div class="flex items-center gap-2 mb-1">
                                                        <LucideIcon name="user" size=16 class="text-blue-500 dark:text-blue-400" />
                                                        <h3 class="text-sm font-medium text-text-primary dark:text-text-primary-dark m-0">
                                                            {MemberRole::Member.display_name()}
                                                        </h3>
                                                    </div>
                                                    <p class="text-xs text-text-secondary dark:text-text-secondary-dark m-0 leading-relaxed">
                                                        {MemberRole::Member.description()}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>

                                        // Admin Role Option
                                        <div 
                                            class=move || {
                                                let base_classes = "p-4 border rounded-lg cursor-pointer transition-all duration-200 ";
                                                if selected_role.get() == MemberRole::Admin {
                                                    format!("{}border-accent dark:border-accent-dark bg-accent/5 dark:bg-accent-dark/5", base_classes)
                                                } else {
                                                    format!("{}border-border dark:border-border-dark bg-white dark:bg-surface-dark hover:border-accent/50 dark:hover:border-accent-dark/50", base_classes)
                                                }
                                            }
                                            on:click=move |_| set_selected_role.set(MemberRole::Admin)
                                        >
                                            <div class="flex items-start gap-3">
                                                <div class=move || {
                                                    let base_classes = "w-4 h-4 rounded-full border-2 mt-0.5 transition-colors duration-200 ";
                                                    if selected_role.get() == MemberRole::Admin {
                                                        format!("{}border-accent dark:border-accent-dark bg-accent dark:bg-accent-dark", base_classes)
                                                    } else {
                                                        format!("{}border-gray-300 dark:border-gray-600", base_classes)
                                                    }
                                                }>
                                                    {move || if selected_role.get() == MemberRole::Admin {
                                                        view! {
                                                            <div class="w-2 h-2 bg-white rounded-full m-0.5"></div>
                                                        }.into_view()
                                                    } else {
                                                        view! {}.into_view()
                                                    }}
                                                </div>
                                                <div class="flex-1">
                                                    <div class="flex items-center gap-2 mb-1">
                                                        <LucideIcon name="crown" size=16 class="text-purple-500 dark:text-purple-400" />
                                                        <h3 class="text-sm font-medium text-text-primary dark:text-text-primary-dark m-0">
                                                            {MemberRole::Admin.display_name()}
                                                        </h3>
                                                    </div>
                                                    <p class="text-xs text-text-secondary dark:text-text-secondary-dark m-0 leading-relaxed">
                                                        {MemberRole::Admin.description()}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // Action Buttons
                                <div class="flex gap-3 mt-6">
                                    <button
                                        type="button"
                                        class="flex-1 px-4 py-2.5 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark font-medium cursor-pointer transition-colors hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                                        on:click=move |_| handle_close(())
                                        disabled=move || is_inviting.get()
                                    >
                                        "Annulla"
                                    </button>
                                    <button
                                        type="submit"
                                        class=move || {
                                            let base_classes = "flex-1 px-4 py-2.5 text-sm rounded-md font-medium cursor-pointer transition-colors border-none flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed";
                                            if is_form_valid.get() && !is_inviting.get() {
                                                format!("{} bg-brand-primary dark:bg-brand-primary-dark hover:bg-brand-primary-light dark:hover:bg-brand-primary text-white", base_classes)
                                            } else {
                                                format!("{} bg-gray-300 dark:bg-gray-600 text-gray-500 dark:text-gray-400", base_classes)
                                            }
                                        }
                                        disabled=move || !is_form_valid.get() || is_inviting.get()
                                    >
                                        {move || if is_inviting.get() {
                                            view! {
                                                <span>"Invitando..."</span>
                                            }.into_view()
                                        } else {
                                            view! { 
                                                <span>"Invia Invito"</span>
                                            }.into_view()
                                        }}
                                    </button>
                                </div>
                            </form>
                            
                            // CSS personalizzato per animazioni
                            <style>
                                "
                                @keyframes slideInError {
                                    0% {
                                        transform: translateY(-10px) scale(0.95);
                                        opacity: 0;
                                    }
                                    50% {
                                        transform: translateY(2px) scale(1.02);
                                    }
                                    100% {
                                        transform: translateY(0px) scale(1);
                                        opacity: 1;
                                    }
                                }
                                
                                .modal-backdrop {
                                    will-change: opacity, backdrop-filter;
                                }
                                
                                .modal-container {
                                    will-change: transform, opacity;
                                }
                                "
                            </style>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! { <></> }.into_view()
            }
        }
    }
    }
}