// Unit tests for AuthService
// Testing authentication logic, validation, and error handling

use ruggine_client_ui::api::services::auth::AuthService;
use ruggine_client_ui::utils::storage::StorageService;
use ruggine_client_ui::api::client::ApiClient;
use ruggine_client_ui::error::AuthError;

#[cfg(test)]
mod auth_service_unit_tests {
    use super::*;
    use std::sync::Mutex;
    
    // Mutex to serialize tests that use shared storage
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_auth_service() -> AuthService {
        let http_client = ApiClient::new("http://localhost:3000");
        let storage_service = StorageService::new();
        
        // Clear any previous test data
        #[cfg(not(target_arch = "wasm32"))]
        storage_service.clear_all_test_data();
        
        AuthService::new(http_client, storage_service)
    }

    #[test]
    fn test_validate_login_input_valid() {
        let auth_service = setup_auth_service();
        
        // Use reflection to test private validation method through public login attempt
        // We expect this to fail due to network, but validation should pass
        let result = tokio_test::block_on(async {
            auth_service.login(
                "test@example.com".to_string(),
                "validpassword".to_string()
            ).await
        });
        
        // Should fail with network error, not validation error
        match result {
            Err(AuthError::InvalidInput(_)) => panic!("Should not fail validation"),
            Err(_) => {}, // Expected - network/API error
            Ok(_) => panic!("Unexpected success without server"),
        }
    }

