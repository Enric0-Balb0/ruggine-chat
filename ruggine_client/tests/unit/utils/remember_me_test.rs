use ruggine_client_ui::utils::StorageService;
use ruggine_client_ui::types::TokenResponse;
use ruggine_client_ui::hooks::should_refresh_token;
use ruggine_client_ui::error::StorageError;
use wasm_bindgen_test::*;

// Configure for headless browser testing
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_remember_me_credentials_storage() {
    let storage = StorageService::new();
    
    // Pulisci lo storage prima del test
    let _ = storage.clear_remember_me();
    
    // Test storage credenziali
    let email = "test@example.com";
    let password = "password123";
    
    let result = storage.set_remember_me(email, password, true);
    assert!(result.is_ok(), "Should successfully store remember me credentials");
    
    // Test retrieval credenziali
    let retrieved = storage.get_remember_me_credentials();
    assert!(retrieved.is_some(), "Should retrieve stored credentials");
    
    let (retrieved_email, retrieved_password) = retrieved.unwrap();
    assert_eq!(retrieved_email, email, "Email should match");
    assert_eq!(retrieved_password, password, "Password should match");
    
    // Test che Remember Me sia attivo
    assert!(storage.is_remember_me_active(), "Remember Me should be active");
}

#[wasm_bindgen_test]
fn test_remember_me_expiration() {
    let storage = StorageService::new();
    
    // Pulisci lo storage prima del test
    let _ = storage.clear_remember_me();
    
    // Simula credenziali scadute modificando il timestamp salvato
    let email = "test@example.com";
    let password = "password123";
    
    // Prima salva normalmente
    let _ = storage.set_remember_me(email, password, true);
    assert!(storage.is_remember_me_active(), "Should be active initially");
    
    // Per testare la scadenza dovremmo modificare il timestamp,
    // ma per ora testiamo solo che il check non crashhi
    let credentials = storage.get_remember_me_credentials();
    assert!(credentials.is_some(), "Should still have credentials");
}

#[wasm_bindgen_test]
fn test_remember_me_clear() {
    let storage = StorageService::new();
    
    // Setup: salva credenziali
    let _ = storage.set_remember_me("test@example.com", "password123", true);
    assert!(storage.is_remember_me_active(), "Should be active initially");
    
    // Test clear
    let result = storage.clear_remember_me();
    assert!(result.is_ok(), "Should successfully clear remember me");
    
    // Verifica che sia tutto pulito
    assert!(!storage.is_remember_me_active(), "Should not be active after clear");
    assert!(storage.get_remember_me_credentials().is_none(), "Should have no credentials after clear");
}

#[wasm_bindgen_test]
fn test_remember_me_encryption() {
    let storage = StorageService::new();
    
    // Pulisci lo storage prima del test
    let _ = storage.clear_remember_me();
    
    let email = "test@example.com";
    let password = "supersecretpassword";
    
    // Salva credenziali
    let _ = storage.set_remember_me(email, password, true);
    
    // Verifica che i dati salvati nel localStorage siano criptati
    // (non dovrebbero contenere la password in chiaro)
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        if let Some(window) = window() {
            if let Some(local_storage) = window.local_storage().ok().flatten() {
                if let Some(stored_data) = local_storage.get_item("remember_me_credentials").ok().flatten() {
                    // I dati salvati non dovrebbero contenere la password in chiaro
                    assert!(!stored_data.contains(password), "Stored data should not contain plain password");
                    assert!(!stored_data.contains(email), "Stored data should not contain plain email");
                }
            }
        }
    }
    
    // Ma dovremmo poter recuperare i dati originali
    let (retrieved_email, retrieved_password) = storage.get_remember_me_credentials().unwrap();
    assert_eq!(retrieved_email, email);
    assert_eq!(retrieved_password, password);
}

