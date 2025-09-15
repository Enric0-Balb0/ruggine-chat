// Integration tests for storage service with authentication
use ruggine_client_ui::utils::storage::StorageService;
use ruggine_client_ui::types::{UserProfile, auth::TokenResponse};
use crate::common::TestFactory;

#[cfg(test)]
mod storage_service_auth_tests {
    use super::*;
    use crate::common::init_test_logging;
    use std::sync::Mutex;
    
    // Mutex to serialize tests that use shared storage
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_storage_service() -> StorageService {
        init_test_logging();
        let storage = StorageService::new();
        
        // Clear any previous test data
        #[cfg(not(target_arch = "wasm32"))]
        storage.clear_all_test_data();
        
        storage
    }

    #[test]
    fn test_token_storage_and_retrieval() {
        let _lock = TEST_MUTEX.lock().expect("Failed to acquire test mutex");
        let storage = setup_storage_service();
        
        // Initially no token
        assert!(storage.get_token().is_none());
        
        // Store token using TestFactory
        let token = TestFactory::mock_token_response();
        
        let result = storage.store_token(&token);
        assert!(result.is_ok(), "Should be able to store token");
        
        // Retrieve token
        let retrieved = storage.get_token();
        assert!(retrieved.is_some(), "Should be able to retrieve token");
        
        let retrieved_token = retrieved.unwrap();
        assert_eq!(retrieved_token.token, token.token);
    }

    #[test]
    fn test_user_profile_storage_and_retrieval() {
        let _lock = TEST_MUTEX.lock().expect("Failed to acquire test mutex");
        let storage = setup_storage_service();
        
        // Initially no profile
        assert!(storage.get_user_profile().is_none());
        
        // Store profile using TestFactory
    let profile = TestFactory::mock_user_profile();
        
        let result = storage.store_user_profile(&profile);
        assert!(result.is_ok(), "Should be able to store profile");
        
        // Retrieve profile
        let retrieved = storage.get_user_profile();
        assert!(retrieved.is_some(), "Should be able to retrieve profile");
        
        let retrieved_profile = retrieved.unwrap();
        assert_eq!(retrieved_profile.email, profile.email);
        assert_eq!(retrieved_profile.username, profile.username);
    }

