use leptos::*;
use leptos_router::*;
use crate::api::services::AuthService;
use crate::utils::{StorageService, auth_error_to_register_message};
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;
use crate::components::{ThemeToggle, use_toast};
use chrono;

#[derive(Clone, Debug, PartialEq)]
pub enum RegistrationStep {
    PersonalInfo,
    AccountInfo,
    Security,
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let navigate = use_navigate();
    let navigate_clone = navigate.clone();
    let toast = use_toast();
    
    // Step management
    let (current_step, set_current_step) = create_signal(RegistrationStep::PersonalInfo);
    
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
    
    // Validation error signals for visual feedback
    let (first_name_error, set_first_name_error) = create_signal(false);
    let (last_name_error, set_last_name_error) = create_signal(false);
    let (birthday_error, set_birthday_error) = create_signal(false);
    let (gender_error, set_gender_error) = create_signal(false);
    let (address_error, set_address_error) = create_signal(false);
    let (username_error, set_username_error) = create_signal(false);
    let (email_error, set_email_error) = create_signal(false);
    let (password_error, set_password_error) = create_signal(false);
    let (confirm_password_error, set_confirm_password_error) = create_signal(false);

    let storage_service = StorageService::new();
    let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
    
    // Configure auth token if available
    if let Some(token_response) = storage_service.get_token() {
        http_client.set_auth_token(Some(token_response.token));
    }

    let auth_service = AuthService::new(
        http_client.clone(),
        storage_service.clone(),
    );

    let validate_current_step = move || -> Vec<String> {
        let mut validation_errors = Vec::new();
        
        // Reset all error states first
        set_first_name_error.set(false);
        set_last_name_error.set(false);
        set_birthday_error.set(false);
        set_gender_error.set(false);
        set_address_error.set(false);
        set_username_error.set(false);
        set_email_error.set(false);
        set_password_error.set(false);
        set_confirm_password_error.set(false);

    match current_step.get_untracked() {
            RegistrationStep::PersonalInfo => {
                // Check fields in order and stop at first empty one
                if first_name.get_untracked().trim().is_empty() {
                    validation_errors.push("Il nome è obbligatorio".to_string());
                    set_first_name_error.set(true);
                    return validation_errors; // Stop here - only highlight first empty field
                }
                if last_name.get_untracked().trim().is_empty() {
                    validation_errors.push("Il cognome è obbligatorio".to_string());
                    set_last_name_error.set(true);
                    return validation_errors;
                }
                if birthday.get_untracked().trim().is_empty() {
                    validation_errors.push("La data di nascita è obbligatoria".to_string());
                    set_birthday_error.set(true);
                    return validation_errors;
                }
                
                // Validate birthday is not in the future
                if !birthday.get_untracked().trim().is_empty() {
                    if let Ok(birth_date) = chrono::NaiveDate::parse_from_str(&birthday.get_untracked(), "%Y-%m-%d") {
                        let today = chrono::Local::now().date_naive();
                        if birth_date > today {
                            validation_errors.push("La data di nascita non può essere nel futuro".to_string());
                            set_birthday_error.set(true);
                            return validation_errors;
                        }
                    } else {
                        validation_errors.push("Formato data non valido".to_string());
                        set_birthday_error.set(true);
                        return validation_errors;
                    }
                }
                if gender.get_untracked().trim().is_empty() {
                    validation_errors.push("Il genere è obbligatorio".to_string());
                    set_gender_error.set(true);
                    return validation_errors;
                }
                if address.get_untracked().trim().is_empty() {
                    validation_errors.push("L'indirizzo è obbligatorio".to_string());
                    set_address_error.set(true);
                    return validation_errors;
                }
            }
            RegistrationStep::AccountInfo => {
                // Check username first
                if username.get_untracked().trim().is_empty() {
                    validation_errors.push("Lo username è obbligatorio".to_string());
                    set_username_error.set(true);
                    return validation_errors;
                } else if username.get_untracked().len() < 3 {
                    validation_errors.push("Lo username deve essere almeno 3 caratteri".to_string());
                    set_username_error.set(true);
                    return validation_errors;
                }
                // Then check email
                if email.get_untracked().trim().is_empty() {
                    validation_errors.push("L'email è obbligatoria".to_string());
                    set_email_error.set(true);
                    return validation_errors;
                } else if !email.get_untracked().contains('@') {
                    validation_errors.push("L'email non è valida".to_string());
                    set_email_error.set(true);
                    return validation_errors;
                }
            }
            RegistrationStep::Security => {
                // Check password first
                if password.get_untracked().trim().is_empty() {
                    validation_errors.push("La password è obbligatoria".to_string());
                    set_password_error.set(true);
                    return validation_errors;
                } else if password.get_untracked().len() < 6 {
                    validation_errors.push("La password deve essere lunga almeno 6 caratteri".to_string());
                    set_password_error.set(true);
                    return validation_errors;
                }
                // Then check password confirmation
                if confirm_password.get_untracked() != password.get_untracked() {
                    validation_errors.push("Le password non corrispondono".to_string());
                    set_confirm_password_error.set(true);
                    return validation_errors;
                }
            }
        }

        validation_errors
    };
    
