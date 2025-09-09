// Storage Service - Client-side data persistence
// Handles localStorage, sessionStorage, and caching

use crate::types::{TokenResponse, UserProfile};
use crate::error::StorageError;
use crate::config::storage::StorageKeys;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};

#[derive(Clone, Debug)]
pub struct StorageService;

#[derive(Serialize, Deserialize, Debug)]
struct RememberMeCredentials {
    email: String,
    password: String, // encrypted
    timestamp: u64,
}

impl StorageService {
    pub fn new() -> Self {
        Self
    }

    /// Store authentication token
    pub fn store_token(&self, token: &TokenResponse) -> Result<(), StorageError> {
        let token_json = serde_json::to_string(token)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        self.set_item(StorageKeys::AUTH_TOKEN, &token_json)
    }

    /// Retrieve authentication token
    pub fn get_token(&self) -> Option<TokenResponse> {
        // Prima prova sessionStorage (per token temporanei)
        if let Some(token_json) = self.get_session_item(StorageKeys::AUTH_TOKEN) {
            if let Ok(token) = serde_json::from_str(&token_json) {
                log::debug!("StorageService::get_token: Token trovato in sessionStorage");
                return Some(token);
            }
        }
        
        // Poi prova localStorage (per token persistenti con Remember Me)
        if let Some(token_json) = self.get_item(StorageKeys::AUTH_TOKEN) {
            if let Ok(token) = serde_json::from_str(&token_json) {
                log::debug!("StorageService::get_token: Token trovato in localStorage");
                return Some(token);
            }
        }
        
        log::debug!("StorageService::get_token: Nessun token trovato");
        None
    }

    /// Remove authentication token
    pub fn remove_token(&self) {
        // Rimuovi da entrambi i storage per sicurezza
        self.remove_item(StorageKeys::AUTH_TOKEN);
        self.remove_session_item(StorageKeys::AUTH_TOKEN);
    }

    /// Store user profile
    pub fn store_user_profile(&self, user: &UserProfile) -> Result<(), StorageError> {
        log::info!("StorageService::store_user_profile: Salvando profilo per {} {} (email: {})", 
            user.first_name, user.last_name, user.email);
        
        let user_json = serde_json::to_string(user)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        let result = self.set_item(StorageKeys::USER_PROFILE, &user_json);
        
        if result.is_ok() {
            log::info!("StorageService::store_user_profile: Profilo salvato con successo");
            
            // Verifica immediatamente se il profilo è recuperabile
            if let Some(verified_profile) = self.get_user_profile() {
                log::info!("StorageService::store_user_profile: Verifica lettura OK - {} {}", 
                    verified_profile.first_name, verified_profile.last_name);
            } else {
                log::error!("StorageService::store_user_profile: ERRORE - Profilo salvato ma non leggibile!");
            }
        } else {
            log::error!("StorageService::store_user_profile: Errore nel salvataggio: {:?}", result);
        }
        
        result
    }

    /// Store user profile with remember me preference
    pub fn store_user_profile_with_remember_me(&self, user: &UserProfile, remember_me: bool) -> Result<(), StorageError> {
        log::info!("StorageService::store_user_profile_with_remember_me: Salvando profilo per {} {} (remember_me={})", 
            user.first_name, user.last_name, remember_me);
        
        let user_json = serde_json::to_string(user)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        let result = if remember_me {
            // Con Remember Me, salva in localStorage (persistente)
            log::info!("StorageService::store_user_profile_with_remember_me: Salvando in localStorage (persistente)");
            self.set_item(StorageKeys::USER_PROFILE, &user_json)
        } else {
            // Senza Remember Me, salva in sessionStorage (temporaneo)
            log::info!("StorageService::store_user_profile_with_remember_me: Salvando in sessionStorage (temporaneo)");
            self.set_session_item(StorageKeys::USER_PROFILE, &user_json)
        };
        
        if result.is_ok() {
            log::info!("StorageService::store_user_profile_with_remember_me: Profilo salvato con successo");
            
            // Verifica immediatamente se il profilo è recuperabile
            if let Some(verified_profile) = self.get_user_profile() {
                log::info!("StorageService::store_user_profile_with_remember_me: Verifica lettura OK - {} {}", 
                    verified_profile.first_name, verified_profile.last_name);
            } else {
                log::error!("StorageService::store_user_profile_with_remember_me: ERRORE - Profilo salvato ma non leggibile!");
            }
        } else {
            log::error!("StorageService::store_user_profile_with_remember_me: Errore nel salvataggio: {:?}", result);
        }
        
        result
    }

