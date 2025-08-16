use leptos::*;
use leptos_router::*;
use crate::api::services::AuthService;
use crate::utils::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;
use crate::error::AuthError;
use crate::components::{ThemeToggle, use_toast};

#[component]
pub fn LandingPage() -> impl IntoView {
    let navigate = use_navigate();
    let toast = use_toast();
    
    // Form signals
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal(Option::<String>::None);

    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    );

    let handle_login = {
        let navigate = navigate.clone();
        let toast = toast.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            
            let email_val = email.get();
            let password_val = password.get();
            let auth_service = auth_service.clone();
            let navigate = navigate.clone();
            let toast = toast.clone();
            
            spawn_local(async move {
                set_loading.set(true);
                set_error_message.set(None);
                
                match auth_service.login(email_val, password_val).await {
                    Ok(_user_profile) => {
                        toast.success("Login effettuato con successo! Benvenuto/a!");
                        navigate("/", Default::default());
                    }
                    Err(AuthError::InvalidCredentials) => {
                        set_error_message.set(Some("Email o password non validi".to_string()));
                    }
                    Err(AuthError::NetworkError(msg)) => {
                        set_error_message.set(Some(format!("Errore di connessione: {}", msg)));
                    }
                    Err(_e) => {
                        set_error_message.set(Some("Errore durante il login".to_string()));
                    }
                }
                
                set_loading.set(false);
            });
        }
    };

    view! {
        <div class="h-screen w-screen overflow-hidden bg-cover bg-center bg-no-repeat transition-colors" 
             style="background-image: url('public/images/bg-landing-full.png');">
            // Overlay per migliorare la leggibilità del testo
            <div class="absolute inset-0 bg-black bg-opacity-50 dark:bg-opacity-70"></div>

            <div class="absolute left-8 top-8 z-20 flex items-center gap-4">
                <img 
                    src="public/logos/logo-full-white.png" 
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
                        
                        <form on:submit=handle_login class="space-y-4">
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

                            {move || error_message.get().map(|msg| view! {
                                <div class="bg-error-light dark:bg-error-dark border border-error dark:border-error text-error-dark dark:text-error-light px-3 py-2 mb-3 rounded-md text-sm text-center">
                                    {msg}
                                </div>
                            })}

                            <button
                                type="submit"
                                disabled=loading
                                class="w-full bg-brand-primary dark:bg-brand-primary-dark hover:bg-brand-primary-light dark:hover:bg-brand-primary disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2.5 px-4 rounded-md transition-colors text-sm"
                            >
                                {move || if loading.get() { "Accesso in corso..." } else { "Accedi" }}
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
