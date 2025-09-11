use leptos::*;
use crate::utils::StorageService;
use crate::api::services::AuthService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;
use crate::context::auth_context::use_auth_context;
use gloo_timers::future::TimeoutFuture;

/// Hook per gestire il refresh automatico del token quando Remember Me è attivo
#[component]
pub fn RememberMeRefreshProvider(children: Children) -> impl IntoView {
    let auth_ctx = use_auth_context();
    let storage_service = StorageService::new();
    
    // Controlla periodicamente se il token deve essere rinnovato
    create_effect(move |_| {
        let is_active = storage_service.is_remember_me_active();
        log::info!("RememberMeRefreshProvider: is_remember_me_active = {}", is_active);
        
        if is_active {
            log::info!("RememberMeRefreshProvider: Avvio loop di refresh automatico");
            let storage_clone = storage_service.clone();
            let auth_clone = auth_ctx.clone();
            spawn_local(async move {
                loop {
                    // Calcola l'intervallo di controllo basato sulla durata del token
                    let check_interval_ms = calculate_check_interval(&storage_clone);
                    log::info!("RememberMeRefreshProvider: prossimo controllo tra {}ms", check_interval_ms);
                    TimeoutFuture::new(check_interval_ms).await;
                    
                    log::info!("RememberMeRefreshProvider: controllo token...");
                    if let Some(token_response) = storage_clone.get_token() {
                        log::info!("RememberMeRefreshProvider: token trovato, exp={}, iat={}", token_response.exp, token_response.iat);
                        
                        // Controlla se il token scade nelle prossime 2 ore (o nella metà rimanente se < 4h)
                        if should_refresh_token(&token_response) {
                            log::info!("RememberMeRefreshProvider: token necessita refresh, provo login automatico");
                            if let Some((email, password)) = storage_clone.get_remember_me_credentials() {
                                let _ = refresh_token_silently(&email, &password, &storage_clone, &auth_clone).await;
                            } else {
                                log::warn!("RememberMeRefreshProvider: Remember Me attivo ma credenziali mancanti");
                            }
                        } else {
                            log::info!("RememberMeRefreshProvider: token ancora valido");
                        }
                    } else {
                        log::info!("RememberMeRefreshProvider: nessun token trovato, provo login automatico");
                        // Se non c'è token ma Remember Me è attivo, prova a fare login automatico
                        if let Some((email, password)) = storage_clone.get_remember_me_credentials() {
                            let _ = refresh_token_silently(&email, &password, &storage_clone, &auth_clone).await;
                        } else {
                            log::warn!("RememberMeRefreshProvider: Remember Me attivo ma credenziali mancanti");
                        }
                    }
                }
            });
        }
    });
    
    children()
}

pub fn should_refresh_token(token_response: &crate::types::TokenResponse) -> bool {
    // Usa i metodi del TokenResponse per controllare la scadenza
    if token_response.is_expired() {
        // Token già scaduto
        true
    } else {
        let time_to_expiry = token_response.time_to_expiry();
        
        // Logica adattiva basata sulla durata totale del token
        let total_duration = token_response.exp - token_response.iat;
        
        if total_duration <= 3600 {
            // Token dura <= 1 ora: refresh quando rimane meno del 25% (15 minuti per 1h)
            time_to_expiry < (total_duration / 4).max(300) // Minimo 5 minuti
        } else if total_duration <= 7200 {
            // Token dura <= 2 ore: refresh quando rimane meno del 33% 
            time_to_expiry < (total_duration / 3).max(600) // Minimo 10 minuti
        } else {
            // Token dura > 2 ore: usa la logica originale (2 ore)
            time_to_expiry < 7200
        }
    }
}

