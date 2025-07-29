use ruggine_server::handler::user_handler::update_profile_handler::update_profile;
use ruggine_server::dto::user_dto::{UserReadDto, ProfileUpdateDto};
use ruggine_server::entity::user::{User, Gender};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::config::database::DatabaseTrait;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::state::user_state::UserState;
use axum::{extract::State, Extension, Json};
use chrono::NaiveDate;
use crate::common::cleanup_user;

#[cfg(test)]
mod update_profile_handler_integration_tests {
    use validator::Validate;
    use ruggine_server::error::request_error::ValidatedRequest;
    use crate::get_database;
    use super::*;

    /// Helper function to create a real user in the database for testing
    async fn create_test_user(prefix: &str) -> (User, String) {
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);
        
        let user_dto = UserFactory::unique_fake_user_register_dto(prefix);
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for update test");
        
        // Get the created user from database
        let user_option = repository.find_by_email(user_dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        (user_option.unwrap(), user_dto.password.clone())
    }

    #[tokio::test]
    async fn test_update_profile_success_complete() {
        // Arrange: Create a user and prepare update data
        let (user, _) = create_test_user("update_complete").await;
        let db = get_database().await;
        let user_state = UserState::new(&db);

        let update_dto = UserFactory::fake_user_update_dto_from_user_read_dto(
            &UserReadDto::from(user.clone()), "update_complete",
        );


        // Act: Call the update handler
        let result = update_profile(
            Extension(user.clone()),
            State(user_state),
            ValidatedRequest(update_dto.clone()),
        ).await;

        // Assert: Verify the update was successful
        assert!(result.is_ok(), "Update profile should succeed");
        let response = result.unwrap().0;
        let updated_user = response.data();

        assert_eq!(updated_user.first_name, update_dto.first_name.unwrap());
        assert_eq!(updated_user.last_name, update_dto.last_name.unwrap());
        assert_eq!(updated_user.birthday, update_dto.birthday.unwrap());
        assert_eq!(updated_user.address, update_dto.address.unwrap());
        assert_eq!(updated_user.gender, update_dto.gender.unwrap());
        assert_eq!(updated_user.id, user.id);
        assert_eq!(updated_user.email, user.email); // Email should remain unchanged

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_profile_success_partial() {
        // Arrange: Create a user and prepare partial update data
        let (user, _) = create_test_user("update_partial").await;
        let db = get_database().await;
        let user_state = UserState::new(&db);

        let update_dto = ProfileUpdateDto {
            first_name: Some("PartialFirst".to_string()),
            last_name: None,
            birthday: None,
            address: Some("Partial Address Update".to_string()),
            gender: None,
        };

        // Act: Call the update handler
        let result = update_profile(
            Extension(user.clone()),
            State(user_state),
            ValidatedRequest(update_dto.clone()),
        ).await;

        // Assert: Verify only specified fields were updated
        assert!(result.is_ok(), "Partial update should succeed");
        let response = result.unwrap().0;
        let updated_user = response.data();

        assert_eq!(updated_user.first_name, "PartialFirst");
        assert_eq!(updated_user.last_name, user.last_name); // Should remain unchanged
        assert_eq!(updated_user.birthday, user.birthday); // Should remain unchanged
        assert_eq!(updated_user.address, "Partial Address Update");
        assert_eq!(updated_user.gender, user.gender); // Should remain unchanged

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_profile_no_updates() {
        // Arrange: Create a user and prepare empty update data
        let (user, _) = create_test_user("update_empty").await;
        let db = get_database().await;
        let user_state = UserState::new(&db);

        let empty_update_dto = UserFactory::fake_user_update_dto_empty();

        // Act: Call the update handler
        let result = update_profile(
            Extension(user.clone()),
            State(user_state),
            ValidatedRequest(empty_update_dto),
        ).await;

        // Assert: Verify no updates error
        assert!(result.is_err(), "Should fail with no updates error");
        match result.unwrap_err() {
            ApiError::UserError(ruggine_server::error::user_error::UserError::NoFieldsToUpdate) => {
                // Expected error
            },
            other => panic!("Expected UserError::NoFieldsToUpdate, got: {:?}", other),
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_profile_with_different_genders() {
        // Arrange: Create a user
        let (user, _) = create_test_user("update_genders").await;
        let db = get_database().await;
        let user_state = UserState::new(&db);

        // Test updating to each gender
        let genders = vec![Gender::Male, Gender::Female, Gender::Other];
        
        for (i, gender) in genders.iter().enumerate() {
            let update_dto = ProfileUpdateDto {
                first_name: Some(format!("User{}", i)),
                last_name: None,
                birthday: None,
                address: None,
                gender: Some(gender.clone()),
            };

            // Act: Update the user
            let result = update_profile(
                Extension(user.clone()),
                State(user_state.clone()),
                ValidatedRequest(update_dto),
            ).await;

            // Assert: Verify the update was successful
            assert!(result.is_ok(), "Gender update should succeed for {:?}", gender);
            let response = result.unwrap().0;
            let updated_user = response.data();
            assert_eq!(updated_user.gender, *gender);
            assert_eq!(updated_user.first_name, format!("User{}", i));
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_profile_preserves_unchanged_fields() {
        // Arrange: Create a user and save original values
        let (user, _) = create_test_user("update_preserve").await;
        let db = get_database().await;
        let user_state = UserState::new(&db);

        let original_username = user.username.clone();
        let original_email = user.email.clone();
        let original_user_type = user.user_type.clone();
        let original_user_status = user.user_status.clone();

        let update_dto = ProfileUpdateDto {
            first_name: Some("NewFirst".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        // Act: Update only first_name
        let result = update_profile(
            Extension(user.clone()),
            State(user_state),
            ValidatedRequest(update_dto),
        ).await;

        // Assert: Verify unchanged fields are preserved
        assert!(result.is_ok(), "Update should succeed");
        let response = result.unwrap().0;
        let updated_user = response.data();

        assert_eq!(updated_user.first_name, "NewFirst");
        assert_eq!(updated_user.username, original_username);
        assert_eq!(updated_user.email, original_email);
        assert_eq!(updated_user.user_type, original_user_type);
        assert_eq!(updated_user.user_status, original_user_status);

        // Cleanup
        cleanup_user(user.email).await;
    }
}
