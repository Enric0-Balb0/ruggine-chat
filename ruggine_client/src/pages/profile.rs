
use leptos::*;
use leptos_router::*;
use crate::api::services::user::{UserService, UserProfileUpdate};
use crate::api::services::AuthService;
use crate::utils::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;
use crate::components::{ThemeToggle, use_toast};

#[component]
pub fn ProfilePage() -> impl IntoView {
    let navigate = use_navigate();

    // Signals per i dati utente
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (address, set_address) = create_signal(String::new());
    let (birthday, set_birthday) = create_signal(String::new());
    let (gender, set_gender) = create_signal(String::new());
    // Username signals rimossi
    let (original_first_name, set_original_first_name) = create_signal(String::new());
    let (original_last_name, set_original_last_name) = create_signal(String::new());
    let (original_address, set_original_address) = create_signal(String::new());
    let (original_birthday, set_original_birthday) = create_signal(String::new());
    let (original_gender, set_original_gender) = create_signal(String::new());
    let (loading_username, set_loading_username) = create_signal(false);
    let (loading_profile, set_loading_profile) = create_signal(false);
    let (error_username, set_error_username) = create_signal(Option::<String>::None);
    let (error_profile, set_error_profile) = create_signal(Option::<String>::None);
    let (success_username, set_success_username) = create_signal(Option::<String>::None);
    let (success_profile, set_success_profile) = create_signal(Option::<String>::None);

    // Carica dati utente all'apertura
    create_effect(move |_| {
        let storage = StorageService::new();
        if let Some(profile) = storage.get_user_profile() {
            set_first_name.set(profile.first_name.clone());
            set_last_name.set(profile.last_name.clone());
            set_address.set(profile.address.clone());
            set_birthday.set(profile.birthday.to_string());
            set_gender.set(profile.gender.to_string());
            set_original_first_name.set(profile.first_name);
            set_original_last_name.set(profile.last_name);
            set_original_address.set(profile.address);
            set_original_birthday.set(profile.birthday.to_string());
            set_original_gender.set(profile.gender.to_string());
        }
    });

    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        StorageService::new(),
    );

    // Sezione username rimossa


    // Cambia dati anagrafici
    let handle_profile = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading_profile.set(true);
        set_error_profile.set(None);
        set_success_profile.set(None);
        let storage_service = StorageService::new();
        let mut http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        if let Some(token_response) = storage_service.get_token() {
            http_client.set_auth_token(Some(token_response.token));
        }
        let user_service = UserService::new(http_client, storage_service);
        let first_name = first_name.get();
        let last_name = last_name.get();
        let address = address.get();
        let birthday = birthday.get();
        let gender = gender.get();
        let original_first_name = original_first_name.get();
        let original_last_name = original_last_name.get();
        let original_address = original_address.get();
        let original_birthday = original_birthday.get();
        let original_gender = original_gender.get();
        if first_name == original_first_name && last_name == original_last_name && address == original_address && birthday == original_birthday && gender == original_gender {
            set_error_profile.set(Some("Modifica almeno un campo prima di salvare.".to_string()));
            set_loading_profile.set(false);
            return;
        }
        spawn_local(async move {
            let toast = crate::components::use_toast();
            let update = UserProfileUpdate {
                first_name: Some(first_name.clone()),
                last_name: Some(last_name.clone()),
                address: Some(address.clone()),
                birthday: Some(birthday.clone()),
                gender: Some(gender.clone()),
            };
            match user_service.update_profile(update).await {
                Ok(_) => {
                    set_success_profile.set(Some("Dati anagrafici aggiornati con successo!".to_string()));
                    toast.success("Dati anagrafici aggiornati!");
                    set_original_first_name.set(first_name);
                    set_original_last_name.set(last_name);
                    set_original_address.set(address);
                    set_original_birthday.set(birthday);
                    set_original_gender.set(gender);
                },
                Err(e) => {
                    set_error_profile.set(Some(format!("Errore: {}", e)));
                }
            }
            set_loading_profile.set(false);
        });
    };

    view! {
    <div class="min-h-screen w-screen overflow-auto bg-cover bg-center bg-no-repeat transition-colors" style="background-image: url('public/images/bg-landing-full.png');">
        <div class="absolute inset-0 bg-black bg-opacity-50 dark:bg-opacity-70"></div>
        <div class="absolute top-4 left-8 right-8 z-20 flex items-center justify-between">
            <img 
                src="public/logos/logo-full-white.png" 
                alt="Ruggine" 
                class="h-24 w-auto drop-shadow-lg"
            />
            <ThemeToggle />
        </div>
        <main class="relative z-10 min-h-screen flex items-center justify-center px-6 py-8 overflow-auto" style="min-height: 100vh; max-height: 100vh;">
            <div class="max-w-2xl w-full px-4">
                <div class="text-center mb-6">
                    <h1 class="text-3xl font-bold text-white mb-3 drop-shadow-lg">
                        Il Tuo Profilo
                    </h1>
                    <p class="text-white text-base drop-shadow-md opacity-90 mt-8">
                        Gestisci le tue informazioni personali
                    </p>
                </div>
                <div class="bg-white/95 dark:bg-surface-dark/95 backdrop-blur-sm rounded-lg shadow-xl border border-border dark:border-border-dark transition-colors">
                    <div class="form-container-scroll px-6 pb-6 pt-6 space-y-10">
                        <form on:submit=handle_profile class="space-y-6 w-full" autocomplete="off">
                            <h2 class="text-xl font-semibold text-text-primary dark:text-text-primary-dark mb-4 text-center">Dati anagrafici</h2>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div>
                                    <label class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">Nome</label>
                                    <input
                                        type="text"
                                        required
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        placeholder="Nome"
                                        prop:value=first_name
                                        on:input=move |ev| set_first_name.set(event_target_value(&ev))
                                        disabled=move || loading_profile.get()
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">Cognome</label>
                                    <input
                                        type="text"
                                        required
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        placeholder="Cognome"
                                        prop:value=last_name
                                        on:input=move |ev| set_last_name.set(event_target_value(&ev))
                                        disabled=move || loading_profile.get()
                                    />
                                </div>
                                <div class="md:col-span-2">
                                    <label class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">Indirizzo</label>
                                    <input
                                        type="text"
                                        required
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        placeholder="Indirizzo"
                                        prop:value=address
                                        on:input=move |ev| set_address.set(event_target_value(&ev))
                                        disabled=move || loading_profile.get()
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">Data di nascita</label>
                                    <input
                                        type="date"
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        prop:value=birthday
                                        on:input=move |ev| set_birthday.set(event_target_value(&ev))
                                        disabled=move || loading_profile.get()
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-text-secondary dark:text-text-secondary-dark mb-1">Genere</label>
                                    <select
                                        class="w-full px-3 py-2 text-sm border border-border dark:border-border-dark rounded-md bg-white dark:bg-surface-dark text-text-primary dark:text-text-primary-dark focus:outline-none focus:ring-2 focus:ring-accent dark:focus:ring-accent-dark transition-colors focus:border-transparent"
                                        prop:value=gender
                                        on:input=move |ev| set_gender.set(event_target_value(&ev))
                                        disabled=move || loading_profile.get()
                                    >
                                        <option value="">Seleziona...</option>
                                        <option value="male">Maschio</option>
                                        <option value="female">Femmina</option>
                                        <option value="other">Altro</option>
                                    </select>
                                </div>
                            </div>
                            {move || error_profile.get().map(|msg| view! {
                                <div class="bg-error-light dark:bg-error-dark border border-error dark:border-error text-error-dark dark:text-error-light px-3 py-2 rounded-md text-sm text-center">{msg}</div>
                            })}
                            <button type="submit"
                                disabled=move || loading_profile.get() || gender.get().is_empty() || (
                                    first_name.get() == original_first_name.get() &&
                                    last_name.get() == original_last_name.get() &&
                                    address.get() == original_address.get() &&
                                    birthday.get() == original_birthday.get() &&
                                    gender.get() == original_gender.get()
                                )
                                class="w-full bg-brand-primary dark:bg-brand-primary-dark hover:bg-brand-primary-light dark:hover:bg-brand-primary disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium py-2.5 px-4 rounded-md transition-colors text-sm">
                                {move || if loading_profile.get() { "Salvataggio..." } else { "Modifica Informazioni" }}
                            </button>
                        </form>
                    </div>
                </div>
            </div>
        </main>
    </div>
}
}
