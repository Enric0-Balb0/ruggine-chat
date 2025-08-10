// Integration tests for storage service with authentication
use ruggine_client_ui::services::storage_service::StorageService;
use ruggine_client_ui::dto::{UserProfile, TokenResponse};

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
        let token = TokenResponse {
            token: "test_token_123".to_string(),
            expires_in: Some(3600),
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
            id: "test_id_123".to_string(),
            email: "test@example.com".to_string(),
            full_name: "Test User".to_string(),
            created_at: Some("2025-01-01T00:00:00Z".to_string()),
            updated_at: Some("2025-01-01T00:00:00Z".to_string()),
        };
        
        let result = storage.store_user_profile(&profile);
        assert!(result.is_ok(), "Should be able to store user profile");
        
        // Retrieve profile
        let retrieved = storage.get_user_profile();
        assert!(retrieved.is_some(), "Should be able to retrieve user profile");
        
        let retrieved_profile = retrieved.unwrap();
        assert_eq!(retrieved_profile.id, "test_id_123");
        assert_eq!(retrieved_profile.email, "test@example.com");
        assert_eq!(retrieved_profile.full_name, "Test User");
    }

    #[test]
    fn test_session_clearing() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage_service();
        
        // Store some data
        let token = TokenResponse {
            token: "test_token_123".to_string(),
            expires_in: Some(3600),
        };
        let profile = UserProfile {
            id: "test_id_123".to_string(),
            email: "test@example.com".to_string(),
            full_name: "Test User".to_string(),
            created_at: Some("2025-01-01T00:00:00Z".to_string()),
            updated_at: Some("2025-01-01T00:00:00Z".to_string()),
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
        let token1 = TokenResponse {
            token: "token_1".to_string(),
            expires_in: Some(3600),
        };
        storage.store_token(&token1).unwrap();
        
        let retrieved1 = storage.get_token().unwrap();
        assert_eq!(retrieved1.token, "token_1");
        
        // Overwrite with second token
        let token2 = TokenResponse {
            token: "token_2".to_string(),
            expires_in: Some(7200),
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
            id: "id_1".to_string(),
            email: "test1@example.com".to_string(),
            full_name: "Test User 1".to_string(),
            created_at: Some("2025-01-01T00:00:00Z".to_string()),
            updated_at: Some("2025-01-01T00:00:00Z".to_string()),
        };
        storage.store_user_profile(&profile1).unwrap();
        
        let retrieved1 = storage.get_user_profile().unwrap();
        assert_eq!(retrieved1.id, "id_1");
        assert_eq!(retrieved1.email, "test1@example.com");
        
        // Overwrite with second profile
        let profile2 = UserProfile {
            id: "id_2".to_string(),
            email: "test2@example.com".to_string(),
            full_name: "Test User 2".to_string(),
            created_at: Some("2025-01-01T01:00:00Z".to_string()),
            updated_at: Some("2025-01-01T01:00:00Z".to_string()),
        };
        storage.store_user_profile(&profile2).unwrap();
        
        let retrieved2 = storage.get_user_profile().unwrap();
        assert_eq!(retrieved2.id, "id_2");
        assert_eq!(retrieved2.email, "test2@example.com");
        
        // Should only have the latest profile
        assert_ne!(retrieved2.id, "id_1");
    }

    #[test]
    fn test_storage_persistence_across_service_instances() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage1 = setup_storage_service();
        
        // Store data with first instance
        let token = TokenResponse {
            token: "persistent_token".to_string(),
            expires_in: Some(3600),
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
                let token = TokenResponse {
                    token: format!("token_{}", i),
                    expires_in: Some(3600),
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
}
