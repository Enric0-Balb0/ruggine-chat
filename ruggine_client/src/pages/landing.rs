use leptos::*;
use leptos_router::*;
use crate::api::services::AuthService;
use crate::utils::{StorageService, auth_error_to_login_message};
use crate::api::client::ApiClient;
use crate::context::auth_context::use_auth_context;
use crate::config::constants::AppConstants;
use crate::components::{ThemeToggle, use_toast};

#[component]
pub fn LandingPage() -> impl IntoView {
    let navigate = use_navigate();
    let toast = use_toast();
    let auth_ctx = use_auth_context();
    
    // Form signals
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (remember_me, set_remember_me) = create_signal(false);
    let (loading, set_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal(Option::<String>::None);

    let storage_service = StorageService::new();
    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        storage_service.clone(),
    );

    // Carica credenziali salvate se Remember Me è attivo
    create_effect({
        let storage_service = storage_service.clone();
        move |_| {
            if let Some((saved_email, saved_password)) = storage_service.get_remember_me_credentials() {
                set_email.set(saved_email);
                set_password.set(saved_password);
                set_remember_me.set(true);
            }
        }
    });

    let handle_login = create_action({
        let navigate = navigate.clone();
        let toast = toast.clone();
        let storage_service = storage_service.clone();
        move |_: &()| {
            let email_val = email.get_untracked();
            let password_val = password.get_untracked();
            let remember_me_val = remember_me.get_untracked();
            
            let auth_service = auth_service.clone();
            let navigate = navigate.clone();
            let toast = toast.clone();
            let storage_service = storage_service.clone();
            
            async move {
                log::info!("Landing::handle_login: INIZIO - email={}, remember_me={}", email_val, remember_me_val);
                // Validation
                if email_val.is_empty() || password_val.is_empty() {
                    set_error_message.set(Some("Email e password sono obbligatori".to_string()));
                    return;
                }
                
                set_loading.set(true);
                set_error_message.set(None);
                
                match auth_service.login(email_val.clone(), password_val.clone()).await {
                    Ok(user_profile) => {
                        log::info!("Landing::handle_login: Login riuscito, remember_me={}", remember_me_val);
                        
                        // Prima gestisci Remember Me credentials
                        if remember_me_val {
                            if let Err(e) = storage_service.set_remember_me(&email_val, &password_val, true) {
                                log::warn!("Errore nel salvare Remember Me: {:?}", e);
                            } else {
                                log::info!("Landing::handle_login: Remember Me credenziali salvate");
                            }
                        } else {
                            let _ = storage_service.clear_remember_me();
                            log::info!("Landing::handle_login: Remember Me disabilitato, credenziali pulite");
                        }
                        
                        // Poi gestisci il token con la modalità corretta
                        if let Some(token) = auth_service.get_storage_service().get_token() {
                            log::info!("Landing::handle_login: Token trovato, salvandolo con remember_me={}", remember_me_val);
                            
                            // Cancella il token temporaneo salvato da AuthService
                            let _ = auth_service.get_storage_service().clear_session();
                            
                            // Salva il token nella modalità corretta (localStorage o sessionStorage)
                            if let Err(e) = storage_service.store_token_with_remember_me(&token, remember_me_val) {
                                log::error!("Landing::handle_login: Errore nel salvare il token: {:?}", e);
                            } else {
                                log::info!("Landing::handle_login: Token salvato correttamente");
                            }
                            
                            // Aggiorna il context con il token
                            auth_ctx.token.set(Some(token.token.clone()));
                            log::info!("Landing::handle_login: Context aggiornato con token");
                        } else {
                            log::error!("Landing::handle_login: ERRORE - Nessun token trovato dopo login riuscito!");
                        }
                        
                        // Salva nuovamente il profilo con la modalità corretta (localStorage o sessionStorage)
                        if let Err(e) = storage_service.store_user_profile_with_remember_me(&user_profile, remember_me_val) {
                            log::error!("Landing::handle_login: Errore nel salvare il profilo con remember_me: {:?}", e);
                        } else {
                            log::info!("Landing::handle_login: Profilo salvato correttamente con remember_me={}", remember_me_val);
                        }
                        
                        let success_message = format!("Login effettuato con successo! {}", user_profile.welcome_message_success());
                        toast.success(&success_message);
                        set_loading.set(false);
                        navigate("/", Default::default());
                    }
                    Err(error) => {
                        set_loading.set(false);
                        let error_message = auth_error_to_login_message(&error);
                        set_error_message.set(Some(error_message.clone()));
                        
                        // Also show in toast for better visibility
                        toast.error(&error_message);
                    }
                }
            }
        }
    });

    view! {
        <div class="h-screen w-screen overflow-hidden bg-cover bg-center bg-no-repeat transition-colors" 
             style="background-image: url('/public/images/bg-landing-full.png');">
            // Overlay per migliorare la leggibilità del testo
            <div class="absolute inset-0 bg-black bg-opacity-50 dark:bg-opacity-70"></div>

            <div class="absolute left-8 top-8 z-20 flex items-center gap-4">
                <img 
                    src="/public/logos/logo-full-white.png" 
                    alt="Ruggine" 
                    class="h-24 w-auto drop-shadow-lg"
                />
                <div class="ml-auto">
                    <ThemeToggle />
                </div>
            </div>
            
            // Contenuto principale centrato  
            <main class="relative z-10 h-full flex items-center justify-center px-6">
                <div class="max-w-md w-full">
                    <div class="text-center mb-8">
                        <h1 class="text-4xl font-bold text-white mb-4 drop-shadow-lg">
                            "Bentornato in Ruggine"
                        </h1>
                        <p class="text-white text-lg drop-shadow-md opacity-90">
                            "Accedi per continuare a collaborare con il tuo team"
                        </p>
                    </div>

                    // Login Form
                    <div class="bg-white/95 dark:bg-surface-dark/95 backdrop-blur-sm rounded-lg shadow-xl p-8 border border-border dark:border-border-dark transition-colors">
                        <h2 class="text-2xl font-semibold text-text-primary dark:text-text-primary-dark mb-6 text-center">
                            "Accedi Ora"
                        </h2>
                        
                        <form on:submit=move |ev| {
                            ev.prevent_default();
                            handle_login.dispatch(());
                        } class="space-y-4">
                            <div>
                                <label for="email" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                    "Email"
                                </label>
                                <input
                                    type="email"
                                    id="email"
                                    required
                                    class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark focus:border-transparent transition-colors"
                                    placeholder="Email"
                                    prop:value=email
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                />
                            </div>

                            <div>
                                <label for="password" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                    "Password"
                                </label>
                                <input
                                    type="password"
                                    id="password"
                                    required
                                    class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark focus:border-transparent transition-colors"
                                    placeholder="Password"
                                    prop:value=password
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                />
                            </div>

                            // Remember Me checkbox
                            <div class="flex items-center justify-between">
                                <div class="flex items-center">
                                    <input
                                        type="checkbox"
                                        id="remember_me"
                                        class="h-4 w-4 text-brand-primary focus:ring-accent dark:focus:ring-accent-dark border-gray-300 dark:border-gray-600 rounded transition-colors"
                                        prop:checked=remember_me
                                        on:change=move |ev| {
                                            let checked = event_target_checked(&ev);
                                            set_remember_me.set(checked);
                                        }
                                    />
                                    <label for="remember_me" class="ml-2 block text-sm text-text-secondary dark:text-text-secondary-dark">
                                        "Ricordami"
                                    </label>
                                </div>
                            </div>

                            {move || error_message.get().map(|msg| view! {
                                <div class="bg-error-light dark:bg-error-dark border border-error dark:border-error text-error-dark dark:text-error-light px-3 py-2 mb-3 rounded-md text-sm text-center">
                                    {msg}
                                </div>
                            })}

                            <button
                                type="submit"
                                disabled=move || loading.get() || handle_login.pending().get()
                                class="w-full bg-brand-primary dark:bg-brand-primary-dark hover:bg-brand-primary-light dark:hover:bg-brand-primary disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2.5 px-4 rounded-md transition-colors text-sm"
                            >
                                {move || if loading.get() || handle_login.pending().get() { "Accesso in corso..." } else { "Accedi" }}
                            </button>
                        </form>

                        <div class="mt-4 text-center text-sm">
                            <p class="text-sm text-text-secondary dark:text-text-secondary-dark">
                                "Non hai un account?"
                            </p>
                            <button 
                                type="button"
                                class="text-brand-primary dark:text-accent hover:text-brand-primary-light dark:hover:text-accent-dark font-medium"
                                on:click=move |_| {
                                    navigate("/register", Default::default());
                                }
                            >
                                "Registrati"
                            </button>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    }
}
