use leptos::*;
use leptos_router::*;
use crate::api::services::AuthService;
use crate::utils::{StorageService, auth_error_to_register_message};
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;
use crate::components::{ThemeToggle, use_toast};

#[component]
pub fn RegisterPage() -> impl IntoView {
    let navigate = use_navigate();
    let toast = use_toast();
    
    // Form signals
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (confirm_password, set_confirm_password) = create_signal(String::new());
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (username, set_username) = create_signal(String::new());
    let (birthday, set_birthday) = create_signal(String::new());
    let (address, set_address) = create_signal(String::new());
    let (gender, set_gender) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal(Option::<String>::None);

    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    );

    let handle_register = {
        let navigate = navigate.clone();
        let toast = toast.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            
            let email_val = email.get();
            let password_val = password.get();
            let confirm_password_val = confirm_password.get();
            let first_name_val = first_name.get();
            let last_name_val = last_name.get();
            let username_val = username.get();
            let birthday_val = birthday.get();
            let address_val = address.get();
            let gender_val = gender.get();
            let auth_service = auth_service.clone();
            let navigate = navigate.clone();
            let toast = toast.clone();
            
            // Validazione campi obbligatori
            if first_name_val.trim().is_empty() || last_name_val.trim().is_empty() || 
               username_val.trim().is_empty() || birthday_val.trim().is_empty() || 
               address_val.trim().is_empty() || gender_val.trim().is_empty() {
                set_error_message.set(Some("Tutti i campi sono obbligatori".to_string()));
                return;
            }
            
            // Validazione password
            if password_val != confirm_password_val {
                set_error_message.set(Some("Le password non corrispondono".to_string()));
                return;
            }
            
            if password_val.len() < 6 {
                set_error_message.set(Some("La password deve essere di almeno 6 caratteri".to_string()));
                return;
            }
            
            spawn_local(async move {
                set_loading.set(true);
                set_error_message.set(None);
                
                match auth_service.register(
                    email_val, 
                    password_val, 
                    first_name_val,
                    last_name_val,
                    username_val,
                    birthday_val,
                    address_val,
                    gender_val
                ).await {
                    Ok(user_profile) => {
                        let success_message = format!("Registrazione completata con successo! {}", user_profile.welcome_message_success());
                        toast.success(&success_message);
                        set_loading.set(false);
                        navigate("/", Default::default());
                    }
                    Err(error) => {
                        set_loading.set(false);
                        let error_message = auth_error_to_register_message(&error);
                        set_error_message.set(Some(error_message.clone()));
                        
                        // Also show in toast for better visibility
                        toast.error(&error_message);
                    }
                }
            });
        }
    };

    view! {
        <div class="min-h-screen w-screen overflow-auto bg-cover bg-center bg-no-repeat transition-colors" style="background-image: url('public/images/bg-landing-full.png');">
            // Overlay per migliorare la leggibilità del testo
            <div class="absolute inset-0 bg-black bg-opacity-50 dark:bg-opacity-70"></div>
            
            <div class="absolute top-4 left-8 right-8 z-20 flex items-center justify-between">
                <img 
                    src="public/logos/logo-full-white.png" 
                    alt="Ruggine" 
                    class="h-24 w-auto drop-shadow-lg"
                />
                <ThemeToggle />
            </div>
            
            // Contenuto principale centrato  
            <main class="relative z-10 min-h-screen flex items-center justify-center px-6 py-8">
                <div class="max-w-lg w-full">
                    // Hero Section
                    <div class="text-center mb-6">
                        <h1 class="text-3xl font-bold text-white mb-3 drop-shadow-lg">
                            "Unisciti a Ruggine"
                        </h1>
                        <p class="text-white text-base drop-shadow-md opacity-90">
                            "Crea il tuo account e inizia a collaborare"
                        </p>
                    </div>

                    // Register Form
                    <div class="bg-white/95 dark:bg-surface-dark/95 backdrop-blur-sm rounded-lg shadow-xl border border-border dark:border-border-dark transition-colors">
                        
                        <div class="max-h-96 overflow-y-auto form-container-scroll px-6 pb-6">
                            <form id="register-form" on:submit=handle_register class="space-y-3">
                                <div class="grid grid-cols-2 mt-6 gap-3">
                                    <div>
                                        <label for="first_name" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                            "Nome"
                                        </label>
                                        <input
                                            type="text"
                                            id="first_name"
                                            required
                                            class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                            placeholder="Nome"
                                            prop:value=first_name
                                            on:input=move |ev| set_first_name.set(event_target_value(&ev))
                                        />
                                    </div>

                                    <div>
                                        <label for="last_name" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                            "Cognome"
                                        </label>
                                        <input
                                            type="text"
                                            id="last_name"
                                            required
                                            class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                            placeholder="Cognome"
                                            prop:value=last_name
                                            on:input=move |ev| set_last_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>

                                <div>
                                    <label for="username" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                        "Username"
                                    </label>
                                    <input
                                        type="text"
                                        id="username"
                                        required
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        placeholder="Username"
                                        prop:value=username
                                        on:input=move |ev| set_username.set(event_target_value(&ev))
                                    />
                                </div>

                                <div>
                                    <label for="email" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                        "Email"
                                    </label>
                                    <input
                                        type="email"
                                        id="email"
                                        required
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        placeholder="Email"
                                        prop:value=email
                                        on:input=move |ev| set_email.set(event_target_value(&ev))
                                    />
                                </div>

                                <div class="grid grid-cols-2 gap-3">
                                    <div>
                                        <label for="birthday" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                            "Data di nascita"
                                        </label>
                                        <input
                                            type="date"
                                            id="birthday"
                                            required
                                            class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                            style="color: inherit;"
                                            prop:value=birthday
                                            on:input=move |ev| set_birthday.set(event_target_value(&ev))
                                        />
                                    </div>

                                    <div>
                                        <label for="gender" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                            "Genere"
                                        </label>
                                        <select
                                            id="gender"
                                            required
                                            class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                            style="color: inherit;"
                                            prop:value=gender
                                            on:change=move |ev| set_gender.set(event_target_value(&ev))
                                        >
                                            <option value="">"Seleziona..."</option>
                                            <option value="male">"Maschio"</option>
                                            <option value="female">"Femmina"</option>
                                            <option value="other">"Altro"</option>
                                        </select>
                                    </div>
                                </div>

                                <div>
                                    <label for="address" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                        "Indirizzo"
                                    </label>
                                    <input
                                        type="text"
                                        id="address"
                                        required
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        placeholder="Indirizzo"
                                        prop:value=address
                                        on:input=move |ev| set_address.set(event_target_value(&ev))
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
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        placeholder="Password"
                                        prop:value=password
                                        on:input=move |ev| set_password.set(event_target_value(&ev))
                                    />
                                </div>

                                <div>
                                    <label for="confirm_password" class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">
                                        "Conferma password"
                                    </label>
                                    <input
                                        type="password"
                                        id="confirm_password"
                                        required
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        placeholder="Conferma password"
                                        prop:value=confirm_password
                                        on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                    />
                                </div>

                                {move || error_message.get().map(|msg| view! {
                                    <div class="bg-error-light dark:bg-error-dark border border-error dark:border-error text-error-dark dark:text-error-light px-3 py-2 mb-2 rounded-md text-sm text-center">
                                        {msg}
                                    </div>
                                })}
                            </form>
                        </div>
                        
                        <div class="px-6 pb-6">
                            <button
                                form="register-form"
                                type="submit"
                                disabled=loading
                                class="w-full bg-brand-primary dark:bg-brand-primary-dark hover:bg-brand-primary-light dark:hover:bg-brand-primary disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2.5 px-4 rounded-md transition-colors text-sm"
                            >
                                {move || if loading.get() { "Registrazione in corso..." } else { "Registrati" }}
                            </button>
                            
                            <div class="mt-4 text-center text-sm">
                                <span class="text-text-secondary dark:text-text-secondary-dark">"Hai già un account? "</span>
                                <button 
                                    type="button"
                                    class="text-brand-primary dark:text-accent hover:text-brand-primary-light dark:hover:text-accent-dark font-medium"
                                    on:click=move |_| {
                                        navigate("/login", Default::default());
                                    }
                                >
                                    "Accedi"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </main>
        </div>
    }
}
