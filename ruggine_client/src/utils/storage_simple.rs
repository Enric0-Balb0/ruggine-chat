use serde::{Deserialize, Serialize};
use crate::config::storage::{StorageKeys, StorageConfig};
use crate::types::{TokenResponse, UserProfile};
use crate::error::StorageError;

#[derive(Clone, Debug)]
pub struct StorageService;

impl StorageService {
    pub fn new() -> Self {
        Self
    }

    /// Store authentication token in localStorage (persistent)
    pub fn store_token(&self, token: &TokenResponse) -> Result<(), StorageError> {
        let token_json = serde_json::to_string(token)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        self.set_item(StorageKeys::AUTH_TOKEN, &token_json)
    }

    /// Retrieve authentication token from localStorage
    pub fn get_token(&self) -> Option<TokenResponse> {
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
    pub fn remove_token(&self) -> Result<(), StorageError> {
        self.remove_item(StorageKeys::AUTH_TOKEN);
        Ok(())
    }

    /// Store user profile in localStorage (persistent)
    pub fn store_user_profile(&self, user: &UserProfile) -> Result<(), StorageError> {
        log::info!("StorageService::store_user_profile: Salvando profilo per {} {} (email: {})", 
            user.first_name, user.last_name, user.email);
        
        let user_json = serde_json::to_string(user)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        let result = self.set_item(StorageKeys::USER_PROFILE, &user_json);
        
        if result.is_ok() {
            log::info!("StorageService::store_user_profile: Profilo salvato con successo in localStorage");
            
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

    /// Retrieve user profile from localStorage
    pub fn get_user_profile(&self) -> Option<UserProfile> {
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
    pub fn remove_user_profile(&self) -> Result<(), StorageError> {
        self.remove_item(StorageKeys::USER_PROFILE);
        Ok(())
    }

    /// Clear all stored data
    pub fn clear_all(&self) -> Result<(), StorageError> {
        self.remove_item(StorageKeys::AUTH_TOKEN);
        self.remove_item(StorageKeys::USER_PROFILE);
        self.remove_session_item(StorageKeys::AUTH_TOKEN);
        self.remove_session_item(StorageKeys::USER_PROFILE);
        Ok(())
    }

    /// Clear user session data
    pub fn clear_session(&self) -> Result<(), StorageError> {
        // Rimuovi i dati dalla session e localStorage
        self.remove_session_item(StorageKeys::AUTH_TOKEN);
        self.remove_session_item(StorageKeys::USER_PROFILE);
        self.remove_item(StorageKeys::AUTH_TOKEN);
        self.remove_item(StorageKeys::USER_PROFILE);
        Ok(())
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
    fn set_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        use web_sys::window;
        
        let window = window().ok_or_else(|| StorageError::AccessFailed("No window available".to_string()))?;
        let storage = window
            .local_storage()
            .map_err(|_| StorageError::AccessFailed("Failed to access localStorage".to_string()))?
            .ok_or_else(|| StorageError::NotAvailable)?;
        
        storage
            .set_item(key, value)
            .map_err(|_| StorageError::AccessFailed(format!("Failed to set item {}", key)))
    }

    #[cfg(target_arch = "wasm32")]
    fn get_item(&self, key: &str) -> Option<String> {
        use web_sys::window;
        
        let window = window()?;
        let storage = window.local_storage().ok()??;
        storage.get_item(key).ok()?
    }

    #[cfg(target_arch = "wasm32")]
    fn remove_item(&self, key: &str) {
        use web_sys::window;
        
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item(key);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn set_session_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        use web_sys::window;
        
        let window = window().ok_or_else(|| StorageError::AccessFailed("No window available".to_string()))?;
        let storage = window
            .session_storage()
            .map_err(|_| StorageError::AccessFailed("Failed to access sessionStorage".to_string()))?
            .ok_or_else(|| StorageError::NotAvailable)?;
        
        storage
            .set_item(key, value)
            .map_err(|_| StorageError::AccessFailed(format!("Failed to set session item {}", key)))
    }

    #[cfg(target_arch = "wasm32")]
    fn get_session_item(&self, key: &str) -> Option<String> {
        use web_sys::window;
        
        let window = window()?;
        let storage = window.session_storage().ok()??;
        storage.get_item(key).ok()?
    }

    #[cfg(target_arch = "wasm32")]
    fn remove_session_item(&self, key: &str) {
        use web_sys::window;
        
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.session_storage() {
                let _ = storage.remove_item(key);
            }
        }
    }

    // Test environment implementation
    #[cfg(not(target_arch = "wasm32"))]
    fn set_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        if let Ok(mut storage_write) = get_test_storage().write() {
            storage_write.insert(key.to_string(), value.to_string());
            Ok(())
        } else {
            Err(StorageError::AccessFailed("Failed to acquire storage lock".to_string()))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_item(&self, key: &str) -> Option<String> {
        if let Ok(storage_read) = get_test_storage().read() {
            storage_read.get(key).cloned()
        } else {
            None
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn remove_item(&self, key: &str) {
        if let Ok(mut storage_write) = get_test_storage().write() {
            storage_write.remove(key);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn set_session_item(&self, key: &str, value: &str) -> Result<(), StorageError> {
        // In test environment, session and local storage are the same
        self.set_item(key, value)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_session_item(&self, key: &str) -> Option<String> {
        // In test environment, session and local storage are the same
        self.get_item(key)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn remove_session_item(&self, key: &str) {
        // In test environment, session and local storage are the same
        self.remove_item(key)
    }
}

// Test environment storage
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;
#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;

#[cfg(not(target_arch = "wasm32"))]
static TEST_STORAGE: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

#[cfg(not(target_arch = "wasm32"))]
fn get_test_storage() -> &'static RwLock<HashMap<String, String>> {
    &TEST_STORAGE
}
