use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::dto::user_dto::{ProfileUpdateDto};
use ruggine_server::entity::user::{User, Gender};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::user_error::UserError;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;
use chrono::NaiveDate;
use crate::common::cleanup_user;

#[cfg(test)]
mod user_service_update_profile_integration_tests {
    use ruggine_server::dto::user_dto::UserReadDto;
    use ruggine_server::entity::user::UpdateUser;
    use crate::get_database;
    use super::*;

    /// Helper function to create a real user in the database for testing
    async fn create_test_user_for_update(prefix: &str) -> User {
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let dto = UserFactory::unique_fake_user_register_dto(prefix);
        let create_result = service.create_user(dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for update test");

        // Get the created user from database
        let user_option = repository.find_by_email(dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        user_option.unwrap()
    }

    #[tokio::test]
    async fn test_update_user_profile_success_complete() {
        // Arrange: Create a user in the database
        let user = create_test_user_for_update("service_update_complete").await;
        let db = get_database().await;
        let service = UserService::new(&db);

        let update_dto = ProfileUpdateDto {
            first_name: Some("ServiceUpdatedFirst".to_string()),
            last_name: Some("ServiceUpdatedLast".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(1985, 12, 25).unwrap()),
            address: Some("Service Updated Address 789".to_string()),
            gender: Some(Gender::Other),
        };

        // Act: Call the update_user_profile method
        let result = service.update_user_profile(user.id, UpdateUser::from_dto(update_dto.clone())).await;

        // Assert: Verify the update was successful
        assert!(result.is_ok(), "Update user profile should succeed");
        let updated_user = result.unwrap();

        assert_eq!(updated_user.first_name, "ServiceUpdatedFirst");
        assert_eq!(updated_user.last_name, "ServiceUpdatedLast");
        assert_eq!(updated_user.birthday, NaiveDate::from_ymd_opt(1985, 12, 25).unwrap());
        assert_eq!(updated_user.address, "Service Updated Address 789");
        assert_eq!(updated_user.gender, Gender::Other);
        assert_eq!(updated_user.id, user.id);
        assert_eq!(updated_user.email, user.email); // Email should remain unchanged

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_user_profile_success_partial() {
        // Arrange: Create a user in the database
        let user = create_test_user_for_update("service_update_partial").await;
        let db = get_database().await;
        let service = UserService::new(&db);

        let update_dto = ProfileUpdateDto {
            first_name: Some("ServicePartialFirst".to_string()),
            last_name: None,
            birthday: None,
            address: Some("Service Partial Address".to_string()),
            gender: None,
        };

        // Act: Call the update_user_profile method
        let result = service.update_user_profile(user.id, UpdateUser::from_dto(update_dto.clone())).await;

        // Assert: Verify only specified fields were updated
        assert!(result.is_ok(), "Partial update should succeed");
        let updated_user = result.unwrap();

        assert_eq!(updated_user.first_name, "ServicePartialFirst");
        assert_eq!(updated_user.last_name, user.last_name); // Should remain unchanged
        assert_eq!(updated_user.birthday, user.birthday); // Should remain unchanged
        assert_eq!(updated_user.address, "Service Partial Address");
        assert_eq!(updated_user.gender, user.gender); // Should remain unchanged

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_user_profile_no_updates() {
        // Arrange: Create a user in the database
        let mut user = create_test_user_for_update("service_update_empty").await;
        let db = get_database().await;
        let service = UserService::new(&db);

        let empty_update_dto = UserFactory::fake_user_update_dto_empty();

        // Act: Call the update_user_profile method
        let result = service.update_user_profile(user.id, UpdateUser::from_dto(empty_update_dto)).await;

        // Assert: Verify no updates error
        assert!(result.is_ok(), "Expected ok, got: {:?}", result);
        let updated_user = result.unwrap();
        user.updated_at = updated_user.updated_at;
        assert_eq!(updated_user, UserReadDto::from(user.clone()));


        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_user_profile_user_not_found() {
        // Arrange: Use a non-existent user ID
        let db = get_database().await;
        let service = UserService::new(&db);
        let non_existent_id = 999999; // Very unlikely to exist

        let update_dto = ProfileUpdateDto {
            first_name: Some("NotFound".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        // Act: Call the update_user_profile method
        let result = service.update_user_profile(non_existent_id, UpdateUser::from_dto(update_dto)).await;

        // Assert: Verify user not found error
        assert!(result.is_err(), "Should fail with user not found error");
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserNotFound) => { /* Success */ },
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_user_profile_with_all_genders() {
        // Arrange: Create a user in the database
        let user = create_test_user_for_update("service_update_genders").await;
        let db = get_database().await;
        let service = UserService::new(&db);

        // Test updating to each gender type
        let genders = vec![Gender::Male, Gender::Female, Gender::Other];
        
        for (i, gender) in genders.iter().enumerate() {
            let update_dto = ProfileUpdateDto {
                first_name: Some(format!("ServiceUser{}", i)),
                last_name: None,
                birthday: None,
                address: None,
                gender: Some(gender.clone()),
            };

            // Act: Update the user
            let result = service.update_user_profile(user.id, UpdateUser::from_dto(update_dto)).await;

            // Assert: Verify the update was successful
            assert!(result.is_ok(), "Gender update should succeed for {:?}", gender);
            let updated_user = result.unwrap();
            assert_eq!(updated_user.gender, *gender);
            assert_eq!(updated_user.first_name, format!("ServiceUser{}", i));
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_user_profile_birthday_validation() {
        // Arrange: Create a user in the database
        let user = create_test_user_for_update("service_update_birthday").await;
        let db = get_database().await;
        let service = UserService::new(&db);

        // Test with valid future birthday (should be allowed)
        let future_birthday = NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();
        let update_dto = ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: Some(future_birthday),
            address: None,
            gender: None,
        };

        // Act: Update with future birthday
        let result = service.update_user_profile(user.id, UpdateUser::from_dto(update_dto)).await;

        // Assert: Should succeed (no business rule against future birthdays in this system)
        assert!(result.is_ok(), "Future birthday should be allowed");
        let updated_user = result.unwrap();
        assert_eq!(updated_user.birthday, future_birthday);

        // Test with very old birthday
        let old_birthday = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let update_dto_old = ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: Some(old_birthday),
            address: None,
            gender: None,
        };

        let result_old = service.update_user_profile(user.id, UpdateUser::from_dto(update_dto_old)).await;
        assert!(result_old.is_ok(), "Old birthday should be allowed");
        let updated_user_old = result_old.unwrap();
        assert_eq!(updated_user_old.birthday, old_birthday);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_user_profile_preserves_critical_fields() {
        // Arrange: Create a user in the database
        let user = create_test_user_for_update("service_update_preserve").await;
        let db = get_database().await;
        let service = UserService::new(&db);

        // Save original critical fields
        let original_id = user.id;
        let original_username = user.username.clone();
        let original_email = user.email.clone();
        let original_password = user.password.clone();
        let original_user_type = user.user_type.clone();
        let original_user_status = user.user_status.clone();
        let original_created_at = user.created_at;

        let update_dto = ProfileUpdateDto {
            first_name: Some("PreserveTest".to_string()),
            last_name: Some("PreserveLast".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(2000, 6, 15).unwrap()),
            address: Some("Preserve Address".to_string()),
            gender: Some(Gender::Female),
        };

        // Act: Update profile fields
        let result = service.update_user_profile(user.id, UpdateUser::from_dto(update_dto)).await;

        // Assert: Verify critical fields are preserved
        assert!(result.is_ok(), "Update should succeed");
        let updated_user = result.unwrap();

        // Check updated fields
        assert_eq!(updated_user.first_name, "PreserveTest");
        assert_eq!(updated_user.last_name, "PreserveLast");
        assert_eq!(updated_user.birthday, NaiveDate::from_ymd_opt(2000, 6, 15).unwrap());
        assert_eq!(updated_user.address, "Preserve Address");
        assert_eq!(updated_user.gender, Gender::Female);

        // Check preserved critical fields
        assert_eq!(updated_user.id, original_id);
        assert_eq!(updated_user.username, original_username);
        assert_eq!(updated_user.email, original_email);
        assert_eq!(updated_user.user_type, original_user_type);
        assert_eq!(updated_user.user_status, original_user_status);
        assert_eq!(updated_user.created_at, original_created_at);

        // Verify updated_at changed
        assert!(updated_user.updated_at > user.updated_at, "updated_at should be newer");

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_user_profile_string_length_validation() {
        // Arrange: Create a user in the database
        let user = create_test_user_for_update("service_update_length").await;
        let db = get_database().await;
        let service = UserService::new(&db);

        // Test with address that's too long (over 256 characters)
        let long_address = "a".repeat(257);
        let invalid_update_dto = ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: None,
            address: Some(long_address),
            gender: None,
        };

        // Act: Call the update_user_profile method
        let result = service.update_user_profile(user.id, UpdateUser::from_dto(invalid_update_dto)).await;

        // Assert: Verify validation error
        assert!(result.is_err(), "Should fail with validation error for long address");

        // Test with very long first name
        let long_first_name = "b".repeat(257);
        let invalid_name_dto = ProfileUpdateDto {
            first_name: Some(long_first_name),
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        let result_name = service.update_user_profile(user.id, UpdateUser::from_dto(invalid_name_dto)).await;
        assert!(result_name.is_err(), "Should fail with validation error for long first name");

        // Cleanup
        cleanup_user(user.email).await;
    }
}
