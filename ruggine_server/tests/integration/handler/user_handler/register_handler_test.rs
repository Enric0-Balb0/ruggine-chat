use ruggine_server::handler::user_handler::register_handler::register;
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
    use ruggine_server::entity::user::UserStatus;

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
    async fn test_register_handler_creates_user_successfully_with_valid_data() {
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
        assert_eq!(user_response.data().user_status, UserStatus::Active);
        assert!(user_response.data().id > 0, "User should have a valid ID");
        assert!(user_response.data().created_at <= chrono::Utc::now(), "Created date should not be in future");
        
        // Verify new fields from registration are properly set
        assert_eq!(user_response.data().birthday, register_dto.birthday);
        assert_eq!(user_response.data().address, register_dto.address);
        assert_eq!(user_response.data().gender, register_dto.gender);
        
        // Verify default values for auto-generated fields
        assert!(!user_response.data().is_online, "New user should not be online by default");
        assert_eq!(user_response.data().current_action, ruggine_server::entity::user::CurrentAction::Waiting, 
                  "New user should be in Waiting state by default");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_handler_fails_when_user_already_exists() {
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
    async fn test_register_handler_persists_user_data_to_database() {
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
        assert_eq!(stored_user.user_status, UserStatus::Active);
        assert_eq!(stored_user.id, user_response.data().id);
        
        // Verify new fields are correctly stored in database
        assert_eq!(stored_user.birthday, register_dto.birthday);
        assert_eq!(stored_user.address, register_dto.address);
        assert_eq!(stored_user.gender, register_dto.gender);
        
        // Verify default values are correctly set in database
        assert!(!stored_user.is_online, "User should not be online by default in database");
        assert_eq!(stored_user.current_action, ruggine_server::entity::user::CurrentAction::Waiting, 
                  "User should be in Waiting state by default in database");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_handler_hashes_password_before_storage() {
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
    async fn test_register_handler_handles_special_characters_in_user_data() {
        // Arrange: Create user state and registration data with special characters
        let user_state = create_user_state().await;
        let register_dto = UserRegisterDto {
            email: "special.chars+test@example.com".to_string(),
            password: "P@ssw0rd!#$%".to_string(),
            username: "user_with_underscore_123".to_string(),
            first_name: "José María".to_string(),
            last_name: "García-López".to_string(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            address: "Calle de la Paz, 123".to_string(),
            gender: ruggine_server::entity::user::Gender::Male,
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
        
        // Verify new fields with special characters are properly handled
        assert_eq!(user_response.data().birthday, register_dto.birthday);
        assert_eq!(user_response.data().address, register_dto.address);
        assert_eq!(user_response.data().gender, register_dto.gender);

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_handler_supports_concurrent_user_registrations() {
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
    async fn test_register_handler_preserves_all_input_data_in_response() {
        // Arrange: Create user state with specific input data
        let user_state = create_user_state().await;
        let register_dto = UserRegisterDto {
            email: "preserve.test@example.com".to_string(),
            password: "test_password_123".to_string(),
            username: "preserve_user".to_string(),
            first_name: "PreserveFirst".to_string(),
            last_name: "PreserveLast".to_string(),
            birthday: chrono::NaiveDate::from_ymd_opt(1995, 12, 1).unwrap(),
            address: "100 Preserve Street".to_string(),
            gender: ruggine_server::entity::user::Gender::Female,
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
        
        // Verify new fields are preserved from input
        assert_eq!(user_response.data().birthday, register_dto.birthday);
        assert_eq!(user_response.data().address, register_dto.address);
        assert_eq!(user_response.data().gender, register_dto.gender);
        
        // Verify default values for auto-generated fields
        assert!(!user_response.data().is_online, "User should not be online by default");
        assert_eq!(user_response.data().current_action, ruggine_server::entity::user::CurrentAction::Waiting);
        
        // Verify response structure
        assert!(user_response.data().id > 0);
        assert_eq!(user_response.data().user_status, UserStatus::Active);
        assert!(user_response.data().created_at <= chrono::Utc::now());

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_handler_sets_new_user_status_to_active_by_default() {
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
        
        assert_eq!(user_response.data().user_status, UserStatus::Active, "User should be active by default");

        // Verify in database as well
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let stored_user = repository.find_by_email(register_dto.email.clone()).await;
        
        assert!(stored_user.is_some(), "User should exist in database");
        assert_eq!(stored_user.unwrap().user_status, UserStatus::Active, "User should be active in database");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_handler_correctly_stores_all_user_fields_to_database() {
        // Arrange: Create user state and registration data with new fields
        let user_state = create_user_state().await;
        let register_dto = UserRegisterDto {
            email: "newfields.test@example.com".to_string(),
            password: "test_password_456".to_string(),
            username: "newfields_user".to_string(),
            first_name: "NewField".to_string(),
            last_name: "TestUser".to_string(),
            birthday: chrono::NaiveDate::from_ymd_opt(1988, 4, 22).unwrap(),
            address: "456 New Field Avenue, Test City".to_string(),
            gender: ruggine_server::entity::user::Gender::Other,
        };
        
        // Act: Register the user
        let result = register(
            State(user_state),
            ValidatedRequest(register_dto.clone()),
        ).await;

        // Assert: New fields should be correctly stored and returned
        assert!(result.is_ok(), "Registration should succeed");
        let user_response = result.unwrap().0;
        
        // Check that new fields are in the response
        assert_eq!(user_response.data().birthday, register_dto.birthday);
        assert_eq!(user_response.data().address, register_dto.address);
        assert_eq!(user_response.data().gender, register_dto.gender);
        assert_eq!(user_response.data().is_online, false); // Should default to false
        assert_eq!(user_response.data().current_action, ruggine_server::entity::user::CurrentAction::Waiting); // Should default to Waiting

        // Verify new fields are stored in database
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let stored_user = repository.find_by_email(register_dto.email.clone()).await;
        
        assert!(stored_user.is_some(), "User should exist in database");
        let stored_user = stored_user.unwrap();
        
        assert_eq!(stored_user.birthday, register_dto.birthday);
        assert_eq!(stored_user.address, register_dto.address);
        assert_eq!(stored_user.gender, register_dto.gender);
        assert_eq!(stored_user.is_online, false); // Should default to false
        assert_eq!(stored_user.current_action, ruggine_server::entity::user::CurrentAction::Waiting); // Should default to Waiting

        // Cleanup
        cleanup_user(register_dto.email).await;
    }
}
