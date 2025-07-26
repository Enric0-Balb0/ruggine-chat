use ruggine_server::handler::user::register_handler::register;
use ruggine_server::dto::user_dto::UserRegisterDto;
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::service::user_service::UserService;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::state::user_state::UserState;
use ruggine_server::error::{api_error::ApiError, user_error::UserError, request_error::ValidatedRequest};
use axum::extract::State;
use std::sync::Arc;
use crate::common::cleanup_user;

#[cfg(test)]
mod register_handler_integration_tests {
    use crate::get_database;
    use super::*;

    /// Helper function to create a real user state with database connections
    async fn create_user_state() -> UserState {
        let db = get_database().await;
        let user_service = Arc::new(UserService::new(&db));
        let user_repo = Arc::new(UserRepository::new(&db));
        
        UserState {
            user_service,
            user_repo,
        }
    }

    #[tokio::test]
    async fn test_register_success_with_valid_data() {
        // Arrange: Create user state and registration data
        let user_state = create_user_state().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("register_success");
        
        // Act: Call register handler
        let result = register(
            State(user_state),
            ValidatedRequest(register_dto.clone()),
        ).await;

        // Assert: Should succeed and return user data
        assert!(result.is_ok(), "Registration should succeed with valid data");
        let user_response = result.unwrap().0;
        
        assert_eq!(user_response.data().email, register_dto.email);
        assert_eq!(user_response.data().username, register_dto.username);
        assert_eq!(user_response.data().first_name, register_dto.first_name);
        assert_eq!(user_response.data().last_name, register_dto.last_name);
        assert_eq!(user_response.data().is_active, 1);
        assert!(user_response.data().id > 0, "User should have a valid ID");
        assert!(user_response.data().created_at <= chrono::Utc::now(), "Created date should not be in future");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_user_already_exists() {
        // Arrange: Create user state and register a user first
        let user_state = create_user_state().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("register_duplicate");
        
        // Register user first time
        let first_result = register(
            State(user_state.clone()),
            ValidatedRequest(register_dto.clone()),
        ).await;
        assert!(first_result.is_ok(), "First registration should succeed");

        // Act: Try to register the same user again
        let result = register(
            State(user_state),
            ValidatedRequest(register_dto.clone()),
        ).await;

        // Assert: Should fail with UserAlreadyExists error
        assert!(result.is_err(), "Second registration should fail");
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserAlreadyExists(_)) => {
                // Expected error
            }
            other => panic!("Expected UserError::UserAlreadyExists, got {:?}", other),
        }

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_creates_user_in_database() {
        // Arrange: Create user state and registration data
        let user_state = create_user_state().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("register_db_check");
        
        // Act: Register the user
        let result = register(
            State(user_state),
            ValidatedRequest(register_dto.clone()),
        ).await;
        assert!(result.is_ok(), "Registration should succeed");
        let user_response = result.unwrap().0;

        // Assert: Verify user exists in database
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let stored_user = repository.find_by_email(register_dto.email.clone()).await;
        
        assert!(stored_user.is_some(), "User should exist in database");
        let stored_user = stored_user.unwrap();
        
        assert_eq!(stored_user.email, register_dto.email);
        assert_eq!(stored_user.username, register_dto.username);
        assert_eq!(stored_user.first_name, register_dto.first_name);
        assert_eq!(stored_user.last_name, register_dto.last_name);
        assert_eq!(stored_user.is_active, 1);
        assert_eq!(stored_user.id, user_response.data().id);

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_password_is_hashed() {
        // Arrange: Create user state and registration data with known password
        let user_state = create_user_state().await;
        let mut register_dto = UserFactory::unique_fake_user_register_dto("register_password");
        let original_password = "plain_text_password_123";
        register_dto.password = original_password.to_string();
        
        // Act: Register the user
        let result = register(
            State(user_state),
            ValidatedRequest(register_dto.clone()),
        ).await;
        assert!(result.is_ok(), "Registration should succeed");

        // Assert: Verify password is hashed in database
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let stored_user = repository.find_by_email(register_dto.email.clone()).await;
        
        assert!(stored_user.is_some(), "User should exist in database");
        let stored_user = stored_user.unwrap();
        
        // Password should be hashed, not plain text
        assert_ne!(stored_user.password, original_password, "Password should be hashed");
        assert!(stored_user.password.len() > original_password.len(), "Hashed password should be longer");
        assert!(stored_user.password.starts_with("$"), "Hashed password should start with hash identifier");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_with_special_characters() {
        // Arrange: Create user state and registration data with special characters
        let user_state = create_user_state().await;
        let register_dto = UserRegisterDto {
            email: "special.chars+test@example.com".to_string(),
            password: "P@ssw0rd!#$%".to_string(),
            username: "user_with_underscore_123".to_string(),
            first_name: "José María".to_string(),
            last_name: "García-López".to_string(),
        };
        
        // Act: Register the user
        let result = register(
            State(user_state),
            ValidatedRequest(register_dto.clone()),
        ).await;

        // Assert: Should succeed with special characters
        assert!(result.is_ok(), "Registration should succeed with special characters");
        let user_response = result.unwrap().0;
        
        assert_eq!(user_response.data().email, register_dto.email);
        assert_eq!(user_response.data().username, register_dto.username);
        assert_eq!(user_response.data().first_name, register_dto.first_name);
        assert_eq!(user_response.data().last_name, register_dto.last_name);

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_multiple_users_concurrently() {
        // Arrange: Create user state and multiple registration data
        let user_state = create_user_state().await;
        let register_dto1 = UserFactory::unique_fake_user_register_dto("register_concurrent1");
        let register_dto2 = UserFactory::unique_fake_user_register_dto("register_concurrent2");
        
        // Act: Register users concurrently
        let (result1, result2) = tokio::join!(
            register(State(user_state.clone()), ValidatedRequest(register_dto1.clone())),
            register(State(user_state), ValidatedRequest(register_dto2.clone()))
        );

        // Assert: Both registrations should succeed
        assert!(result1.is_ok(), "First concurrent registration should succeed");
        assert!(result2.is_ok(), "Second concurrent registration should succeed");
        
        let user1 = result1.unwrap().0;
        let user2 = result2.unwrap().0;
        
        // Users should have different IDs
        assert_ne!(user1.data().id, user2.data().id, "Users should have different IDs");
        assert_ne!(user1.data().email, user2.data().email, "Users should have different emails");

        // Cleanup
        cleanup_user(register_dto1.email).await;
        cleanup_user(register_dto2.email).await;
    }

    #[tokio::test]
    async fn test_register_preserves_input_data() {
        // Arrange: Create user state with specific input data
        let user_state = create_user_state().await;
        let register_dto = UserRegisterDto {
            email: "preserve.test@example.com".to_string(),
            password: "test_password_123".to_string(),
            username: "preserve_user".to_string(),
            first_name: "PreserveFirst".to_string(),
            last_name: "PreserveLast".to_string(),
        };
        
        // Act: Register the user
        let result = register(
            State(user_state),
            ValidatedRequest(register_dto.clone()),
        ).await;

        // Assert: All input data should be preserved in response
        assert!(result.is_ok(), "Registration should succeed");
        let user_response = result.unwrap().0;
        
        assert_eq!(user_response.data().email, register_dto.email);
        assert_eq!(user_response.data().username, register_dto.username);
        assert_eq!(user_response.data().first_name, register_dto.first_name);
        assert_eq!(user_response.data().last_name, register_dto.last_name);
        
        // Verify response structure
        assert!(user_response.data().id > 0);
        assert_eq!(user_response.data().is_active, 1);
        assert!(user_response.data().created_at <= chrono::Utc::now());

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_user_is_active_by_default() {
        // Arrange: Create user state and registration data
        let user_state = create_user_state().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("register_active_default");
        
        // Act: Register the user
        let result = register(
            State(user_state),
            ValidatedRequest(register_dto.clone()),
        ).await;

        // Assert: User should be active by default
        assert!(result.is_ok(), "Registration should succeed");
        let user_response = result.unwrap().0;
        
        assert_eq!(user_response.data().is_active, 1, "User should be active by default");

        // Verify in database as well
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let stored_user = repository.find_by_email(register_dto.email.clone()).await;
        
        assert!(stored_user.is_some(), "User should exist in database");
        assert_eq!(stored_user.unwrap().is_active, 1, "User should be active in database");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }
}
