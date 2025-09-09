use ruggine_client_ui::utils::StorageService;
use ruggine_client_ui::types::TokenResponse;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_remember_me_login_flow() {
    let storage = StorageService::new();
    
    // Pulisci lo storage prima del test
    let _ = storage.clear_remember_me();
    
    // Simula un login con Remember Me
    let email = "test@example.com";
    let password = "password123";
    
    // Step 1: Salva credenziali Remember Me
    let result = storage.set_remember_me(email, password, true);
    assert!(result.is_ok(), "Should save remember me credentials");
    
    // Step 2: Simula che l'utente abbia fatto login e ottenuto un token
    let now = chrono::Utc::now().timestamp();
    let token = TokenResponse {
        token: "test_jwt_token".to_string(),
        iat: now,
        exp: now + 86400, // 24 ore
    };
    
    let result = storage.store_token_with_remember_me(&token, true);
    assert!(result.is_ok(), "Should store token with remember me");
    
    // Step 3: Simula riavvio dell'app - verifica che i dati persistano
    assert!(storage.is_remember_me_active(), "Remember me should be active after restart");
    
    let retrieved_credentials = storage.get_remember_me_credentials();
    assert!(retrieved_credentials.is_some(), "Should retrieve credentials after restart");
    
    let (retrieved_email, retrieved_password) = retrieved_credentials.unwrap();
    assert_eq!(retrieved_email, email);
    assert_eq!(retrieved_password, password);
    
    let retrieved_token = storage.get_token();
    assert!(retrieved_token.is_some(), "Should retrieve token after restart");
    assert_eq!(retrieved_token.unwrap().token, token.token);
}

#[wasm_bindgen_test]
async fn test_remember_me_token_refresh_detection() {
    let storage = StorageService::new();
    let _ = storage.clear_remember_me();
    
    let now = chrono::Utc::now().timestamp();
    
    // Test con token che scade presto (1 ora)
    let expiring_token = TokenResponse {
        token: "expiring_token".to_string(),
        iat: now,
        exp: now + 3600, // 1 ora
    };
    
    let _ = storage.store_token_with_remember_me(&expiring_token, true);
    
    // Verifica che il token sia considerato "vicino alla scadenza"
    let retrieved = storage.get_token().unwrap();
    assert!(retrieved.time_to_expiry() < 7200, "Token should be near expiry");
    assert!(!retrieved.is_expired(), "Token should not be expired yet");
    
    // Test con token scaduto
    let expired_token = TokenResponse {
        token: "expired_token".to_string(),
        iat: now - 7200,
        exp: now - 3600,
    };
    
    let _ = storage.store_token_with_remember_me(&expired_token, true);
    let retrieved = storage.get_token().unwrap();
    assert!(retrieved.is_expired(), "Token should be expired");
}

#[wasm_bindgen_test]
async fn test_remember_me_security_isolation() {
    let storage = StorageService::new();
    let _ = storage.clear_remember_me();
    
    // Test che Remember Me funzioni indipendentemente da session storage
    let email = "test@example.com";
    let password = "password123";
    
    // Salva in Remember Me
    let _ = storage.set_remember_me(email, password, true);
    
    // Salva un token normale (senza remember me)
    let now = chrono::Utc::now().timestamp();
    let session_token = TokenResponse {
        token: "session_token".to_string(),
        iat: now,
        exp: now + 3600,
    };
    
    let _ = storage.store_token(&session_token);
    
    // Verifica che entrambi coesistano
    assert!(storage.is_remember_me_active(), "Remember me should be active");
    assert!(storage.get_remember_me_credentials().is_some(), "Should have remember me credentials");
    assert!(storage.get_token().is_some(), "Should have session token");
    
    // Clear solo Remember Me
    let _ = storage.clear_remember_me();
    
    // Verifica che Remember Me sia pulito ma il token di sessione rimanga
    assert!(!storage.is_remember_me_active(), "Remember me should not be active");
    assert!(storage.get_remember_me_credentials().is_none(), "Should not have remember me credentials");
    assert!(storage.get_token().is_some(), "Should still have session token");
}

#[wasm_bindgen_test]
async fn test_remember_me_error_scenarios() {
    let storage = StorageService::new();
    let _ = storage.clear_remember_me();
    
    // Test con dati corrotti
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;
        if let Some(window) = window() {
            if let Some(local_storage) = window.local_storage().ok().flatten() {
                // Inserisci dati corrotti
                let _ = local_storage.set_item("remember_me_credentials", "invalid_json_data");
                
                // Verifica che il sistema gestisca gracefully i dati corrotti
                let credentials = storage.get_remember_me_credentials();
                assert!(credentials.is_none(), "Should return None for corrupted data");
                assert!(!storage.is_remember_me_active(), "Should not be active with corrupted data");
            }
        }
    }
    
    // Test con stringhe vuote
    let result = storage.set_remember_me("", "", false);
    // A seconda dell'implementazione, potrebbe essere ok o errore
    // L'importante è che non crashhi
    
    let result = storage.set_remember_me("valid@email.com", "", false);
    // Stesso discorso per password vuota
}

/// Test helper per configurare un ambiente di test pulito
pub fn setup_clean_test_environment() -> StorageService {
    let storage = StorageService::new();
    let _ = storage.clear_remember_me();
    // Pulisci altri dati se necessario
    storage
}

/// Test helper per creare un token valido per i test
pub fn create_test_token(hours_until_expiry: i64) -> TokenResponse {
    let now = chrono::Utc::now().timestamp();
    TokenResponse {
        token: format!("test_token_{}", now),
        iat: now,
        exp: now + (hours_until_expiry * 3600),
    }
}

/// Test helper per creare un token scaduto
pub fn create_expired_token() -> TokenResponse {
    let now = chrono::Utc::now().timestamp();
    TokenResponse {
        token: format!("expired_token_{}", now),
        iat: now - 7200,
        exp: now - 3600,
    }
}
