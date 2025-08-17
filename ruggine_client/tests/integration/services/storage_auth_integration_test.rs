// Integration tests for storage service with authentication
use ruggine_client_ui::utils::storage::StorageService;
use ruggine_client_ui::types::{UserProfile, auth::TokenResponse};

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
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // Initially no token
        assert!(storage.get_token().is_none());
        
        // Store token
        let now = chrono::Utc::now().timestamp();
        let token = TokenResponse {
            token: "test_token_123".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour from now
        };
        
        let result = storage.store_token(&token);
        assert!(result.is_ok(), "Should be able to store token");
        
        // Retrieve token
        let retrieved = storage.get_token();
        assert!(retrieved.is_some(), "Should be able to retrieve token");
        
        let retrieved_token = retrieved.unwrap();
        assert_eq!(retrieved_token.token, "test_token_123");
    }

    #[test]
    fn test_user_profile_storage_and_retrieval() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // Initially no profile
        assert!(storage.get_user_profile().is_none());
        
        // Store profile
        let profile = UserProfile {
            id: 123,  // Changed to i32
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),  // Changed to first_name
            last_name: "User".to_string(),   // Changed to last_name
            username: "testuser".to_string(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: ruggine_client_ui::types::user::Gender::Male,
            user_type: ruggine_client_ui::types::user::UserType::EndUser,
            user_status: ruggine_client_ui::types::user::UserStatus::Active,
            current_action: ruggine_client_ui::types::user::CurrentAction::Waiting,
            is_online: true,
            created_at: chrono::DateTime::from_timestamp(1609459200, 0).unwrap(), // 2021-01-01
            updated_at: chrono::DateTime::from_timestamp(1609459200, 0).unwrap(),
            last_login: None,
        };
        
        let result = storage.store_user_profile(&profile);
        assert!(result.is_ok(), "Should be able to store user profile");
        
        // Retrieve profile
        let retrieved = storage.get_user_profile();
        assert!(retrieved.is_some(), "Should be able to retrieve user profile");
        
        let retrieved_profile = retrieved.unwrap();
        assert_eq!(retrieved_profile.id, 123);
        assert_eq!(retrieved_profile.email, "test@example.com");
        assert_eq!(retrieved_profile.full_name(), "Test User");  // Use method call
    }

    #[test]
    fn test_session_clearing() {
        let _lock = TEST_MUTEX.lock().unwrap();
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
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // Store first token
        let now = chrono::Utc::now().timestamp();
        let token1 = TokenResponse {
            token: "token_1".to_string(),
            iat: now,
            exp: now + 3600,
        };
        storage.store_token(&token1).unwrap();
        
        let retrieved1 = storage.get_token().unwrap();
        assert_eq!(retrieved1.token, "token_1");
        
        // Overwrite with second token
        let token2 = TokenResponse {
            token: "token_2".to_string(),
            iat: now,
            exp: now + 7200,
        };
        storage.store_token(&token2).unwrap();
        
        let retrieved2 = storage.get_token().unwrap();
        assert_eq!(retrieved2.token, "token_2");
        
        // Should only have the latest token
        assert_ne!(retrieved2.token, "token_1");
    }

    #[test]
    fn test_multiple_profile_overwrites() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // Store first profile
        let profile1 = UserProfile {
            id: 1,  // Changed to i32
            email: "test1@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User 1".to_string(),
            username: "testuser1".to_string(),
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
        storage.store_user_profile(&profile1).unwrap();
        
        let retrieved1 = storage.get_user_profile().unwrap();
        assert_eq!(retrieved1.id, 1);  // Changed to i32
        assert_eq!(retrieved1.email, "test1@example.com");
        
        // Overwrite with second profile
        let profile2 = UserProfile {
            id: 2,  // Changed to i32
            email: "test2@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User 2".to_string(),
            username: "testuser2".to_string(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "456 Test Ave".to_string(),
            gender: ruggine_client_ui::types::user::Gender::Female,
            user_type: ruggine_client_ui::types::user::UserType::EndUser,
            user_status: ruggine_client_ui::types::user::UserStatus::Active,
            current_action: ruggine_client_ui::types::user::CurrentAction::Waiting,
            is_online: false,
            created_at: chrono::DateTime::from_timestamp(1609462800, 0).unwrap(), // 1 hour later
            updated_at: chrono::DateTime::from_timestamp(1609462800, 0).unwrap(),
            last_login: None,
        };
        storage.store_user_profile(&profile2).unwrap();
        
        let retrieved2 = storage.get_user_profile().unwrap();
        assert_eq!(retrieved2.id, 2);  // Changed to i32
        assert_eq!(retrieved2.email, "test2@example.com");
        
        // Should only have the latest profile
        assert_ne!(retrieved2.id, 1);  // Changed to i32
    }

    #[test]
    fn test_storage_persistence_across_service_instances() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage1 = setup_storage_service();
        
        // Store data with first instance
        let now = chrono::Utc::now().timestamp();
        let token = TokenResponse {
            token: "persistent_token".to_string(),
            iat: now,
            exp: now + 3600,
        };
        storage1.store_token(&token).unwrap();
        
        // Create new storage instance
        let storage2 = StorageService::new();
        
        // Should be able to retrieve data from new instance
        let retrieved = storage2.get_token();
        assert!(retrieved.is_some(), "Data should persist across service instances");
        
        let retrieved_token = retrieved.unwrap();
        assert_eq!(retrieved_token.token, "persistent_token");
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
        
        // Setup complete session data (simulating logged-in user)
        let now = chrono::Utc::now().timestamp();
        let token = TokenResponse {
            token: "logout_test_token".to_string(),
            iat: now,
            exp: now + 3600,
        };
        
        let profile = UserProfile {
            id: 999,
            email: "logout@test.com".to_string(),
            first_name: "Logout".to_string(),
            last_name: "Test".to_string(),
            username: "logouttest".to_string(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "Logout Test St".to_string(),
            gender: ruggine_client_ui::types::user::Gender::Other,
            user_type: ruggine_client_ui::types::user::UserType::EndUser,
            user_status: ruggine_client_ui::types::user::UserStatus::Active,
            current_action: ruggine_client_ui::types::user::CurrentAction::Waiting,
            is_online: true,
            created_at: chrono::DateTime::from_timestamp(1609459200, 0).unwrap(),
            updated_at: chrono::DateTime::from_timestamp(1609459200, 0).unwrap(),
            last_login: Some(chrono::DateTime::from_timestamp(1609459200, 0).unwrap()),
        };
        
        // Store session data
        storage.store_token(&token).expect("Should store token for logout test");
        storage.store_user_profile(&profile).expect("Should store profile for logout test");
        
        // Verify data exists before logout
        assert!(storage.get_token().is_some(), "Token should exist before logout");
        assert!(storage.get_user_profile().is_some(), "Profile should exist before logout");
        
        let stored_token = storage.get_token().unwrap();
        let stored_profile = storage.get_user_profile().unwrap();
        assert_eq!(stored_token.token, "logout_test_token");
        assert_eq!(stored_profile.email, "logout@test.com");
        
        // Perform logout cleanup (clear session)
        let clear_result = storage.clear_session();
        assert!(clear_result.is_ok(), "Session clear should succeed: {:?}", clear_result);
        
        // Verify complete cleanup after logout
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
        
        // Store some data, then clear multiple times
        let now = chrono::Utc::now().timestamp();
        let token = TokenResponse {
            token: "multi_clear_test".to_string(),
            iat: now,
            exp: now + 3600,
        };
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
        
        // This test verifies that logout only clears session-specific data
        // and preserves other application data (if any exists in the future)
        
        // Store session data
        let now = chrono::Utc::now().timestamp();
        let token = TokenResponse {
            token: "session_token".to_string(),
            iat: now,
            exp: now + 3600,
        };
        
        let profile = UserProfile {
            id: 100,
            email: "preserve@test.com".to_string(),
            first_name: "Preserve".to_string(),
            last_name: "Test".to_string(),
            username: "preservetest".to_string(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "Preserve Test St".to_string(),
            gender: ruggine_client_ui::types::user::Gender::Female,
            user_type: ruggine_client_ui::types::user::UserType::EndUser,
            user_status: ruggine_client_ui::types::user::UserStatus::Active,
            current_action: ruggine_client_ui::types::user::CurrentAction::Waiting,
            is_online: false,
            created_at: chrono::DateTime::from_timestamp(1609459200, 0).unwrap(),
            updated_at: chrono::DateTime::from_timestamp(1609459200, 0).unwrap(),
            last_login: None,
        };
        
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
