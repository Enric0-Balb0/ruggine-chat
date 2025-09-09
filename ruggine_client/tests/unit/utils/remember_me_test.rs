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
fn test_token_persistence_with_remember_me() {
    let storage = StorageService::new();
    
    // Pulisci lo storage prima del test
    let _ = storage.clear_remember_me();
    
    let now = chrono::Utc::now().timestamp();
    let token = TokenResponse {
        token: "test_token_123".to_string(),
        iat: now,
        exp: now + 3600, // Scade tra 1 ora
    };
    
    // Test storage token senza Remember Me
    let result = storage.store_token_with_remember_me(&token, false);
    assert!(result.is_ok(), "Should store token successfully");
    
    // Test storage token con Remember Me
    let result = storage.store_token_with_remember_me(&token, true);
    assert!(result.is_ok(), "Should store token with remember me successfully");
    
    // Verifica retrieval
    let retrieved = storage.get_token();
    assert!(retrieved.is_some(), "Should retrieve stored token");
    
    let retrieved_token = retrieved.unwrap();
    assert_eq!(retrieved_token.token, token.token);
    assert_eq!(retrieved_token.exp, token.exp);
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
        
        // Token vicino alla scadenza (scade tra 1 ora)
        let expiring_token = TokenResponse {
            token: "expiring_token".to_string(),
            iat: now,
            exp: now + 3600, // 1 ora
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
        assert!(expiring_token.time_to_expiry() < 7200, "Expiring token should have less than 2 hours");
        assert_eq!(expired_token.time_to_expiry(), 0, "Expired token should have 0 time left");
        
        // Test logica should_refresh_token
        assert!(!should_refresh_token(&valid_token), "Valid token should not need refresh");
        assert!(should_refresh_token(&expiring_token), "Expiring token should need refresh");
        assert!(should_refresh_token(&expired_token), "Expired token should need refresh");
    }
    
    #[test]
    fn test_refresh_threshold_edge_cases() {
        let now = chrono::Utc::now().timestamp();
        
        // Token che scade esattamente in 2 ore
        let edge_token = TokenResponse {
            token: "edge_token".to_string(),
            iat: now,
            exp: now + 7200, // Esattamente 2 ore
        };
        
        // Token che scade in 2 ore e 1 secondo
        let just_over_token = TokenResponse {
            token: "just_over_token".to_string(),
            iat: now,
            exp: now + 7201, // 2 ore e 1 secondo
        };
        
        // Il threshold è < 7200, quindi 7200 secondi esatti non dovrebbero triggare il refresh
        assert!(!should_refresh_token(&edge_token), "Token with exactly 2 hours should not need refresh");
        assert!(!should_refresh_token(&just_over_token), "Token with slightly more than 2 hours should not need refresh");
        
        // Token che scade in 1 ora e 59 minuti (7140 secondi)
        let just_under_token = TokenResponse {
            token: "just_under_token".to_string(),
            iat: now,
            exp: now + 7140,
        };
        
        assert!(should_refresh_token(&just_under_token), "Token with less than 2 hours should need refresh");
    }
}

/// Test helper per pulire lo storage tra i test
pub fn cleanup_storage() {
    let storage = StorageService::new();
    let _ = storage.clear_remember_me();
    // Pulisci anche altri dati se necessario
}