    #[test]
    fn test_session_clearing() {
        let _lock = TEST_MUTEX.lock().expect("Failed to acquire test mutex");
        let storage = setup_storage_service();
        
        // Store some data
        let now = chrono::Utc::now().timestamp();
        let token = TokenResponse {
            token: "test_token_123".to_string(),
            iat: now,
            exp: now + 3600,
        };
        let profile = UserProfile {
            id: 123,
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username: "testuser".to_string(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: ruggine_client_ui::types::user::Gender::Male,
            user_type: ruggine_client_ui::types::user::UserType::EndUser,
            user_status: ruggine_client_ui::types::user::UserStatus::Active,
            current_action: ruggine_client_ui::types::user::CurrentAction::Waiting,
            is_online: true,
            created_at: chrono::DateTime::from_timestamp(1609459200, 0).unwrap(),
            updated_at: chrono::DateTime::from_timestamp(1609459200, 0).unwrap(),
            last_login: None,
        };
        
        storage.store_token(&token).unwrap();
        storage.store_user_profile(&profile).unwrap();
        
        // Verify data exists
        assert!(storage.get_token().is_some());
        assert!(storage.get_user_profile().is_some());
        
        // Clear session
        let result = storage.clear_session();
        assert!(result.is_ok(), "Should be able to clear session");
        
        // Verify data is cleared
        assert!(storage.get_token().is_none());
        assert!(storage.get_user_profile().is_none());
    }

    #[test]
    fn test_multiple_token_overwrites() {
        let _lock = TEST_MUTEX.lock().expect("Failed to acquire test mutex");
        let storage = setup_storage_service();
        
        // Store first token using TestFactory
        let token1 = TestFactory::mock_token_response();
        storage.store_token(&token1).unwrap();
        
        let retrieved1 = storage.get_token().unwrap();
        assert_eq!(retrieved1.token, token1.token);
        
        // Wait a bit to ensure different timestamps for unique tokens
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        // Store second token (should overwrite)
        let token2 = TestFactory::mock_token_response();
        storage.store_token(&token2).unwrap();
        
        let retrieved2 = storage.get_token().unwrap();
        assert_eq!(retrieved2.token, token2.token);
        assert_ne!(retrieved2.token, token1.token);
    }

    #[test]
    fn test_multiple_profile_overwrites() {
        let _lock = TEST_MUTEX.lock().expect("Failed to acquire test mutex");
        let storage = setup_storage_service();
        
        // Store first profile using TestFactory
    let profile1 = TestFactory::mock_user_profile();
        storage.store_user_profile(&profile1).unwrap();
        
        let retrieved1 = storage.get_user_profile().unwrap();
        assert_eq!(retrieved1.email, profile1.email);
        
        // Store second profile (should overwrite)
    let profile2 = TestFactory::mock_user_profile();
        storage.store_user_profile(&profile2).unwrap();
        
        let retrieved2 = storage.get_user_profile().unwrap();
        assert_eq!(retrieved2.email, profile2.email);
        assert_ne!(retrieved2.email, profile1.email);
    }

    #[test]
    fn test_storage_persistence_across_service_instances() {
        let _lock = TEST_MUTEX.lock().expect("Failed to acquire test mutex");
        let storage1 = setup_storage_service();
        
        // Store data with first instance using TestFactory
        let token = TestFactory::mock_token_response();
        storage1.store_token(&token).unwrap();
        
        // Create new storage instance
        let storage2 = StorageService::new();
        
        // Should be able to retrieve data from new instance
        let retrieved = storage2.get_token();
        assert!(retrieved.is_some(), "Data should persist across service instances");
        
        let retrieved_token = retrieved.unwrap();
        assert_eq!(retrieved_token.token, token.token);
    }

    #[test]
    fn test_concurrent_storage_operations() {
        let _lock = TEST_MUTEX.lock().unwrap();
        use std::sync::Arc;
        use std::thread;
        
        let storage = Arc::new(setup_storage_service());
        let mut handles = vec![];
        
        // Spawn multiple threads doing storage operations
        for i in 0..5 {
            let storage_clone = Arc::clone(&storage);
            let handle = thread::spawn(move || {
                let now = chrono::Utc::now().timestamp();
                let token = TokenResponse {
                    token: format!("token_{}", i),
                    iat: now,
                    exp: now + 3600,
                };
                storage_clone.store_token(&token).unwrap();
                
                // Verify we can retrieve some token (may not be the one we just stored due to concurrency)
                let retrieved = storage_clone.get_token();
                assert!(retrieved.is_some());
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final state is consistent
        let final_token = storage.get_token();
        assert!(final_token.is_some(), "Should have a token after concurrent operations");
    }

    #[test]
    fn test_error_handling_for_corrupted_data() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // This test would verify handling of corrupted storage data
        // For now, we just test that operations don't panic
        
        // Clear any existing data
        storage.clear_session().unwrap();
        
        // Verify clean state
        assert!(storage.get_token().is_none());
        assert!(storage.get_user_profile().is_none());
    }

    #[test]
    fn test_logout_storage_cleanup_integration() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // Setup complete session data (simulating logged-in user) using TestFactory
        let token = TestFactory::mock_token_response();
    let profile = TestFactory::mock_user_profile();
        
        // Store session data
        storage.store_token(&token).unwrap();
        storage.store_user_profile(&profile).unwrap();
        
        // Verify data exists before logout
        assert!(storage.get_token().is_some());
        assert!(storage.get_user_profile().is_some());
        
        // Perform logout cleanup
        let result = storage.clear_session();
        assert!(result.is_ok(), "Logout cleanup should succeed");
        
        // Verify all session data is cleared
        assert!(storage.get_token().is_none(), "Token should be cleared after logout");
        assert!(storage.get_user_profile().is_none(), "Profile should be cleared after logout");
    }

    #[test]
    fn test_logout_cleanup_robustness() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // Test multiple logout attempts (should be safe)
        let clear_result1 = storage.clear_session();
        assert!(clear_result1.is_ok(), "First clear should succeed");
        
        let clear_result2 = storage.clear_session();
        assert!(clear_result2.is_ok(), "Second clear should also succeed");
        
        // Verify state remains clean
        assert!(storage.get_token().is_none());
        assert!(storage.get_user_profile().is_none());
        
        // Store some data, then clear multiple times using TestFactory
        let token = TestFactory::mock_token_response();
        storage.store_token(&token).unwrap();
        
        // Multiple clears should be safe
        storage.clear_session().unwrap();
        storage.clear_session().unwrap();
        storage.clear_session().unwrap();
        
        // Should remain clear
        assert!(storage.get_token().is_none());
    }

    #[test]
    fn test_logout_preserves_non_session_data() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // This test verifies that logout only clears session data
        // and preserves other app data (if any)
        
        // Set up session data using TestFactory
        let token = TestFactory::mock_token_response();
    let profile = TestFactory::mock_user_profile();
        
        storage.store_token(&token).unwrap();
        storage.store_user_profile(&profile).unwrap();
        
        // Verify data exists
        assert!(storage.get_token().is_some());
        assert!(storage.get_user_profile().is_some());
        
        // Perform logout
        storage.clear_session().unwrap();
        
        // Session data should be cleared
        assert!(storage.get_token().is_none(), "Session token should be cleared");
        assert!(storage.get_user_profile().is_none(), "User profile should be cleared");
        
        // Note: In the future, if we add app settings or other non-session data,
        // we would verify those remain intact after logout
    }
}