    /// Retrieve user profile
    pub fn get_user_profile(&self) -> Option<UserProfile> {
        // Prima prova sessionStorage (per profili temporanei)
        if let Some(user_json) = self.get_session_item(StorageKeys::USER_PROFILE) {
            match serde_json::from_str::<UserProfile>(&user_json) {
                Ok(profile) => {
                    log::debug!("StorageService::get_user_profile: Profilo trovato in sessionStorage - {} {} (email: {})", 
                        profile.first_name, profile.last_name, profile.email);
                    return Some(profile);
                },
                Err(e) => {
                    log::error!("StorageService::get_user_profile: Errore deserializzazione profilo da sessionStorage: {}", e);
                }
            }
        }
        
        // Poi prova localStorage (per profili persistenti con Remember Me)
        if let Some(user_json) = self.get_item(StorageKeys::USER_PROFILE) {
            match serde_json::from_str::<UserProfile>(&user_json) {
                Ok(profile) => {
                    log::debug!("StorageService::get_user_profile: Profilo trovato in localStorage - {} {} (email: {})", 
                        profile.first_name, profile.last_name, profile.email);
                    return Some(profile);
                },
                Err(e) => {
                    log::error!("StorageService::get_user_profile: Errore deserializzazione profilo da localStorage: {}", e);
                }
            }
        }
        
        log::debug!("StorageService::get_user_profile: Nessun profilo trovato");
        None
    }

    /// Remove user profile
    pub fn remove_user_profile(&self) {
        // Rimuovi da entrambi i storage per sicurezza
        self.remove_item(StorageKeys::USER_PROFILE);
        self.remove_session_item(StorageKeys::USER_PROFILE);
    }

    /// Clear all stored data
    pub fn clear_all(&self) {
        self.remove_token();
        self.remove_user_profile();
    }

    /// Clear user session (alias for clear_all)
    pub fn clear_session(&self) -> Result<(), StorageError> {
        self.clear_all();
        Ok(())
    }

    // Remember Me functionality
    
    /// Enable Remember Me with encrypted credentials storage
    pub fn set_remember_me(&self, email: &str, password: &str, enabled: bool) -> Result<(), StorageError> {
        if enabled {
            // Store encrypted credentials
            let credentials = RememberMeCredentials {
                email: email.to_string(),
                password: self.simple_encrypt(password),
                timestamp: js_sys::Date::now() as u64,
            };
            
            let credentials_json = serde_json::to_string(&credentials)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            
            self.set_item(StorageKeys::REMEMBER_ME_CREDENTIALS, &credentials_json)?;
            
            // Set expiry date (30 giorni da ora)
            let expiry = js_sys::Date::now() as u64 + (crate::config::storage::StorageConfig::REMEMBER_ME_DURATION_DAYS * 24 * 60 * 60 * 1000);
            self.set_item(StorageKeys::REMEMBER_ME_EXPIRY, &expiry.to_string())?;
            
            self.set_item(StorageKeys::REMEMBER_ME_ENABLED, "true")
        } else {
            self.clear_remember_me()
        }
    }
    
    /// Check if Remember Me is enabled and not expired
    pub fn is_remember_me_active(&self) -> bool {
        if let Some(enabled) = self.get_item(StorageKeys::REMEMBER_ME_ENABLED) {
            if enabled == "true" {
                if let Some(expiry_str) = self.get_item(StorageKeys::REMEMBER_ME_EXPIRY) {
                    if let Ok(expiry) = expiry_str.parse::<u64>() {
                        return (js_sys::Date::now() as u64) < expiry;
                    }
                }
            }
        }
        false
    }
    
    /// Get saved credentials if Remember Me is active
    pub fn get_remember_me_credentials(&self) -> Option<(String, String)> {
        if !self.is_remember_me_active() {
            return None;
        }
        
        let credentials_json = self.get_item(StorageKeys::REMEMBER_ME_CREDENTIALS)?;
        let credentials: RememberMeCredentials = serde_json::from_str(&credentials_json).ok()?;
        
        let decrypted_password = self.simple_decrypt(&credentials.password);
        Some((credentials.email, decrypted_password))
    }
    
    /// Clear Remember Me data
    pub fn clear_remember_me(&self) -> Result<(), StorageError> {
        self.remove_item(StorageKeys::REMEMBER_ME_ENABLED);
        self.remove_item(StorageKeys::REMEMBER_ME_CREDENTIALS);
        self.remove_item(StorageKeys::REMEMBER_ME_EXPIRY);
        Ok(())
    }
    
    /// Store authentication token with Remember Me consideration
    pub fn store_token_with_remember_me(&self, token: &TokenResponse, remember_me: bool) -> Result<(), StorageError> {
        log::info!("StorageService::store_token_with_remember_me: remember_me={}", remember_me);
        
        if remember_me {
            // Se Remember Me è attivo, salva il token in localStorage (persistente)
            log::info!("StorageService::store_token_with_remember_me: Salvando in localStorage (persistente)");
            self.store_token(token)
        } else {
            // Altrimenti usa sessionStorage (non persistente)
            log::info!("StorageService::store_token_with_remember_me: Salvando in sessionStorage (temporaneo)");
            let token_json = serde_json::to_string(token)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
            
            let result = self.set_session_item(StorageKeys::AUTH_TOKEN, &token_json);
            
            if result.is_ok() {
                log::info!("StorageService::store_token_with_remember_me: Token salvato correttamente in sessionStorage");
                // Verifica immediatamente se il token è recuperabile
                if let Some(retrieved_token) = self.get_token() {
                    log::info!("StorageService::store_token_with_remember_me: Verifica lettura OK - exp={}", retrieved_token.exp);
                } else {
                    log::error!("StorageService::store_token_with_remember_me: ERRORE - Token salvato ma non leggibile!");
                }
            } else {
                log::error!("StorageService::store_token_with_remember_me: Errore nel salvataggio: {:?}", result);
            }
            
            result
        }
    }
    
