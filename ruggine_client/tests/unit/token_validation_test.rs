use ruggine_client_ui::api::services::auth::AuthService;
use ruggine_client_ui::utils::storage::StorageService;
use ruggine_client_ui::api::client::ApiClient;
use ruggine_client_ui::dto::TokenResponse;
use ruggine_client_ui::config::constants::AppConstants;
use std::sync::{LazyLock, Mutex};

// Mutex per sincronizzare i test che utilizzano lo storage
static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn setup_auth_service_with_token(expires_in: Option<u64>) -> AuthService {
    let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
    let storage_service = StorageService::new();
    storage_service.clear_all_test_data();
    
    // Store a test token if provided
    if let Some(exp) = expires_in {
        let token = TokenResponse {
            token: "test_token_123".to_string(),
            expires_in: Some(exp),
        };
        storage_service.store_token(&token).expect("Failed to store test token");
    }
    
    AuthService::new(http_client, storage_service)
}

#[cfg(test)]
mod token_validation_tests {
    use super::*;

    #[test]
    fn test_is_authenticated_no_token() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let auth_service = setup_auth_service_with_token(None);
        
        assert!(!auth_service.is_authenticated(), "Should not be authenticated without token");
    }

    #[test]
    fn test_is_authenticated_valid_token() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let future_timestamp = (chrono::Utc::now().timestamp() + 3600) as u64; // 1 hour from now
        let auth_service = setup_auth_service_with_token(Some(future_timestamp));
        
        assert!(auth_service.is_authenticated(), "Should be authenticated with valid token");
    }

    #[test]
    fn test_is_authenticated_expired_token() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let past_timestamp = (chrono::Utc::now().timestamp() - 3600) as u64; // 1 hour ago
        let auth_service = setup_auth_service_with_token(Some(past_timestamp));
        
        assert!(!auth_service.is_authenticated(), "Should not be authenticated with expired token");
    }

    #[test]
    fn test_expired_token_cleanup() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let past_timestamp = (chrono::Utc::now().timestamp() - 3600) as u64; // 1 hour ago
        let auth_service = setup_auth_service_with_token(Some(past_timestamp));
        
        // First call should detect expired token and clean up
        assert!(!auth_service.is_authenticated(), "Should not be authenticated with expired token");
        
        // Token should be removed from storage after cleanup
        let stored_token = auth_service.get_storage_service().get_token();
        assert!(stored_token.is_none(), "Expired token should be removed from storage");
    }

    #[test]
    fn test_needs_token_refresh_no_token() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let auth_service = setup_auth_service_with_token(None);
        
        assert!(!auth_service.needs_token_refresh(), "Should not need refresh without token");
    }

    #[test]
    fn test_needs_token_refresh_valid_token() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let future_timestamp = (chrono::Utc::now().timestamp() + 3600) as u64; // 1 hour from now
        let auth_service = setup_auth_service_with_token(Some(future_timestamp));
        
        assert!(!auth_service.needs_token_refresh(), "Should not need refresh for token with 1 hour remaining");
    }

    #[test] 
    fn test_needs_token_refresh_soon_to_expire() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let soon_timestamp = (chrono::Utc::now().timestamp() + 200) as u64; // 200 seconds from now (< 5 minutes)
        let auth_service = setup_auth_service_with_token(Some(soon_timestamp));
        
        assert!(auth_service.needs_token_refresh(), "Should need refresh for token expiring in < 5 minutes");
    }

    #[test]
    fn test_needs_token_refresh_expired_token() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let past_timestamp = (chrono::Utc::now().timestamp() - 100) as u64; // expired
        let auth_service = setup_auth_service_with_token(Some(past_timestamp));
        
        assert!(!auth_service.needs_token_refresh(), "Should not need refresh for already expired token");
    }

    #[test]
    fn test_token_without_expiry_backwards_compatibility() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        storage_service.clear_all_test_data();
        
        // Store token without expiry (backwards compatibility)
        let token = TokenResponse {
            token: "test_token_no_expiry".to_string(),
            expires_in: None,
        };
        storage_service.store_token(&token).expect("Failed to store test token");
        
        let auth_service = AuthService::new(http_client, storage_service);
        
        assert!(auth_service.is_authenticated(), "Should be authenticated with token without expiry (backwards compatibility)");
        assert!(!auth_service.needs_token_refresh(), "Should not need refresh for token without expiry");
    }
}