#[wasm_bindgen_test]
fn test_dual_storage_token_management() {
    let storage = StorageService::new();
    
    // Pulisci lo storage prima del test
    let _ = storage.clear_remember_me();
    
    let now = chrono::Utc::now().timestamp();
    let token = TokenResponse {
        token: "test_token_dual".to_string(),
        iat: now,
        exp: now + 3600, // Scade tra 1 ora
    };
    
    // Test storage con Remember Me (dovrebbe andare in localStorage)
    let result1 = storage.store_token_with_remember_me(&token, true);
    assert!(result1.is_ok(), "Should store token with Remember Me");
    
    let retrieved1 = storage.get_token();
    assert!(retrieved1.is_some(), "Should retrieve token stored with Remember Me");
    assert_eq!(retrieved1.unwrap().token, token.token, "Token should match");
    
    // Clear e test storage senza Remember Me (dovrebbe andare in sessionStorage)
    let _ = storage.clear_session();
    
    let result2 = storage.store_token_with_remember_me(&token, false);
    assert!(result2.is_ok(), "Should store token without Remember Me");
    
    let retrieved2 = storage.get_token();
    assert!(retrieved2.is_some(), "Should retrieve token stored without Remember Me");
    assert_eq!(retrieved2.unwrap().token, token.token, "Token should match from sessionStorage");
}

#[wasm_bindgen_test]
fn test_profile_storage_with_remember_me_mode() {
    let storage = StorageService::new();
    
    // Pulisci lo storage prima del test
    let _ = storage.clear_remember_me();
    
    // Crea un profilo mock
    let profile = crate::common::TestFactory::mock_user_profile();
    
    // Test storage profilo con Remember Me
    let result1 = storage.store_user_profile_with_remember_me(&profile, true);
    assert!(result1.is_ok(), "Should store profile with Remember Me");
    
    let retrieved1 = storage.get_user_profile();
    assert!(retrieved1.is_some(), "Should retrieve profile stored with Remember Me");
    
    let retrieved_profile1 = retrieved1.unwrap();
    assert_eq!(retrieved_profile1.email, profile.email, "Email should match");
    assert_eq!(retrieved_profile1.first_name, profile.first_name, "First name should match");
    assert_eq!(retrieved_profile1.last_name, profile.last_name, "Last name should match");
    
    // Clear e test storage profilo senza Remember Me
    let _ = storage.clear_session();
    
    let result2 = storage.store_user_profile_with_remember_me(&profile, false);
    assert!(result2.is_ok(), "Should store profile without Remember Me");
    
    let retrieved2 = storage.get_user_profile();
    assert!(retrieved2.is_some(), "Should retrieve profile stored without Remember Me");
    
    let retrieved_profile2 = retrieved2.unwrap();
    assert_eq!(retrieved_profile2.email, profile.email, "Email should match from sessionStorage");
}

#[wasm_bindgen_test]
fn test_landing_page_storage_workflow() {
    let storage = StorageService::new();
    
    // Pulisci lo storage prima del test
    let _ = storage.clear_remember_me();
    
    // Simula workflow landing page con Remember Me attivo
    let email = "landing@example.com";
    let password = "landingpass123";
    let token = crate::common::TestFactory::mock_token_response();
    let profile = crate::common::TestFactory::mock_user_profile();
    
    // 1. Utente abilita Remember Me e fa login
    let remember_me_result = storage.set_remember_me(email, password, true);
    assert!(remember_me_result.is_ok(), "Should set Remember Me credentials");
    
    // 2. Landing page salva token e profilo con Remember Me
    let token_result = storage.store_token_with_remember_me(&token, true);
    assert!(token_result.is_ok(), "Should store token for Remember Me");
    
    let profile_result = storage.store_user_profile_with_remember_me(&profile, true);
    assert!(profile_result.is_ok(), "Should store profile for Remember Me");
    
    // 3. Verifica che tutto sia salvato correttamente
    assert!(storage.is_remember_me_active(), "Remember Me should be active");
    assert!(storage.get_token().is_some(), "Token should be available");
    assert!(storage.get_user_profile().is_some(), "Profile should be available");
    
    let (retrieved_email, retrieved_password) = storage.get_remember_me_credentials().unwrap();
    assert_eq!(retrieved_email, email, "Remember Me email should match");
    assert_eq!(retrieved_password, password, "Remember Me password should match");
    
    // 4. Simula workflow senza Remember Me
    let _ = storage.clear_session();
    let _ = storage.clear_remember_me();
    
    let token_result2 = storage.store_token_with_remember_me(&token, false);
    assert!(token_result2.is_ok(), "Should store token without Remember Me");
    
    let profile_result2 = storage.store_user_profile_with_remember_me(&profile, false);
    assert!(profile_result2.is_ok(), "Should store profile without Remember Me");
    
    // Verifica che sia salvato ma senza Remember Me
    assert!(!storage.is_remember_me_active(), "Remember Me should not be active");
    assert!(storage.get_token().is_some(), "Token should be available in session");
    assert!(storage.get_user_profile().is_some(), "Profile should be available in session");
    assert!(storage.get_remember_me_credentials().is_none(), "No Remember Me credentials");
}

