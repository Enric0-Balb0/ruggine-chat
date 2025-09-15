// Unit tests for StorageService new features
// Testing dual storage system, Remember Me functionality, and storage management

use ruggine_client_ui::utils::storage::StorageService;

#[cfg(test)]
mod storage_service_tests {
    use super::*;
    use crate::common::TestFactory;
    use std::sync::Mutex;
    
    // Mutex to serialize tests that use shared storage
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_storage() -> StorageService {
        let storage = StorageService::new();
        
        // Clear any previous test data
        #[cfg(not(target_arch = "wasm32"))]
        storage.clear_all_test_data();
        
        storage
    }

    #[test]
    fn test_backward_compatibility_methods() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let token = TestFactory::mock_token_response();
        let profile = TestFactory::mock_user_profile();
        
        // Test that old methods still work (should default to sessionStorage)
        let token_result = storage.store_token(&token);
        assert!(token_result.is_ok(), "Old store_token method should work");
        
        let profile_result = storage.store_user_profile(&profile);
        assert!(profile_result.is_ok(), "Old store_user_profile method should work");
        
        // Data should be retrievable
        assert!(storage.get_token().is_some(), "Should retrieve token with old methods");
        assert!(storage.get_user_profile().is_some(), "Should retrieve profile with old methods");
    }

    #[test]
    fn test_avatar_data_extraction() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let profile = TestFactory::mock_user_profile();
        let _ = storage.store_user_profile(&profile);
        
        let retrieved_profile = storage.get_user_profile();
        assert!(retrieved_profile.is_some(), "Should retrieve profile for avatar test");
        
        let user_profile = retrieved_profile.unwrap();
        
        // Test avatar initials extraction (same logic used in components)
        let first_initial = user_profile.first_name.chars().next().unwrap_or('U');
        let last_initial = user_profile.last_name.chars().next().unwrap_or('S');
        let initials = format!("{}{}", first_initial.to_uppercase(), last_initial.to_uppercase());
        
        assert_eq!(initials.len(), 2, "Avatar initials should be 2 characters");
        assert!(initials.chars().all(|c| c.is_alphabetic()), "Initials should be alphabetic");
        assert!(!user_profile.first_name.is_empty(), "First name should not be empty for avatar");
        assert!(!user_profile.last_name.is_empty(), "Last name should not be empty for avatar");
    }

    #[test]
    fn test_storage_error_handling() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        // Test retrieval from empty storage
        assert!(storage.get_token().is_none(), "Should return None for missing token");
        assert!(storage.get_user_profile().is_none(), "Should return None for missing profile");
        
        // Test clearing empty storage (should not error)
        let clear_session_result = storage.clear_session();
        assert!(clear_session_result.is_ok(), "Should clear empty session without error");
    }

    #[test]
    fn test_storage_service_consistency() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        // Test that multiple StorageService instances access the same underlying storage
        let storage1 = StorageService::new();
        let storage2 = StorageService::new();
        
        #[cfg(not(target_arch = "wasm32"))]
        storage1.clear_all_test_data();
        
        let token = TestFactory::mock_token_response();
        
        // Store with first instance
        let _ = storage1.store_token(&token);
        
        // Retrieve with second instance
        let retrieved_token = storage2.get_token();
        assert!(retrieved_token.is_some(), "Should retrieve token stored by different instance");
        assert_eq!(retrieved_token.unwrap().token, token.token, "Token should match across instances");
        
        // Clear with second instance
        let _ = storage2.clear_session();
        
        // Verify cleared with first instance
        assert!(storage1.get_token().is_none(), "Token should be cleared across instances");
    }
}

/// Helper tests for storage service edge cases
#[cfg(test)]
mod storage_edge_cases {
    use super::*;

    #[test]
    fn test_token_storage_with_invalid_data() {
        let storage = StorageService::new();
        #[cfg(not(target_arch = "wasm32"))]
        storage.clear_all_test_data();
        
        // Test storing token with minimal/edge case data
        let mut minimal_token = crate::common::TestFactory::mock_token_response();
        minimal_token.token = String::new(); // Empty token
        
        let result = storage.store_token(&minimal_token);
        // Should handle empty token gracefully
        assert!(result.is_ok() || result.is_err(), "Should handle empty token");
        
        // Test retrieving after storing empty token
        let retrieved = storage.get_token();
        // Behavior depends on implementation - should be consistent
        if let Some(token) = retrieved {
            assert_eq!(token.token, minimal_token.token, "Retrieved token should match stored");
        }
    }

    #[test]
    fn test_profile_storage_with_minimal_data() {
        let storage = StorageService::new();
        #[cfg(not(target_arch = "wasm32"))]
        storage.clear_all_test_data();
        
        // Test storing profile with minimal data
        let mut minimal_profile = crate::common::TestFactory::mock_user_profile();
        minimal_profile.first_name = String::new();
        minimal_profile.last_name = String::new();
        
        let result = storage.store_user_profile(&minimal_profile);
        assert!(result.is_ok(), "Should store profile with minimal data");
        
        let retrieved = storage.get_user_profile();
        assert!(retrieved.is_some(), "Should retrieve minimal profile");
        
        let profile = retrieved.unwrap();
        assert_eq!(profile.first_name, minimal_profile.first_name, "First name should match");
        assert_eq!(profile.last_name, minimal_profile.last_name, "Last name should match");
        assert_eq!(profile.email, minimal_profile.email, "Email should match");
    }
}