    // Helper function to get input CSS classes with error state
    let get_input_class = move |has_error: ReadSignal<bool>| {
        move || {
            if has_error.get() {
                "w-full px-3 py-2 text-sm border-2 border-error rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-error focus:border-error !focus:border-error transition-colors"
            } else {
                "w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark focus:border-transparent transition-colors"
            }
        }
    };

    let handle_next = move |_| {
        let validation_errors = validate_current_step();
        if !validation_errors.is_empty() {
            // Show only the first error message
            set_error_message.set(Some(validation_errors[0].clone()));
            return;
        }

        set_error_message.set(None);
        match current_step.get_untracked() {
            RegistrationStep::PersonalInfo => set_current_step.set(RegistrationStep::AccountInfo),
            RegistrationStep::AccountInfo => set_current_step.set(RegistrationStep::Security),
            RegistrationStep::Security => {
                // Final registration
                let auth_service = auth_service.clone();
                let navigate = navigate.clone();
                let toast = toast.clone();
                
                spawn_local(async move {
                    set_loading.set(true);
                    set_error_message.set(None);

                    match auth_service.register(
                        email.get_untracked(), 
                        password.get_untracked(), 
                        first_name.get_untracked(),
                        last_name.get_untracked(),
                        username.get_untracked(),
                        birthday.get_untracked(),
                        address.get_untracked(),
                        gender.get_untracked()
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
                            toast.error(&error_message);
                            // Don't reset form values on error - keep user's input
                        }
                    }
                });
            }
        }
    };

    let handle_back = move |_| {
        set_error_message.set(None);
        match current_step.get() {
            RegistrationStep::AccountInfo => set_current_step.set(RegistrationStep::PersonalInfo),
            RegistrationStep::Security => set_current_step.set(RegistrationStep::AccountInfo),
            _ => {}
        }
    };

    let get_step_title = move || {
        match current_step.get() {
            RegistrationStep::PersonalInfo => "Informazioni Personali",
            RegistrationStep::AccountInfo => "Informazioni Account",
            RegistrationStep::Security => "Sicurezza",
        }
    };

    let get_step_subtitle = move || {
        match current_step.get() {
            RegistrationStep::PersonalInfo => "Iniziamo con i tuoi dati",
            RegistrationStep::AccountInfo => "Scegli username e email",
            RegistrationStep::Security => "Imposta la tua password",
        }
    };

    let render_step_content = move || {
        match current_step.get() {
            RegistrationStep::PersonalInfo => view! {
                <div class="space-y-3 p-4">
                    <div class="grid grid-cols-2 gap-2">
                        <div>
                            <label for="first_name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Nome *"
                            </label>
                            <input
                                type="text"
                                id="first_name"
                                class=get_input_class(first_name_error)
                                placeholder="Nome"
                                prop:value=first_name
                                on:input=move |ev| {
                                    set_first_name.set(event_target_value(&ev));
                                    // Clear error when user starts typing
                                    if first_name_error.get_untracked() {
                                        set_first_name_error.set(false);
                                    }
                                }
                            />
                        </div>
                        <div>
                            <label for="last_name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Cognome *"
                            </label>
                            <input
                                type="text"
                                id="last_name"
                                class=get_input_class(last_name_error)
                                placeholder="Cognome"
                                prop:value=last_name
                                on:input=move |ev| {
                                    set_last_name.set(event_target_value(&ev));
                                    // Clear error when user starts typing
                                    if last_name_error.get_untracked() {
                                        set_last_name_error.set(false);
                                    }
                                }
                            />
                        </div>
                    </div>

                    <div class="grid grid-cols-2 gap-2">
                        <div>
                            <label for="birthday" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Nascita *"
                            </label>
                            <input
                                type="date"
                                id="birthday"
                                class=get_input_class(birthday_error)
                                prop:value=birthday
                                max=move || chrono::Local::now().format("%Y-%m-%d").to_string()
                                on:input=move |ev| {
                                    set_birthday.set(event_target_value(&ev));
                                    // Clear error when user starts typing
                                    if birthday_error.get_untracked() {
                                        set_birthday_error.set(false);
                                    }
                                }
                            />
                        </div>
                        <div>
                            <label for="gender" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Genere *"
                            </label>
                            <select
                                id="gender"
                                class=get_input_class(gender_error)
                                on:change=move |ev| {
                                    set_gender.set(event_target_value(&ev));
                                    // Clear error when user makes a selection
                                    if gender_error.get_untracked() {
                                        set_gender_error.set(false);
                                    }
                                }
                            >
                                <option value="" selected=move || gender.get().is_empty()>"Seleziona"</option>
                                <option value="male" selected=move || gender.get() == "male">"M"</option>
                                <option value="female" selected=move || gender.get() == "female">"F"</option>
                                <option value="other" selected=move || gender.get() == "other">"Altro"</option>
                            </select>
                        </div>
                    </div>

                    <div>
                        <label for="address" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Indirizzo *"
                        </label>
                        <input
                            type="text"
                            id="address"
                            class=get_input_class(address_error)
                            placeholder="Via, Città"
                            prop:value=address
                            on:input=move |ev| {
                                set_address.set(event_target_value(&ev));
                                // Clear error when user starts typing
                                if address_error.get_untracked() {
                                    set_address_error.set(false);
                                }
                            }
                        />
                    </div>
                </div>
            }.into_view(),
            RegistrationStep::AccountInfo => view! {
                <div class="space-y-3">
                    <div>
                        <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Username *"
                        </label>
                        <input
                            type="text"
                            id="username"
                            class=get_input_class(username_error)
                            placeholder="Username"
                            prop:value=username
                            on:input=move |ev| {
                                set_username.set(event_target_value(&ev));
                                // Clear error when user starts typing
                                if username_error.get_untracked() {
                                    set_username_error.set(false);
                                }
                            }
                        />
                    </div>

                    <div>
                        <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Email *"
                        </label>
                        <input
                            type="email"
                            id="email"
                            class=get_input_class(email_error)
                            placeholder="email@esempio.com"
                            prop:value=email
                            on:input=move |ev| {
                                set_email.set(event_target_value(&ev));
                                // Clear error when user starts typing
                                if email_error.get_untracked() {
                                    set_email_error.set(false);
                                }
                            }
                        />
                    </div>
                </div>
            }.into_view(),
            RegistrationStep::Security => view! {
                <div class="space-y-3">
                    <div>
                        <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Password *"
                        </label>
                        <input
                            type="password"
                            id="password"
                            class=get_input_class(password_error)
                            placeholder="Min 6 caratteri"
                            prop:value=password
                            on:input=move |ev| {
                                set_password.set(event_target_value(&ev));
                                // Clear error when user starts typing
                                if password_error.get_untracked() {
                                    set_password_error.set(false);
                                }
                            }
                        />
                    </div>
                    <div>
                        <label for="confirm_password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Conferma Password *"
                        </label>
                        <input
                            type="password"
                            id="confirm_password"
                            class=get_input_class(confirm_password_error)
                            placeholder="Ripeti la password"
                            prop:value=confirm_password
                            on:input=move |ev| {
                                set_confirm_password.set(event_target_value(&ev));
                                // Clear error when user starts typing
                                if confirm_password_error.get_untracked() {
                                    set_confirm_password_error.set(false);
                                }
                            }
                        />
                    </div>
                </div>
            }.into_view(),
        }
    };

    view! {
        <div class="min-h-screen w-screen overflow-auto bg-cover bg-center bg-no-repeat transition-colors" style="background-image: url('/public/images/bg-landing-full.png');">
            // Overlay per migliorare la leggibilità del testo
            <div class="absolute inset-0 bg-black bg-opacity-50 dark:bg-opacity-70"></div>
            
            <div class="absolute top-3 left-4 right-4 z-20 flex items-center justify-between">
                <img 
                    src="/public/logos/logo-full-white.png" 
                    alt="Ruggine" 
                    class="h-12 w-auto drop-shadow-lg"
                />
                <ThemeToggle />
            </div>
            
            // Contenuto principale centrato  
            <main class="relative z-10 min-h-screen flex items-center justify-center px-4 py-6">
                <div class="max-w-sm w-full">
                    // Hero Section compatto
                    <div class="text-center mb-4">
                        <h1 class="text-2xl font-bold text-white mb-2 drop-shadow-lg">
                            "Registrati su Ruggine"
                        </h1>
                        <p class="text-white text-sm drop-shadow-md opacity-90">
                            {get_step_subtitle}
                        </p>
                    </div>

                    // Register Form Card compatto
                    <div class="bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 transition-colors">
                        <div class="p-4">
                            {/* Header con progress */}
                            <div class="text-center mb-4">
                                <div class="flex justify-center mb-2">
                                    <div class="flex items-center space-x-1">
                                        {[1, 2, 3].into_iter().map(|step| {
                                            let is_active = move || {
                                                let current = match current_step.get() {
                                                    RegistrationStep::PersonalInfo => 1,
                                                    RegistrationStep::AccountInfo => 2,
                                                    RegistrationStep::Security => 3,
                                                };
                                                step <= current
                                            };
                                            view! {
                                                <div class={move || format!("w-2 h-2 rounded-full {}", 
                                                    if is_active() { "bg-brand-primary" } else { "bg-gray-300" }
                                                )}></div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                                <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">
                                    {get_step_title}
                                </h2>
                            </div>

                            // Error Message
                            {move || error_message.get().map(|msg| view! {
                                <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-800 dark:text-red-200 px-3 py-2 mb-3 rounded-md text-sm">
                                    {msg}
                                </div>
                            })}

                            // Contenuto dello step
                            <div class="mb-4">
                                {render_step_content}
                            </div>

                            // Bottoni di navigazione
                            <div class="flex justify-between space-x-2">
                                <Show when=move || current_step.get() != RegistrationStep::PersonalInfo>
                                    <button
                                        type="button"
                                        on:click=handle_back
                                        class="px-4 py-2.5 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 disabled:opacity-50"
                                    >
                                        "Indietro"
                                    </button>
                                </Show>
                                
                                <div class="flex-1"></div>
                                
                                <button
                                    type="button"
                                    on:click=handle_next
                                    disabled=move || loading.get()
                                    class="bg-brand-primary dark:bg-brand-primary-dark hover:bg-brand-primary-light dark:hover:bg-brand-primary text-white py-2.5 px-4 rounded-md text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                >
                                    {move || {
                                        if loading.get() {
                                            view! {
                                                <div class="flex items-center justify-center">
                                                    <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                                    </svg>
                                                    <span>"Registrando..."</span>
                                                </div>
                                            }.into_view()
                                        } else if current_step.get() == RegistrationStep::Security {
                                            view! { <span>"Completa Registrazione"</span> }.into_view()
                                        } else {
                                            view! { <span>"Continua"</span> }.into_view()
                                        }
                                    }}
                                </button>
                            </div>
                        </div>
                        
                        // Footer compatto
                        <div class="px-4 pb-4 border-t border-gray-200 dark:border-gray-700 pt-3">
                            <div class="text-center text-sm">
                                <span class="text-gray-600 dark:text-gray-400">"Hai già un account? "</span>
                                <button 
                                    type="button"
                                    class="text-brand-primary hover:text-brand-primary-light dark:text-brand-primary-light dark:hover:text-brand-primary font-medium"
                                    on:click={
                                        let navigate = navigate_clone.clone();
                                        move |_| {
                                            navigate("/login", Default::default());
                                        }
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