#[cfg(test)]
mod token_expiry_tests {
    use super::*;
    
    #[test]
    fn test_token_expiry_logic() {
        let now = chrono::Utc::now().timestamp();
        
        // Token valido (scade tra 4 ore)
        let valid_token = TokenResponse {
            token: "valid_token".to_string(),
            iat: now,
            exp: now + 14400, // 4 ore
        };
        
        // Token vicino alla scadenza (scade tra 10 minuti - dovrebbe essere refreshato)
        // Per un token di 1 ora, deve rimanere meno del 25% = 15 min per essere refreshato
        let expiring_token = TokenResponse {
            token: "expiring_token".to_string(),
            iat: now - 3000, // Creato 50 min fa
            exp: now + 600,  // Scade tra 10 minuti
        };
        
        // Token scaduto
        let expired_token = TokenResponse {
            token: "expired_token".to_string(),
            iat: now - 7200,
            exp: now - 3600, // Scaduto 1 ora fa
        };
        
        // Test metodi TokenResponse
        assert!(!valid_token.is_expired(), "Valid token should not be expired");
        assert!(!expiring_token.is_expired(), "Expiring token should not be expired yet");
        assert!(expired_token.is_expired(), "Expired token should be expired");
        
        assert!(valid_token.time_to_expiry() > 7200, "Valid token should have more than 2 hours");
        assert!(expiring_token.time_to_expiry() < 900, "Expiring token should have less than 15 minutes");
        assert_eq!(expired_token.time_to_expiry(), 0, "Expired token should have 0 time left");
        
        // Test logica should_refresh_token
        assert!(!should_refresh_token(&valid_token), "Valid token should not need refresh");
        assert!(should_refresh_token(&expiring_token), "Expiring token should need refresh");
        assert!(should_refresh_token(&expired_token), "Expired token should need refresh");
    }
    
    #[test]
    fn test_refresh_threshold_edge_cases() {
        let now = chrono::Utc::now().timestamp();
        
        // Test con token di durata lunga (>2 ore) 
        // Per token > 2 ore: usa la logica originale (refresh quando rimangono < 2 ore)
        let long_token = TokenResponse {
            token: "long_token".to_string(),
            iat: now,
            exp: now + 10800, // 3 ore totali
        };
        
        // Token lungo con 1 ora e 50 minuti rimanenti (< 2 ore)
        let long_token_should_refresh = TokenResponse {
            token: "long_should_refresh".to_string(),
            iat: now - 4200, // Creato 70 minuti fa
            exp: now + 6600, // Scade in 110 minuti (1h 50m)
        };
        
        assert!(!should_refresh_token(&long_token), "Fresh long token should not need refresh");
        assert!(should_refresh_token(&long_token_should_refresh), "Long token with <2h remaining should need refresh");
        
        // Test con token di durata media (esattamente 2 ore)
        // Per token <= 2 ore: refresh quando rimane meno del 33%
        let medium_token = TokenResponse {
            token: "medium_token".to_string(),
            iat: now,
            exp: now + 7200, // Esattamente 2 ore
        };
        
        // Token medio con 30 minuti rimanenti (< 33% di 2 ore = 40 minuti)
        let medium_token_should_refresh = TokenResponse {
            token: "medium_should_refresh".to_string(),
            iat: now - 5400, // Creato 90 minuti fa
            exp: now + 1800,  // Scade in 30 minuti
        };
        
        assert!(!should_refresh_token(&medium_token), "Fresh medium token should not need refresh");
        assert!(should_refresh_token(&medium_token_should_refresh), "Medium token with <33% time remaining should need refresh");
    }
}

/// Test helper per pulire lo storage tra i test
pub fn cleanup_storage() {
    let storage = StorageService::new();
    let _ = storage.clear_remember_me();
    // Pulisci anche altri dati se necessario
}
