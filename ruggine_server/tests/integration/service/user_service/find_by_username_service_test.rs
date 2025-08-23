use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::entity::user::UserStatus;

#[cfg(test)]
mod user_service_find_by_username_integration_tests {
    use crate::common::{get_database, cleanup_user_by_email};
    use super::*;

    /// Helper function to create a real user in the database for testing
    async fn create_test_user_for_find_by_username(prefix: &str) -> (String, String) {
        let db = get_database().await;
        let service = UserService::new(&db);

        let dto = UserFactory::unique_fake_user_register_dto(prefix);
        let username = dto.username.clone();
        let email = dto.email.clone();
        
        let create_result = service.create_user(dto).await;
        assert!(create_result.is_ok(), "Failed to create user for find_by_username test");

        (username, email)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_success() {
        // Arrange
        let (test_username, test_email) = create_test_user_for_find_by_username("find_by_username_success").await;
        
        let db = get_database().await;
        let service = UserService::new(&db);

        // Act
        let result = service.find_by_username(test_username.clone()).await;

        // Assert
        assert!(result.is_ok(), "Service call should succeed");
        let user_option = result.unwrap();
        assert!(user_option.is_some(), "User should be found");
        
        let user_dto = user_option.unwrap();
        assert_eq!(user_dto.username, test_username);
        assert!(!user_dto.email.is_empty());
        assert!(!user_dto.first_name.is_empty());
        assert!(!user_dto.last_name.is_empty());

        // Cleanup
        cleanup_user_by_email(test_email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_not_found() {
        // Arrange
        let db = get_database().await;
        let service = UserService::new(&db);
        let non_existent_username = UserFactory::fake_username("nonexistent_service");

        // Act
        let result = service.find_by_username(non_existent_username).await;

        // Assert
        assert!(result.is_ok(), "Service call should succeed even when user not found");
        let user_option = result.unwrap();
        assert!(user_option.is_none(), "User should not be found");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_case_sensitive() {
        // Arrange
        let (test_username, test_email) = create_test_user_for_find_by_username("find_by_username_case").await;
        
        let db = get_database().await;
        let service = UserService::new(&db);

        // Act - Test with different case
        let uppercase_username = test_username.to_uppercase();
        let result = service.find_by_username(uppercase_username).await;

        // Assert - Should not find user (case sensitive)
        assert!(result.is_ok(), "Service call should succeed");
        let user_option = result.unwrap();
        assert!(user_option.is_none(), "Username search should be case sensitive");

        // Cleanup
        cleanup_user_by_email(test_email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_multiple_users() {
        // Arrange - Create multiple users
        let (username1, email1) = create_test_user_for_find_by_username("find_by_username_multi_1").await;
        let (username2, email2) = create_test_user_for_find_by_username("find_by_username_multi_2").await;
        let (username3, email3) = create_test_user_for_find_by_username("find_by_username_multi_3").await;
        
        let db = get_database().await;
        let service = UserService::new(&db);

        // Act & Assert - Find each user by their username
        let result1 = service.find_by_username(username1.clone()).await;
        assert!(result1.is_ok());
        let user1 = result1.unwrap().unwrap();
        assert_eq!(user1.username, username1);

        let result2 = service.find_by_username(username2.clone()).await;
        assert!(result2.is_ok());
        let user2 = result2.unwrap().unwrap();
        assert_eq!(user2.username, username2);

        let result3 = service.find_by_username(username3.clone()).await;
        assert!(result3.is_ok());
        let user3 = result3.unwrap().unwrap();
        assert_eq!(user3.username, username3);

        // Verify each user has different data
        assert_ne!(user1.id, user2.id);
        assert_ne!(user2.id, user3.id);
        assert_ne!(user1.id, user3.id);

        // Cleanup
        cleanup_user_by_email(email1).await;
        cleanup_user_by_email(email2).await;
        cleanup_user_by_email(email3).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_with_different_user_statuses() {
        // Arrange - Create user with active status
        let (test_username, test_email) = create_test_user_for_find_by_username("find_by_username_status").await;
        
        let db = get_database().await;
        let service = UserService::new(&db);

        // Act
        let result = service.find_by_username(test_username.clone()).await;

        // Assert
        assert!(result.is_ok());
        let user_option = result.unwrap();
        assert!(user_option.is_some());
        
        let user_dto = user_option.unwrap();
        assert_eq!(user_dto.username, test_username);
        assert_eq!(user_dto.user_status, UserStatus::Active); // Default status from factory

        // Cleanup
        cleanup_user_by_email(test_email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_returns_complete_user_dto() {
        // Arrange
        let (test_username, test_email) = create_test_user_for_find_by_username("find_by_username_complete").await;
        
        let db = get_database().await;
        let service = UserService::new(&db);

        // Act
        let result = service.find_by_username(test_username.clone()).await;

        // Assert
        assert!(result.is_ok());
        let user_dto = result.unwrap().unwrap();

        // Verify all required fields are present and valid
        assert!(user_dto.id > 0);
        assert_eq!(user_dto.username, test_username);
        assert!(!user_dto.email.is_empty());
        assert!(!user_dto.first_name.is_empty());
        assert!(!user_dto.last_name.is_empty());
        assert!(!user_dto.address.is_empty());
        
        // Verify timestamps are set
        assert!(user_dto.created_at.timestamp() > 0);
        assert!(user_dto.updated_at.timestamp() > 0);
        
        // Verify default values
        assert_eq!(user_dto.user_status, UserStatus::Active);
        assert!(!user_dto.is_online); // Default value from factory

        // Password should not be included in UserReadDto
        // This is inherent in the DTO design, no explicit check needed

        // Cleanup
        cleanup_user_by_email(test_email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_with_special_characters() {
        // Arrange - Create user with special characters in username (if allowed by validation)
        let db = get_database().await;
        let service = UserService::new(&db);

        // Create user with underscore in username (common case)
        let dto = UserFactory::unique_fake_user_register_dto("find_username_special");
        let special_username = format!("{}_test", dto.username);
        let email = dto.email.clone();
        
        let mut special_dto = dto;
        special_dto.username = special_username.clone();
        
        let create_result = service.create_user(special_dto).await;
        assert!(create_result.is_ok(), "Failed to create user with special characters");

        // Act
        let result = service.find_by_username(special_username.clone()).await;

        // Assert
        assert!(result.is_ok());
        let user_option = result.unwrap();
        assert!(user_option.is_some());
        
        let user_dto = user_option.unwrap();
        assert_eq!(user_dto.username, special_username);

        // Cleanup
        cleanup_user_by_email(email).await;
    }
}