/// Calcola l'intervallo di controllo basato sulla durata del token
fn calculate_check_interval(storage_service: &StorageService) -> u32 {
    if let Some(token) = storage_service.get_token() {
        let total_duration = token.exp - token.iat;
        
        if total_duration <= 900 {
            // Token <= 15 minuti: controlla ogni 2 minuti
            2 * 60 * 1000
        } else if total_duration <= 3600 {
            // Token <= 1 ora: controlla ogni 5 minuti
            5 * 60 * 1000
        } else if total_duration <= 7200 {
            // Token <= 2 ore: controlla ogni 15 minuti
            15 * 60 * 1000
        } else if total_duration <= 14400 {
            // Token <= 4 ore: controlla ogni 30 minuti
            30 * 60 * 1000
        } else {
            // Token > 4 ore: controlla ogni ora
            60 * 60 * 1000
        }
    } else {
        // Nessun token: controlla ogni 5 minuti per il login automatico
        5 * 60 * 1000
    }
}

async fn refresh_token_silently(
    email: &str, 
    password: &str, 
    storage_service: &StorageService,
    auth_ctx: &crate::context::auth_context::AuthContext
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("refresh_token_silently: Tentativo login automatico per email: {}", email);
    
    let auth_service = AuthService::new(
        ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
        storage_service.clone(),
    );
    
    match auth_service.login(email.to_string(), password.to_string()).await {
        Ok(user_profile) => {
            log::info!("refresh_token_silently: Login automatico riuscito");
            
            // Ottieni il token dalla risposta del login
            if let Some(token) = auth_service.get_storage_service().get_token() {
                // Cancella il token temporaneo salvato da AuthService
                let _ = auth_service.get_storage_service().clear_session();
                
                // Salva il nuovo token in modo persistente (Remember Me è sempre true qui)
                let _ = storage_service.store_token_with_remember_me(&token, true);
                
                // Aggiorna il context con il nuovo token
                auth_ctx.token.set(Some(token.token.clone()));
            }
            
            // Salva il profilo utente con Remember Me
            let _ = storage_service.store_user_profile_with_remember_me(&user_profile, true);
            
            // Aggiorna il context con il nuovo profilo
            auth_ctx.user_profile.set(Some(user_profile.clone()));

            // Verifica se il profilo è stato salvato
            if let Some(profile) = storage_service.get_user_profile() {
                log::info!("refresh_token_silently: Profilo utente caricato correttamente: {} {}", 
                    profile.first_name, profile.last_name);
            } else {
                log::warn!("refresh_token_silently: Login riuscito ma profilo non trovato in storage!");
            }
            
            log::info!("Token rinnovato automaticamente con successo");
            Ok(())
        }
        Err(e) => {
            log::error!("refresh_token_silently: Login automatico fallito: {:?}", e);
            // Se il refresh fallisce, cancella i dati Remember Me
            let _ = storage_service.clear_remember_me();
            Err(Box::new(e))
        }
    }
}

