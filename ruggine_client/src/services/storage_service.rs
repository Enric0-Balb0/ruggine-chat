// Storage Service - Client-side data persistence
// Handles localStorage, sessionStorage, and caching

use crate::dto::{TokenResponse, UserProfile};
use crate::error::StorageError;
use crate::config::storage::StorageKeys;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct StorageService;

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
        let token_json = self.get_item(StorageKeys::AUTH_TOKEN)?;
        serde_json::from_str(&token_json).ok()
    }

    /// Remove authentication token
    pub fn remove_token(&self) {
        self.remove_item(StorageKeys::AUTH_TOKEN);
    }

    /// Store user profile
    pub fn store_user_profile(&self, user: &UserProfile) -> Result<(), StorageError> {
        let user_json = serde_json::to_string(user)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        self.set_item(StorageKeys::USER_PROFILE, &user_json)
    }

    /// Retrieve user profile
    pub fn get_user_profile(&self) -> Option<UserProfile> {
        let user_json = self.get_item(StorageKeys::USER_PROFILE)?;
        serde_json::from_str(&user_json).ok()
    }

    /// Remove user profile
    pub fn remove_user_profile(&self) {
        self.remove_item(StorageKeys::USER_PROFILE);
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