    // Simple encryption for credentials (basic obfuscation)
    fn simple_encrypt(&self, text: &str) -> String {
        // Semplice XOR cipher per offuscare le password
        let key = b"ruggine_remember_me_key_2025";
        let encrypted: Vec<u8> = text.bytes()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()])
            .collect();
        general_purpose::STANDARD.encode(encrypted)
    }
    
    fn simple_decrypt(&self, encrypted: &str) -> String {
        if let Ok(encrypted_bytes) = general_purpose::STANDARD.decode(encrypted) {
            let key = b"ruggine_remember_me_key_2025";
            let decrypted: Vec<u8> = encrypted_bytes
                .iter()
                .enumerate()
                .map(|(i, &b)| b ^ key[i % key.len()])
                .collect();
            String::from_utf8_lossy(&decrypted).to_string()
        } else {
            String::new()
        }
    }

    /// Clear all storage for tests
    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_all_test_data(&self) {
        if let Ok(mut storage_write) = get_test_storage().write() {
            storage_write.clear();
        }
    }

    // Platform-specific storage implementation
    #[cfg(target_arch = "wasm32")]
    fn get_item(&self, key: &str) -> Option<String> {
        use web_sys::window;
        let storage = window()?.local_storage().ok()??;
        storage.get_item(key).ok()?
    }

    #[cfg(target_arch = "wasm32")]
    fn set_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        use web_sys::window;
        let storage = window()
            .and_then(|w| w.local_storage().ok()?)
            .ok_or(StorageError::NotAvailable)?;
        
        storage.set_item(key, value)
            .map_err(|_| StorageError::WriteError(key.to_string()))?;
        
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn remove_item(&self, key: &str) {
        use web_sys::window;
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.remove_item(key);
        }
    }

    // SessionStorage methods for non-persistent storage
    #[cfg(target_arch = "wasm32")]
    fn set_session_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        use web_sys::window;
        let storage = window()
            .and_then(|w| w.session_storage().ok()?)
            .ok_or(StorageError::NotAvailable)?;
        
        storage.set_item(key, value)
            .map_err(|_| StorageError::WriteError(key.to_string()))?;
        
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn get_session_item(&self, key: &str) -> Option<String> {
        use web_sys::window;
        let storage = window()?.session_storage().ok()??;
        storage.get_item(key).ok()?
    }

    #[cfg(target_arch = "wasm32")]
    fn remove_session_item(&self, key: &str) {
        use web_sys::window;
        if let Some(storage) = window().and_then(|w| w.session_storage().ok().flatten()) {
            let _ = storage.remove_item(key);
        }
    }

    // For non-WASM targets (testing), use in-memory storage
    #[cfg(not(target_arch = "wasm32"))]
    fn get_item(&self, key: &str) -> Option<String> {
        let storage_read = get_test_storage().read().ok()?;
        storage_read.get(key).cloned()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn set_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let mut storage_write = get_test_storage().write()
            .map_err(|_| StorageError::WriteError("Failed to acquire write lock".to_string()))?;
        
        storage_write.insert(key.to_string(), value.to_string());
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn remove_item(&self, key: &str) {
        if let Ok(mut storage_write) = get_test_storage().write() {
            storage_write.remove(key);
        }
    }

    // For testing - sessionStorage simulation
    #[cfg(not(target_arch = "wasm32"))]
    fn set_session_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        // In test environment, just use regular storage
        self.set_item(key, value)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_session_item(&self, key: &str) -> Option<String> {
        // In test environment, just use regular storage
        self.get_item(key)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn remove_session_item(&self, key: &str) {
        // In test environment, just use regular storage
        self.remove_item(key)
    }
}

// Helper function for test storage
#[cfg(not(target_arch = "wasm32"))]
fn get_test_storage() -> &'static std::sync::Arc<std::sync::RwLock<std::collections::HashMap<String, String>>> {
    use std::sync::{Arc, RwLock};
    use std::collections::HashMap;
    
    static STORAGE: std::sync::OnceLock<Arc<RwLock<HashMap<String, String>>>> = std::sync::OnceLock::new();
    STORAGE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

impl Default for StorageService {
    fn default() -> Self {
        Self::new()
    }
}
