use leptos::*;
use leptos_router::*;
use std::rc::Rc;
use crate::components::{UserAvatar, ThemeSlider, LucideIcon, use_toast};
use crate::components::ui::icons::icon_size;
use crate::api::services::AuthService;
use crate::utils::StorageService;
use crate::hooks::use_group_message_ws::UseGroupMessageWs;
use crate::types::WebSocketMessage;
use crate::types::{ClientAction, GroupAction};
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;
use crate::utils::error_recovery::NetworkOperation;

/// Main app navbar for authenticated users (based on UI mock)
#[component]
pub fn AppNavbar() -> impl IntoView {
    // Toast for user feedback
    let toast = use_toast();
    let navigate = use_navigate();
    let ws_ctx_signal = use_context::<ReadSignal<Option<UseGroupMessageWs>>>()
        .expect("WebSocket context should be provided");

    // Usa il context di autenticazione per ottenere il profilo utente reattivo
    let auth_ctx = crate::context::auth_context::use_auth_context();
    
    // Debug logging per verificare il profilo utente e lo stato del localStorage
    let storage_debug = StorageService::new();
    let has_token = storage_debug.get_token().is_some();
    let has_profile_in_storage = storage_debug.get_user_profile().is_some();
    
    log::info!("AppNavbar: Token presente = {}, Profilo in localStorage = {}", has_token, has_profile_in_storage);
    
    // Se abbiamo token ma non profilo nel context, proviamo a ricaricarlo
    if has_token && !has_profile_in_storage {
        log::warn!("AppNavbar: Token presente ma profilo mancante nel localStorage - possibile problema di sincronizzazione");
    }
    
    // Crea un memo per il profilo utente che reagisce ai cambiamenti
    let user_profile = create_memo(move |_| {
        let profile = auth_ctx.user_profile.get();
        
        if let Some(ref profile) = profile {
            log::info!("AppNavbar: Profilo utente caricato dal context - nome: '{}', cognome: '{}', email: '{}'", 
                profile.first_name, profile.last_name, profile.email);
        } else {
            log::warn!("AppNavbar: Nessun profilo utente trovato nel context");
            log::info!("AppNavbar: Utente deve fare login manuale");
        }
        
        profile
    });

    // Effetto per tentare di ricaricare il profilo se abbiamo token ma non profilo
    create_effect(move |_| {
        let token = auth_ctx.token.get();
        let profile = auth_ctx.user_profile.get();
        
        // Se abbiamo token ma non profilo, proviamo a ricaricarlo dal server
        if token.is_some() && profile.is_none() {
            log::info!("AppNavbar: Token presente ma profilo mancante, tentativo di ricaricarlo dal server");
            
            spawn_local(async move {
                use crate::api::services::UserService;
                use crate::api::client::ApiClient;
                use crate::config::constants::AppConstants;
                use crate::utils::StorageService;
                
                let storage = StorageService::new();
                let client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
                let user_service = UserService::new(client, storage.clone());
                
                match user_service.get_current_profile().await {
                    Ok(profile) => {
                        log::info!("AppNavbar: Profilo ricaricato con successo dal server: {} {}", 
                            profile.first_name, profile.last_name);
                        
                        // Salva il profilo nel localStorage e aggiorna il context
                        if let Err(e) = storage.store_user_profile(&profile) {
                            log::error!("AppNavbar: Errore nel salvare il profilo ricaricato: {:?}", e);
                        } else {
                            auth_ctx.user_profile.set(Some(profile));
                        }
                    }
                    Err(e) => {
                        log::error!("AppNavbar: Errore nel ricaricare il profilo dal server: {:?}", e);
                    }
                }
            });
        }
    });
    
    let is_admin = move || user_profile.get().as_ref().map(|u| u.is_admin()).unwrap_or(false);
    
    // State for hamburger menu dropdown
    let (is_menu_open, set_is_menu_open) = create_signal(false);

    // State for avatar hover menu (shows username/email)
    let (is_avatar_hover_open, set_is_avatar_hover_open) = create_signal(false);
    
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
    let toast = toast.clone();
    let navigate = navigate.clone();
    let set_is_menu_open = set_is_menu_open;
    let ws_ctx_signal = ws_ctx_signal.clone();
    let auth_ctx = auth_ctx.clone();
        
    Callback::new(move |_: leptos::ev::MouseEvent| {
            set_is_menu_open.set(false);
            let toast = toast.clone();
            let navigate = navigate.clone();
            let ws_to_use = ws_ctx_signal.get();
            let auth_ctx = auth_ctx.clone();

            spawn_local(async move {
                // Send leave message first if WebSocket is available
                if let Some(ref ws) = ws_to_use {
                    ws.send_message.set(Some(WebSocketMessage::Request {
                        request_id: uuid::Uuid::new_v4().to_string(),
                        action: ClientAction::Groups(GroupAction::Leave {}),
                    }));
                }

                // Crea un nuovo AuthService per il logout
                let auth_service = AuthService::new(
                    ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
                    StorageService::new(),
                );

                match auth_service.logout().with_auto_retry("logout").await {
                    Some(_) => {
                        // Pulisci anche il context
                        auth_ctx.token.set(None);
                        auth_ctx.user_profile.set(None);
                        
                        toast.success("Logout effettuato con successo!");
                        navigate("/login", Default::default());
                    },
                    None => {
                        leptos::logging::error!("Logout failed after retries");
                        toast.error("Errore durante il logout. L'app verrà comunque disconnessa.");
                        // Force navigation even on failure since we want to log out anyway
                        
                        // Pulisci il context anche in caso di errore
                        auth_ctx.token.set(None);
                        auth_ctx.user_profile.set(None);
                        
                        navigate("/login", Default::default());
                    }
                }

                // Disconnect WebSocket only once, at the end
                if let Some(ref ws) = ws_to_use {
                    // Try to disconnect, but catch any panics if signal is disposed
                    let _ = std::panic::catch_unwind(|| {
                        ws.disconnect.set(true);
                    });
                }
            });
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

    view! {
    <header class="h-12 bg-brand-primary dark:bg-brand-primary-dark text-white flex items-center justify-between pl-2 pr-6 flex-shrink-0 relative">
            <div class="flex items-center gap-4">
                <img 
                    src="/public/logos/logo-full-white.png" 
                    alt="Ruggine" 
                    class="h-8 w-auto cursor-pointer"
                    on:click={let navigate = navigate.clone(); move |_| { navigate("/", Default::default()); }}
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
                        <LucideIcon name="menu" size=icon_size::MEDIUM class="text-white" />
                    </button>
                    
                    // Dropdown menu
                    {
                        let handle_logout = handle_logout.clone();
                        let handle_profile = handle_profile.clone();
                        let navigate_rc = Rc::new(navigate.clone());
                        
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
                                                on:click={let handler = handle_profile.clone(); move |e| handler.call(e)}
                                            >
                                                <LucideIcon name="user" size=icon_size::MEDIUM />
                                                <span>Profilo utente</span>
                                            </button>
                                            
                                            {
                                                let navigate_for_admin = navigate_rc.clone();
                                                move || if is_admin() {
                                                    view! {
                                                        <button 
                                                            class="w-full flex items-center gap-3 px-4 py-2 text-sm text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150"
                                                            on:click={let navigate = (*navigate_for_admin).clone(); let set_is_menu_open = set_is_menu_open; move |_| { set_is_menu_open.set(false); navigate("/admin/cpu-logs", Default::default()); }}
                                                        >
                                                            <LucideIcon name="server" size=icon_size::MEDIUM />
                                                            <span>"CPU Logs"</span>
                                                        </button>
                                                    }.into_view()
                                                } else {
                                                    view! { <div></div> }.into_view()
                                                }
                                            }
                                            
                                            <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
                                            
                                            <button 
                                                class="w-full flex items-center gap-3 px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors duration-150"
                                                on:click={let handler = handle_logout.clone(); move |e| handler.call(e)}
                                            >
                                                <LucideIcon name="log-out" size=icon_size::MEDIUM class="text-red-600 dark:text-red-400" />
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
                
                {move || match user_profile.get() {
                    Some(user) => {
                        // Use a hoverable group wrapper so a small menu with username/email
                        // appears when hovering the avatar. Capture clones for closure use.
                        let first_name = user.first_name.clone();
                        let last_name = user.last_name.clone();
                        let username = if !user.username.is_empty() { user.username.clone() } else { user.email.split('@').next().unwrap_or("user").to_string() };
                        let email = user.email.clone();

                        view! {
                            <div 
                                class="relative"
                                on:mouseenter=move |_| set_is_avatar_hover_open.set(true)
                                on:mouseleave=move |_| set_is_avatar_hover_open.set(false)
                            >
                                <UserAvatar 
                                    name=first_name.clone()
                                    surname=last_name.clone()
                                    username=username.clone()
                                    size="md"
                                />

                                <div class=move || {
                                    let base = "absolute right-0 mt-2 w-56 bg-white dark:bg-gray-800 rounded shadow-lg border border-gray-200 dark:border-gray-700 py-2 px-3 z-50 transition-opacity duration-150";
                                    if is_avatar_hover_open.get() {
                                        format!("{} opacity-100 pointer-events-auto", base)
                                    } else {
                                        format!("{} opacity-0 pointer-events-none", base)
                                    }
                                }>
                                    <div class="text-sm font-semibold text-gray-900 dark:text-gray-100">{move || format!("{} {}", first_name.clone(), last_name.clone())}</div>
                                    <div class="text-xs text-gray-600 dark:text-gray-400">{move || format!("@{}", username.clone())}</div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400 truncate">{move || email.clone()}</div>
                                </div>
                            </div>
                        }.into_view()
                    },
                    None => {
                        // Default avatar without hover menu
                        view! {
                            <UserAvatar 
                                name="User".to_string() 
                                surname="Default".to_string() 
                                username="user".to_string() 
                                size="md" 
                            />
                        }.into_view()
                    },
                }}
            </div>
        </header>
    }
}
