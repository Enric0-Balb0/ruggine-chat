use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::entity::user::{UpdateUser, User};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::dto::user_dto::UserUpdateDto;
use crate::common::{get_database, create_test_user, cleanup_user};

#[cfg(test)]
mod update_profile_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_update_profile_success_complete_update() {
        // Arrange: Create a test user and update DTO
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (user, _password) = create_test_user("update_complete").await;

        let update_dto = UserUpdateDto {
            first_name: Some("UpdatedFirst".to_string()),
            last_name: Some("UpdatedLast".to_string()),
            birthday: Some(chrono::NaiveDate::from_ymd_opt(1985, 6, 15).unwrap()),
            address: Some("Updated Address 123".to_string()),
            gender: Some(ruggine_server::entity::user::Gender::Female),
        };

        let update_user = UpdateUser::from_dto(update_dto.clone());

        // Act: Update the user profile
        let result = repository.update_profile(user.id, update_user).await;

        // Assert: Verify the update was successful
        assert!(result.is_ok(), "Failed to update user profile: {:?}", result);
        let updated_user = result.unwrap();

        assert_eq!(updated_user.id, user.id);
        assert_eq!(updated_user.first_name, update_dto.first_name.unwrap());
        assert_eq!(updated_user.last_name, update_dto.last_name.unwrap());
        assert_eq!(updated_user.birthday, update_dto.birthday.unwrap());
        assert_eq!(updated_user.address, update_dto.address.unwrap());
        assert_eq!(updated_user.gender, update_dto.gender.unwrap());
        assert!(updated_user.updated_at > user.updated_at);

        // Verify unchanged fields remain the same
        assert_eq!(updated_user.email, user.email);
        assert_eq!(updated_user.username, user.username);
        assert_eq!(updated_user.user_status, user.user_status);
        assert_eq!(updated_user.user_type, user.user_type);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_profile_success_partial_update() {
        // Arrange: Create a test user and partial update DTO
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (mut user, _password) = create_test_user("update_partial").await;

        let update_dto = UserUpdateDto {
            first_name: Some("PartiallyUpdated".to_string()),
            last_name: None,
            birthday: None,
            address: Some("New Address Only".to_string()),
            gender: None,
        };

        let update_user = UpdateUser::from_dto(update_dto.clone());

        // Act: Update the user profile
        let result = repository.update_profile(user.id, update_user).await;

        // Assert: Verify the partial update was successful
        assert!(result.is_ok(), "Failed to update user profile: {:?}", result);
        let updated_user = result.unwrap();

        // Update the old user
        user.first_name = "PartiallyUpdated".to_string();
        user.address = "New Address Only".to_string();
        user.updated_at = updated_user.updated_at;

        // Check updated fields with user
        assert_eq!(updated_user, user);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_profile_empty_update() {
        // Arrange: Create a test user and empty update DTO
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (mut user, _password) = create_test_user("update_empty").await;

        let update_dto = UserFactory::fake_user_update_dto_empty();
        let update_user = UpdateUser::from_dto(update_dto);

        // Act: Update the user profile
        let result = repository.update_profile(user.id, update_user).await;

        // Assert: Verify all fields remain the same except updated_at
        assert!(result.is_ok(), "Failed to update user profile: {:?}", result);
        let updated_user = result.unwrap();

        // Update old user and check they are equals
        user.updated_at = updated_user.updated_at;

        assert_eq!(updated_user, user);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_profile_nonexistent_user() {
        // Arrange: Use a non-existent user ID
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let nonexistent_user_id = 999999;

        let update_dto = UserFactory::fake_user_update_dto();
        let update_user = UpdateUser::from_dto(update_dto);

        // Act: Try to update a non-existent user
        let result = repository.update_profile(nonexistent_user_id, update_user).await;

        // Assert: Should fail because user doesn't exist
        assert!(result.is_err(), "Update should fail for non-existent user");
    }

    #[tokio::test]
    async fn test_update_profile_using_factory_dto() {
        // Arrange: Create a test user and use factory DTO
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (user, _password) = create_test_user("update_factory").await;

        let update_dto = UserFactory::unique_fake_user_update_dto("factory_test");
        let update_user = UpdateUser::from_dto(update_dto.clone());

        // Act: Update the user profile
        let result = repository.update_profile(user.id, update_user).await;

        // Assert: Verify the update was successful
        assert!(result.is_ok(), "Failed to update user profile: {:?}", result);
        let updated_user = result.unwrap();

        assert_eq!(updated_user.first_name, update_dto.first_name.unwrap());
        assert_eq!(updated_user.last_name, update_dto.last_name.unwrap());
        assert_eq!(updated_user.birthday, update_dto.birthday.unwrap());
        assert_eq!(updated_user.address, update_dto.address.unwrap());
        assert_eq!(updated_user.gender, update_dto.gender.unwrap());

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_update_profile_only_gender() {
        // Arrange: Create a test user and update only gender
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (mut user, _password) = create_test_user("update_gender").await;

        let update_dto = UserUpdateDto {
            first_name: None,
            last_name: None,
            birthday: None,
            address: None,
            gender: Some(ruggine_server::entity::user::Gender::Other),
        };

        let update_user = UpdateUser::from_dto(update_dto.clone());

        // Act: Update the user profile
        let result = repository.update_profile(user.id, update_user).await;

        // Assert: Verify only gender was updated
        assert!(result.is_ok(), "Failed to update user profile: {:?}", result);
        let updated_user = result.unwrap();

        // Update old user and then check they are equals
        user.updated_at = updated_user.updated_at;
        user.gender = updated_user.gender.clone();
        assert_eq!(user, updated_user);

        // Cleanup
        cleanup_user(user.email).await;
    }
}
