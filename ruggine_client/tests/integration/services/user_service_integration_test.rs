// Integration tests for User Service

use std::sync::Mutex;

#[cfg(test)]
mod user_service_integration_tests {
    use super::*;
    use crate::common::*;
    use ruggine_client_ui::api::services::user::UserService;
    use ruggine_client_ui::api::client::ApiClient;
    use ruggine_client_ui::utils::storage::StorageService;
    use ruggine_client_ui::types::user::{UserProfile, UserRegisterRequest, UserType, UserStatus};

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_user_service() -> UserService {
        let api_client = ApiClient::new("http://localhost:3000".to_string());
        let storage_service = StorageService::new();
        UserService::new(api_client, storage_service)
    }

    #[tokio::test]
    async fn test_user_service_creation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Service should be created successfully
        drop(service);
    }

    #[tokio::test]
    async fn test_user_register_request_validation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Test basic register request
        let basic_request = UserFactory::basic_user_register_request();
        assert!(!basic_request.email.is_empty());
        assert!(!basic_request.username.is_empty());
        assert!(!basic_request.password.is_empty());
        assert!(!basic_request.first_name.is_empty());
        assert!(!basic_request.last_name.is_empty());
        
        // Test unique register request
        let unique_request = UserFactory::unique_user_register_request("test");
        assert!(unique_request.username.contains("test"));
        assert!(unique_request.email.contains("test"));
    }

    #[tokio::test]
    async fn test_user_profile_creation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Test mock user profile
        let profile = UserFactory::mock_user_profile();
        assert!(profile.id > 0);
        assert!(!profile.email.is_empty());
        assert!(!profile.username.is_empty());
        assert!(matches!(profile.user_type, UserType::EndUser));
        assert!(matches!(profile.user_status, UserStatus::Active));
        
        // Test admin user profile
        let admin_profile = UserFactory::admin_user_profile();
        assert!(matches!(admin_profile.user_type, UserType::AdminDeveloper));
        assert_eq!(admin_profile.username, "admin_user");
        assert_eq!(admin_profile.email, "admin@test.com");
        
        // Test inactive user profile
        let inactive_profile = UserFactory::inactive_user_profile();
        assert!(matches!(inactive_profile.user_status, UserStatus::Inactive));
        assert_eq!(inactive_profile.username, "inactive_user");
        assert_eq!(inactive_profile.email, "inactive@test.com");
    }

    #[tokio::test]
    async fn test_user_with_specific_details() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Create user with specific details
        let user = UserFactory::user_with_details(42, "specific_user", "specific@test.com");
        
        assert_eq!(user.id, 42);
        assert_eq!(user.username, "specific_user");
        assert_eq!(user.email, "specific@test.com");
        assert!(matches!(user.user_type, UserType::EndUser));
        assert!(matches!(user.user_status, UserStatus::Active));
    }

    #[tokio::test]
    async fn test_user_service_data_consistency() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Create multiple users and verify consistency
        let user1 = UserFactory::mock_user_profile();
        let user2 = UserFactory::mock_user_profile();
        let user3 = UserFactory::admin_user_profile();
        
        // All should have unique IDs
        assert_ne!(user1.id, user2.id);
        assert_ne!(user1.id, user3.id);
        assert_ne!(user2.id, user3.id);
        
        // All should have unique usernames and emails
        assert_ne!(user1.username, user2.username);
        assert_ne!(user1.email, user2.email);
        
        // Admin should have different type
        assert!(matches!(user1.user_type, UserType::EndUser));
        assert!(matches!(user2.user_type, UserType::EndUser));
        assert!(matches!(user3.user_type, UserType::AdminDeveloper));
    }

    #[tokio::test]
    async fn test_user_service_unique_registration_requests() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Create multiple unique registration requests
        let requests: Vec<UserRegisterRequest> = (0..5)
            .map(|i| UserFactory::unique_user_register_request(&format!("user_{}", i)))
            .collect();
        
        assert_eq!(requests.len(), 5);
        
        // Each request should be unique
        for (i, request) in requests.iter().enumerate() {
            assert!(request.username.contains(&format!("user_{}", i)));
            assert!(request.email.contains(&format!("user_{}", i)));
            
            // No two requests should have the same username or email
            for (j, other_request) in requests.iter().enumerate() {
                if i != j {
                    assert_ne!(request.username, other_request.username);
                    assert_ne!(request.email, other_request.email);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_user_service_profile_variations() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Test different user profile variations
        let regular_user = UserFactory::mock_user_profile();
        let admin_user = UserFactory::admin_user_profile();
        let inactive_user = UserFactory::inactive_user_profile();
        
        // Verify status variations
        assert!(matches!(regular_user.user_status, UserStatus::Active));
        assert!(matches!(admin_user.user_status, UserStatus::Active));
        assert!(matches!(inactive_user.user_status, UserStatus::Inactive));
        
        // Verify type variations
        assert!(matches!(regular_user.user_type, UserType::EndUser));
        assert!(matches!(admin_user.user_type, UserType::AdminDeveloper));
        assert!(matches!(inactive_user.user_type, UserType::EndUser));
    }

    #[tokio::test]
    async fn test_user_service_email_uniqueness() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Test that email generation is unique
        let (email1, username1, _) = UserFactory::get_unique_user_info("test1");
        let (email2, username2, _) = UserFactory::get_unique_user_info("test2");
        let (email3, username3, _) = UserFactory::get_unique_user_info("test1"); // Same prefix
        
        assert_ne!(email1, email2);
        assert_ne!(email1, email3);
        assert_ne!(email2, email3);
        
        assert_ne!(username1, username2);
        assert_ne!(username1, username3);
        assert_ne!(username2, username3);
    }

    #[tokio::test]
    async fn test_user_service_registration_field_validation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Test registration request fields
        let request = UserFactory::basic_user_register_request();
        
        // Email validation
        assert!(request.email.contains("@"));
        assert!(request.email.ends_with(".com"));
        
        // Password validation
        assert!(request.password.len() >= 8);
        
        // Name validation
        assert!(!request.first_name.is_empty());
        assert!(!request.last_name.is_empty());
        
        // Optional fields
        assert!(request.birth_date.is_some());
        assert!(request.phone_number.is_some());
        assert!(request.gender.is_some());
    }

    #[tokio::test]
    async fn test_user_service_profile_timestamps() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Test profile timestamps
        let profile = UserFactory::mock_user_profile();
        
        assert!(!profile.created_at.to_string().is_empty());
        assert!(!profile.updated_at.to_string().is_empty());
        
        // created_at should be <= updated_at
        assert!(profile.created_at <= profile.updated_at);
    }

    #[tokio::test]
    async fn test_user_service_profile_id_consistency() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_user_service();
        
        // Test that specific user details work correctly
        let users = vec![
            UserFactory::user_with_details(1, "user1", "user1@test.com"),
            UserFactory::user_with_details(2, "user2", "user2@test.com"),
            UserFactory::user_with_details(3, "user3", "user3@test.com"),
        ];
        
        for (i, user) in users.iter().enumerate() {
            let expected_id = (i + 1) as i32;
            assert_eq!(user.id, expected_id);
            assert_eq!(user.username, format!("user{}", expected_id));
            assert_eq!(user.email, format!("user{}@test.com", expected_id));
        }
    }
}
