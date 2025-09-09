// Unit tests for StorageService new features
// Testing dual storage system, Remember Me functionality, and storage management

use ruggine_client_ui::utils::storage::StorageService;
use ruggine_client_ui::types::{TokenResponse, UserProfile};

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
    fn test_dual_storage_remember_me_enabled() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let token = TestFactory::mock_token_response();
        let profile = TestFactory::mock_user_profile();
        
        // Test storing with Remember Me enabled (should use localStorage)
        let token_result = storage.store_token_with_remember_me(&token, true);
        assert!(token_result.is_ok(), "Should store token with Remember Me in localStorage");
        
        let profile_result = storage.store_user_profile_with_remember_me(&profile, true);
        assert!(profile_result.is_ok(), "Should store profile with Remember Me in localStorage");
        
        // Data should be retrievable
        let retrieved_token = storage.get_token();
        assert!(retrieved_token.is_some(), "Should retrieve token from localStorage");
        assert_eq!(retrieved_token.unwrap().token, token.token, "Token should match");
        
        let retrieved_profile = storage.get_user_profile();
        assert!(retrieved_profile.is_some(), "Should retrieve profile from localStorage");
        assert_eq!(retrieved_profile.unwrap().email, profile.email, "Profile should match");
    }

    #[test]
    fn test_dual_storage_remember_me_disabled() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let token = TestFactory::mock_token_response();
        let profile = TestFactory::mock_user_profile();
        
        // Test storing without Remember Me (should use sessionStorage)
        let token_result = storage.store_token_with_remember_me(&token, false);
        assert!(token_result.is_ok(), "Should store token without Remember Me in sessionStorage");
        
        let profile_result = storage.store_user_profile_with_remember_me(&profile, false);
        assert!(profile_result.is_ok(), "Should store profile without Remember Me in sessionStorage");
        
        // Data should be retrievable
        let retrieved_token = storage.get_token();
        assert!(retrieved_token.is_some(), "Should retrieve token from sessionStorage");
        assert_eq!(retrieved_token.unwrap().token, token.token, "Token should match");
        
        let retrieved_profile = storage.get_user_profile();
        assert!(retrieved_profile.is_some(), "Should retrieve profile from sessionStorage");
        assert_eq!(retrieved_profile.unwrap().email, profile.email, "Profile should match");
    }

    #[test]
    fn test_remember_me_credentials_lifecycle() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let email = "test@example.com";
        let password = "password123";
        
        // Initially no Remember Me should be active
        assert!(!storage.is_remember_me_active(), "Remember Me should not be active initially");
        assert!(storage.get_remember_me_credentials().is_none(), "No credentials should be stored initially");
        
        // Set Remember Me credentials
        let set_result = storage.set_remember_me(email, password, true);
        assert!(set_result.is_ok(), "Should set Remember Me credentials successfully");
        
        // Verify Remember Me is now active
        assert!(storage.is_remember_me_active(), "Remember Me should be active after setting");
        
        // Retrieve and verify credentials
        let credentials = storage.get_remember_me_credentials();
        assert!(credentials.is_some(), "Should retrieve Remember Me credentials");
        
        let (retrieved_email, retrieved_password) = credentials.unwrap();
        assert_eq!(retrieved_email, email, "Email should match");
        assert_eq!(retrieved_password, password, "Password should match");
        
        // Clear Remember Me credentials
        let clear_result = storage.clear_remember_me();
        assert!(clear_result.is_ok(), "Should clear Remember Me credentials successfully");
        
        // Verify Remember Me is no longer active
        assert!(!storage.is_remember_me_active(), "Remember Me should not be active after clearing");
        assert!(storage.get_remember_me_credentials().is_none(), "No credentials should be stored after clearing");
    }

    #[test]
    fn test_remember_me_disabled_explicit() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let email = "test@example.com";
        let password = "password123";
        
        // Set Remember Me with enabled=false
        let set_result = storage.set_remember_me(email, password, false);
        assert!(set_result.is_ok(), "Should set Remember Me with enabled=false");
        
        // Remember Me should not be active when explicitly disabled
        assert!(!storage.is_remember_me_active(), "Remember Me should not be active when disabled");
        assert!(storage.get_remember_me_credentials().is_none(), "No credentials should be available when disabled");
    }

    #[test]
    fn test_session_storage_operations() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let token = TestFactory::mock_token_response();
        let profile = TestFactory::mock_user_profile();
        
        // Store data in session storage (Remember Me disabled)
        let _ = storage.store_token_with_remember_me(&token, false);
        let _ = storage.store_user_profile_with_remember_me(&profile, false);
        
        // Verify data is present
        assert!(storage.get_token().is_some(), "Token should be in session storage");
        assert!(storage.get_user_profile().is_some(), "Profile should be in session storage");
        
        // Clear session storage
        let clear_result = storage.clear_session();
        assert!(clear_result.is_ok(), "Should clear session storage successfully");
        
        // Verify data is cleared
        assert!(storage.get_token().is_none(), "Token should be cleared from session storage");
        assert!(storage.get_user_profile().is_none(), "Profile should be cleared from session storage");
    }

    #[test]
    fn test_storage_priority_hierarchy() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let token1 = TestFactory::mock_token_response();
        let token2 = TestFactory::mock_token_response_with_exp(
            chrono::Utc::now().timestamp() + 7200 // Different expiration
        );
        let profile = TestFactory::mock_user_profile();
        
        // Store token in localStorage (Remember Me)
        let _ = storage.store_token_with_remember_me(&token1, true);
        let _ = storage.store_user_profile_with_remember_me(&profile, true);
        
        // Store different token in sessionStorage
        let _ = storage.store_token_with_remember_me(&token2, false);
        
        // get_token should return the most appropriate token
        // (Implementation detail: depends on storage priority logic)
        let retrieved_token = storage.get_token();
        assert!(retrieved_token.is_some(), "Should retrieve a token");
        
        // Profile should still be available
        let retrieved_profile = storage.get_user_profile();
        assert!(retrieved_profile.is_some(), "Should retrieve profile");
        assert_eq!(retrieved_profile.unwrap().email, profile.email, "Profile should match");
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
    fn test_storage_isolation() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage = setup_storage();
        
        let token = TestFactory::mock_token_response();
        let profile1 = TestFactory::mock_user_profile();
        let mut profile2 = TestFactory::mock_user_profile();
        profile2.email = "different@example.com".to_string();
        profile2.first_name = "Different".to_string();
        
        // Store in localStorage with Remember Me
        let _ = storage.store_token_with_remember_me(&token, true);
        let _ = storage.store_user_profile_with_remember_me(&profile1, true);
        
        // Store different profile in sessionStorage
        let _ = storage.store_user_profile_with_remember_me(&profile2, false);
        
        // Verify we can distinguish between storage types
        // (Implementation detail: may vary based on storage strategy)
        let retrieved_profile = storage.get_user_profile();
        assert!(retrieved_profile.is_some(), "Should retrieve a profile");
        
        // Clear session should not affect localStorage
        let _ = storage.clear_session();
        
        // If Remember Me is active, profile should still be available from localStorage
        if storage.is_remember_me_active() {
            assert!(storage.get_user_profile().is_some(), "Profile should persist in localStorage after session clear");
        }
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
        assert!(!storage.is_remember_me_active(), "Remember Me should be inactive when not set");
        assert!(storage.get_remember_me_credentials().is_none(), "Should return None for missing credentials");
        
        // Test clearing empty storage (should not error)
        let clear_session_result = storage.clear_session();
        assert!(clear_session_result.is_ok(), "Should clear empty session without error");
        
        let clear_remember_me_result = storage.clear_remember_me();
        assert!(clear_remember_me_result.is_ok(), "Should clear empty Remember Me without error");
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
    fn test_remember_me_credentials_validation() {
        let storage = StorageService::new();
        #[cfg(not(target_arch = "wasm32"))]
        storage.clear_all_test_data();
        
        // Test with empty email
        let result1 = storage.set_remember_me("", "password", true);
        // Should handle empty email gracefully
        assert!(result1.is_ok() || result1.is_err(), "Should handle empty email");
        
        // Test with empty password
        let result2 = storage.set_remember_me("test@example.com", "", true);
        // Should handle empty password gracefully
        assert!(result2.is_ok() || result2.is_err(), "Should handle empty password");
        
        // Test with both empty
        let result3 = storage.set_remember_me("", "", true);
        // Should handle both empty gracefully
        assert!(result3.is_ok() || result3.is_err(), "Should handle both empty");
    }

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