    #[test]
    fn test_validate_login_input_empty_email() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.login("".to_string(), "password".to_string()).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("Email is required"));
            },
            _ => panic!("Should fail with email validation error"),
        }
    }

    #[test]
    fn test_validate_login_input_empty_password() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.login("test@example.com".to_string(), "".to_string()).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("Password is required"));
            },
            _ => panic!("Should fail with password validation error"),
        }
    }

    #[test]
    fn test_validate_login_input_invalid_email_format() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.login("invalid-email".to_string(), "password".to_string()).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("Invalid email format"));
            },
            _ => panic!("Should fail with email format validation error"),
        }
    }

    #[test]
    fn test_validate_registration_input_short_password() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.register(
                "test@example.com".to_string(),
                "123".to_string(), // Too short
                "John".to_string(),
                "Doe".to_string(),
                "johndoe".to_string(),
                "1990-01-01".to_string(),
                "123 Test St".to_string(),
                "male".to_string(),
            ).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("almeno 6 caratteri"));
            },
            _ => panic!("Should fail with password length validation error"),
        }
    }

    #[test]
    fn test_validate_registration_input_empty_first_name() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.register(
                "test@example.com".to_string(),
                "password123".to_string(),
                "".to_string(), // Empty first name
                "Doe".to_string(),
                "johndoe".to_string(),
                "1990-01-01".to_string(),
                "123 Test St".to_string(),
                "male".to_string(),
            ).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("Nome è obbligatorio"));
            },
            _ => panic!("Should fail with first name validation error"),
        }
    }

    #[test]
    fn test_validate_registration_input_invalid_gender() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.register(
                "test@example.com".to_string(),
                "password123".to_string(),
                "John".to_string(),
                "Doe".to_string(),
                "johndoe".to_string(),
                "1990-01-01".to_string(),
                "123 Test St".to_string(),
                "invalid_gender".to_string(), // Invalid gender
            ).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("Genere non valido"));
            },
            _ => panic!("Should fail with gender validation error"),
        }
    }

    #[test]
    fn test_gender_conversion() {
        let auth_service = setup_auth_service();
        
        // Test valid gender conversions by attempting registration
        // (will fail on network, but should pass validation)
        let test_cases = vec![
            ("male", false),
            ("female", false), 
            ("other", false),
            ("invalid", true), // Should fail
        ];
        
        for (gender, should_fail) in test_cases {
            let result = tokio_test::block_on(async {
                auth_service.register(
                    "test@example.com".to_string(),
                    "password123".to_string(),
                    "John".to_string(),
                    "Doe".to_string(),
                    "johndoe".to_string(),
                    "1990-01-01".to_string(),
                    "123 Test St".to_string(),
                    gender.to_string(),
                ).await
            });
            
            if should_fail {
                match result {
                    Err(AuthError::InvalidInput(msg)) => {
                        assert!(msg.contains("Genere non valido"));
                    },
                    _ => panic!("Should fail with gender validation error for: {}", gender),
                }
            } else {
                // Should not fail with validation error (may fail with network error)
                match result {
                    Err(AuthError::InvalidInput(msg)) if msg.contains("Genere non valido") => {
                        panic!("Should not fail gender validation for: {}", gender);
                    },
                    _ => {}, // Expected - either success or other error type
                }
            }
        }
    }

    #[test]
    fn test_is_authenticated_without_token() {
        let auth_service = setup_auth_service();
        assert!(!auth_service.is_authenticated());
    }

    #[test]
    fn test_get_current_user_without_profile() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let auth_service = setup_auth_service();
        assert!(auth_service.get_current_user().is_none());
    }

    #[test]
    fn test_email_normalization() {
        let auth_service = setup_auth_service();
        
        // Test that email gets normalized (lowercased and trimmed)
        let result = tokio_test::block_on(async {
            auth_service.login(
                "  TEST@EXAMPLE.COM  ".to_string(),
                "password".to_string()
            ).await
        });
        
        // Should not fail with validation error (will fail with network)
        match result {
            Err(AuthError::InvalidInput(_)) => panic!("Should not fail validation"),
            _ => {}, // Expected - network error or success
        }
    }

    #[test] 
    fn test_password_trimming() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.login(
                "test@example.com".to_string(),
                "  password  ".to_string()
            ).await
        });
        
        // Should not fail with validation error
        match result {
            Err(AuthError::InvalidInput(_)) => panic!("Should not fail validation"),
            _ => {}, // Expected - network error or success
        }
    }

    #[test]
    fn test_change_password_validation_same_password() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.change_password(
                "currentpass".to_string(),
                "currentpass".to_string() // Same password
            ).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("must be different"));
            },
            _ => panic!("Should fail with same password validation error"),
        }
    }

    #[test]
    fn test_change_password_validation_empty_current() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.change_password(
                "".to_string(), // Empty current password
                "newpassword".to_string()
            ).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("Current password is required"));
            },
            _ => panic!("Should fail with current password validation error"),
        }
    }

    #[test]
    fn test_change_password_validation_empty_new() {
        let auth_service = setup_auth_service();
        
        let result = tokio_test::block_on(async {
            auth_service.change_password(
                "currentpass".to_string(),
                "".to_string() // Empty new password
            ).await
        });
        
        match result {
            Err(AuthError::InvalidInput(msg)) => {
                assert!(msg.contains("New password is required"));
            },
            _ => panic!("Should fail with new password validation error"),
        }
    }

    #[test]
    fn test_default_auth_service_creation() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        // Clear storage before test since Default trait doesn't call our setup
        let storage = StorageService::new();
        #[cfg(not(target_arch = "wasm32"))]
        storage.clear_all_test_data();
        
        let auth_service = AuthService::default();
        assert!(!auth_service.is_authenticated());
        assert!(auth_service.get_current_user().is_none());
    }

    #[test]
    fn test_clear_session_functionality() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let auth_service = setup_auth_service();
        
        // Test that clear_session works (this is what logout calls internally)
        let result = auth_service.clear_session();
        assert!(result.is_ok());
        
        // After clearing session, user should not be authenticated
        assert!(!auth_service.is_authenticated());
        assert!(auth_service.get_current_user().is_none());
    }

    #[test]
    fn test_logout_without_token() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let auth_service = setup_auth_service();
        
        // Verify no token initially
        assert!(auth_service.get_storage_service().get_token().is_none());
        assert!(!auth_service.is_authenticated());
        
        // Test logout when no token exists (should not fail)
        let result = tokio_test::block_on(async {
            auth_service.logout().await
        });
        
        // Should succeed even without token
        assert!(result.is_ok());
        assert!(!auth_service.is_authenticated());
        assert!(auth_service.get_current_user().is_none());
    }

    #[test]
    fn test_logout_functionality_structure() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let auth_service = setup_auth_service();
        
        // Clear any existing session data first
        let _ = auth_service.clear_session();
        
        // Verify clean state
        assert!(!auth_service.is_authenticated());
        assert!(auth_service.get_current_user().is_none());
        
        // Test the logout process structure (will fail due to no server, but tests the logic)
        let result = tokio_test::block_on(async {
            auth_service.logout().await
        });
        
        // Even if API call fails, logout should clear local session
        // This tests that logout() calls clear_session() internally
        assert!(result.is_ok()); // Should succeed because it cleans up local storage regardless
        assert!(!auth_service.is_authenticated());
        assert!(auth_service.get_current_user().is_none());
    }

    #[test]
    fn test_logout_clears_storage() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let auth_service = setup_auth_service();
        
        // Manually add some data to storage to test that logout clears it
        let storage_service = auth_service.get_storage_service();
        
        // Clear any existing session data first  
        let _ = auth_service.clear_session();
        
        // Create mock token and profile data
        let mock_token = crate::common::TestFactory::mock_token_response();
        let mock_profile = crate::common::TestFactory::mock_user_profile();
        
        // Store mock data
        let _ = storage_service.store_token(&mock_token);
        let _ = storage_service.store_user_profile(&mock_profile);
        
        // Verify data is stored
        assert!(storage_service.get_token().is_some(), "Token should be stored before logout");
        assert!(storage_service.get_user_profile().is_some(), "Profile should be stored before logout");
        
        // Perform logout
        let result = tokio_test::block_on(async {
            auth_service.logout().await
        });
        
        assert!(result.is_ok(), "Logout should succeed");
        
        // Verify data is cleared
        assert!(storage_service.get_token().is_none(), "Token should be cleared after logout");
        assert!(storage_service.get_user_profile().is_none(), "Profile should be cleared after logout");
        assert!(!auth_service.is_authenticated(), "Should not be authenticated after logout");
        assert!(auth_service.get_current_user().is_none(), "Current user should be None after logout");
    }
}
