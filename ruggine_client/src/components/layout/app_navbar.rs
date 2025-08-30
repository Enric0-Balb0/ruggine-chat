use leptos::*;
use leptos_router::*;
use crate::components::{UserAvatar, ThemeSlider, LucideIcon, IconSize, use_toast};
use crate::api::services::AuthService;
use crate::utils::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;

/// Main app navbar for authenticated users (based on UI mock)
#[component]
pub fn AppNavbar() -> impl IntoView {
    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    );

    // Toast for user feedback
    let toast = use_toast();
    let navigate = use_navigate();

    // Recupera i dati dell'utente corrente
    let user_profile = auth_service.get_current_user();
    let is_admin = user_profile.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    
    // State for hamburger menu dropdown
    let (is_menu_open, set_is_menu_open) = create_signal(false);
    
    // Toggle menu
    let toggle_menu = move |_| {
        set_is_menu_open.update(|open| *open = !*open);
    };
    
    // Close menu when clicking outside
    let close_menu = move |_| {
        set_is_menu_open.set(false);
    };
    
    // Prevent menu from closing when clicking inside the menu
    let prevent_close = move |event: leptos::ev::MouseEvent| {
        event.stop_propagation();
    };
    
    // Handle menu item clicks
    let handle_logout = {
        let auth_service = auth_service.clone();
        let toast = toast.clone();
        let navigate = navigate.clone();
        let set_is_menu_open = set_is_menu_open;
        
    Callback::new(move |_: leptos::ev::MouseEvent| {
            set_is_menu_open.set(false);
            let auth_service = auth_service.clone();
            let toast = toast.clone();
            let navigate = navigate.clone();
            
            spawn_local(async move {
                match auth_service.logout().await {
                    Ok(_) => {
                        toast.success("Logout effettuato con successo!");
                        navigate("/login", Default::default());
                    },
                    Err(error) => {
                        leptos::logging::error!("Logout failed: {:?}", error);
                        toast.error("Errore durante il logout. Riprova.");
                    }
                }
            });
        })
    };

    let handle_settings = {
        let toast = toast.clone();
        let set_is_menu_open = set_is_menu_open;
    Callback::new(move |_: leptos::ev::MouseEvent| {
            set_is_menu_open.set(false);
            toast.info("Impostazioni - Funzionalità in sviluppo");
        })
    };

    let handle_profile = {
        let set_is_menu_open = set_is_menu_open;
        let navigate = navigate.clone();
    Callback::new(move |_: leptos::ev::MouseEvent| {
            set_is_menu_open.set(false);
            navigate("/profile", Default::default());
        })
    };

    let handle_notifications = {
        let toast = toast.clone();
        let set_is_menu_open = set_is_menu_open;
    Callback::new(move |_: leptos::ev::MouseEvent| {
            set_is_menu_open.set(false);
            toast.info("Notifiche - Funzionalità in sviluppo");
        })
    };

    let handle_help = {
        let toast = toast.clone();
        let set_is_menu_open = set_is_menu_open;
    Callback::new(move |_: leptos::ev::MouseEvent| {
            set_is_menu_open.set(false);
            toast.info("Aiuto & Supporto - Funzionalità in sviluppo");
        })
    };

    let handle_stats = {
        let toast = toast.clone();
        let set_is_menu_open = set_is_menu_open;
        Callback::new(move |_: leptos::ev::MouseEvent| {
            set_is_menu_open.set(false);
            toast.info("Statistiche - Funzionalità in sviluppo");
        })
    };

    view! {
    <header class="h-12 bg-brand-primary dark:bg-brand-primary-dark text-white flex items-center justify-between pl-2 pr-6 flex-shrink-0 relative">
            <div class="flex items-center gap-4">
                <img 
                    src="/public/logos/logo-full-white.png" 
                    alt="Ruggine" 
                    class="h-8 w-auto"
                />
            </div>

            <div class="flex items-center gap-3">
                // Hamburger menu button
                <div class="relative">
                    <button 
                        on:click=toggle_menu
                        class="p-1 rounded-lg hover:bg-white/10 transition-colors duration-200 focus:outline-none focus:ring-1 focus:ring-white/20"
                        aria-label="Open menu"
                    >
                        <LucideIcon name="menu" size=IconSize::MEDIUM class="text-white" />
                    </button>
                    
                    // Dropdown menu
                    {
                        let handle_logout = handle_logout.clone();
                        let handle_settings = handle_settings.clone();
                        let handle_profile = handle_profile.clone();
                        let handle_notifications = handle_notifications.clone();
                        let handle_help = handle_help.clone();
                        let handle_stats = handle_stats.clone();
                        
                        move || {
                            if is_menu_open.get() {
                                view! {
                                    <div 
                                        class="absolute top-full mt-2 w-64 sm:w-72 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-2 z-50 right-0"
                                        on:click=prevent_close
                                    >
                                        // Theme toggle section
                                        <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
                                            <ThemeSlider />
                                        </div>
                                        
                                        // Menu items section
                                        <div class="py-1">
                                            <button 
                                                class="w-full flex items-center gap-3 px-4 py-2 text-sm text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150"
                                                on:click={let handler = handle_settings.clone(); move |e| handler.call(e)}
                                            >
                                                <LucideIcon name="settings" size=IconSize::MEDIUM />
                                                <span>Impostazioni</span>
                                            </button>
                                            
                                            <button 
                                                class="w-full flex items-center gap-3 px-4 py-2 text-sm text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150"
                                                on:click={let handler = handle_profile.clone(); move |e| handler.call(e)}
                                            >
                                                <LucideIcon name="user" size=IconSize::MEDIUM />
                                                <span>Profilo utente</span>
                                            </button>
                                            
                                            <button 
                                                class="w-full flex items-center gap-3 px-4 py-2 text-sm text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150"
                                                on:click={let handler = handle_notifications.clone(); move |e| handler.call(e)}
                                            >
                                                <LucideIcon name="bell" size=IconSize::MEDIUM />
                                                <div class="flex flex-col items-start">
                                                    <span>Notifiche</span>
                                                    <span class="text-xs text-gray-600 dark:text-gray-300">3 non lette</span>
                                                </div>
                                            </button>
                                            
                                            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
                                            
                                            <button 
                                                class="w-full flex items-center gap-3 px-4 py-2 text-sm text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150"
                                                on:click={let handler = handle_help.clone(); move |e| handler.call(e)}
                                            >
                                                <LucideIcon name="help-circle" size=IconSize::MEDIUM />
                                                <span>Aiuto & Supporto</span>
                                            </button>
                                            
                                            {if is_admin {
                                                view! {
                                                    <button 
                                                        class="w-full flex items-center gap-3 px-4 py-2 text-sm text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150"
                                                        on:click={let navigate = navigate.clone(); let set_is_menu_open = set_is_menu_open; move |_| { set_is_menu_open.set(false); navigate("/admin/cpu-logs", Default::default()); }}
                                                    >
                                                        <LucideIcon name="server" size=IconSize::MEDIUM />
                                                        <span>"CPU Logs"</span>
                                                    </button>
                                                }.into_view()
                                            } else {
                                                view! { <div></div> }.into_view()
                                            }}
                                            
                                            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
                                            
                                            <button 
                                                class="w-full flex items-center gap-3 px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors duration-150"
                                                on:click={let handler = handle_logout.clone(); move |e| handler.call(e)}
                                            >
                                                <LucideIcon name="log-out" size=IconSize::MEDIUM class="text-red-600 dark:text-red-400" />
                                                <span>Logout</span>
                                            </button>
                                        </div>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }
                        }
                    }
                    
                    // Backdrop overlay (for all devices - transparent)
                    {move || {
                        if is_menu_open.get() {
                            view! {
                                <div 
                                    class="fixed inset-0 z-40"
                                    on:click=close_menu
                                ></div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }
                    }}
                </div>
                
                {match user_profile {
                    Some(user) => {
                        // Dividi il full_name in first_name e last_name
                        let full_name = user.full_name();
                        let name_parts: Vec<&str> = full_name.split_whitespace().collect();
                        let first_name = name_parts.first().unwrap_or(&"User").to_string();
                        let last_name = name_parts.get(1).unwrap_or(&"Default").to_string();
                        // Usa l'email come username temporaneo (manca username nel DTO)
                        let username = user.email.split('@').next().unwrap_or("user").to_string();
                        
                        view! {
                            <UserAvatar 
                                name=first_name 
                                surname=last_name 
                                username=username 
                                size="md" 
                            />
                        }.into_view()
                    },
                    None => view! {
                        <UserAvatar 
                            name="User".to_string() 
                            surname="Default".to_string() 
                            username="user".to_string() 
                            size="md" 
                        />
                    }.into_view(),
                }}
            </div>
        </header>
    }
}