/// Hook semplificato per inizializzare Remember Me all'avvio
pub fn use_remember_me_init() {
    log::info!("use_remember_me_init: Inizializzazione Remember Me avviata");
    
    let storage_service = StorageService::new();
    let auth_ctx = use_auth_context();
    
    // Esegui la logica solo una volta al caricamento dell'app, non come effect reattivo
    spawn_local(async move {
        let is_active = storage_service.is_remember_me_active();
        log::info!("use_remember_me_init: is_remember_me_active = {}", is_active);
        
        // Verifica anche se abbiamo le credenziali
        if let Some((email, _)) = storage_service.get_remember_me_credentials() {
            log::info!("use_remember_me_init: Credenziali Remember Me trovate per email: {}", email);
        } else {
            log::info!("use_remember_me_init: Nessuna credenziale Remember Me trovata");
        }
        
        // Controlla sempre se c'è un token (indipendentemente da Remember Me)
        if let Some(token_response) = storage_service.get_token() {
            log::info!("use_remember_me_init: token trovato (exp={}), aggiorno context", token_response.exp);
            // Se abbiamo un token salvato, aggiorna il context
            let token_string = token_response.token.clone(); // Clone per evitare move
            auth_ctx.token.set(Some(token_string.clone()));
            
            // Aggiorna anche il profilo se presente in storage
            if let Some(profile) = storage_service.get_user_profile() {
                log::info!("use_remember_me_init: profilo trovato in storage, aggiorno context: {} {}", 
                    profile.first_name, profile.last_name);
                auth_ctx.user_profile.set(Some(profile));
            } else {
                log::warn!("use_remember_me_init: token presente ma profilo mancante in storage");
                
                // Se abbiamo un token ma non il profilo, proviamo a recuperare il profilo dal server
                // usando il token che abbiamo - questo può accadere quando:
                // 1. Il logout non ha cancellato il token ma ha rimosso il profilo
                // 2. C'è stato un problema nel salvataggio del profilo
                // 3. L'utente ha cancellato manualmente i dati del profilo
                
                log::info!("use_remember_me_init: tentativo di recupero profilo dal server usando token esistente");
                let user_service = crate::api::services::user::UserService::new(
                    ApiClient::new(AppConstants::DEFAULT_SERVER_URL),
                    storage_service.clone(),
                );
                
                // Impostiamo temporaneamente il token nel context per poter fare la chiamata API
                auth_ctx.token.set(Some(token_string.clone()));
                
                // Tentativi di recuperare il profilo dal server
                match user_service.get_current_profile().await {
                    Ok(profile) => {
                        log::info!("use_remember_me_init: profilo recuperato dal server: {} {}", 
                            profile.first_name, profile.last_name);
                        
                        // Salviamo il profilo in storage
                        let _ = storage_service.store_user_profile(&profile);
                        
                        // Aggiorniamo il context
                        auth_ctx.user_profile.set(Some(profile));
                        
                        log::info!("use_remember_me_init: token e profilo ripristinati con successo");
                    }
                    Err(e) => {
                        log::error!("use_remember_me_init: impossibile recuperare profilo dal server: {:?}", e);
                        
                        // Se il token non è valido o il server restituisce errore,
                        // proviamo il login automatico se disponibile
                        if is_active && storage_service.get_remember_me_credentials().is_some() {
                            log::info!("use_remember_me_init: provo login automatico come fallback");
                            let (email, password) = storage_service.get_remember_me_credentials().unwrap();
                            let storage_clone = storage_service.clone();
                            let auth_clone = auth_ctx.clone();
                            
                            match refresh_token_silently(&email, &password, &storage_clone, &auth_clone).await {
                                Ok(_) => {
                                    log::info!("use_remember_me_init: login automatico riuscito come fallback");
                                }
                                Err(_) => {
                                    log::warn!("use_remember_me_init: login automatico fallito, rimuovo token");
                                    let _ = storage_service.clear_session();
                                    auth_ctx.token.set(None);
                                }
                            }
                        } else {
                            log::warn!("use_remember_me_init: nessuna credenziale Remember Me disponibile, rimuovo token");
                            let _ = storage_service.clear_session();
                            auth_ctx.token.set(None);
                        }
                    }
                }
            }
        } else if is_active {
            log::info!("use_remember_me_init: Remember Me attivo ma nessun token, provo login automatico");
            // Se Remember Me è attivo ma non c'è token, prova login automatico
            if let Some((email, password)) = storage_service.get_remember_me_credentials() {
                log::info!("use_remember_me_init: credenziali trovate, avvio login automatico per {}", email);
                let storage_clone = storage_service.clone();
                let auth_clone = auth_ctx.clone();
                let _ = refresh_token_silently(&email, &password, &storage_clone, &auth_clone).await;
            } else {
                log::warn!("use_remember_me_init: Remember Me attivo ma credenziali mancanti");
            }
        } else {
            log::info!("use_remember_me_init: Remember Me non attivo e nessun token presente");
        }
    });
}
