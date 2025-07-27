use ruggine_server::handler::user_handler::profile_handler::profile;
use ruggine_server::dto::user_dto::UserReadDto;
use ruggine_server::entity::user::User;
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::config::database::DatabaseTrait;
use axum::Extension;
use crate::common::cleanup_user;

#[cfg(test)]
mod profile_handler_integration_tests {
    use ruggine_server::entity::user::{CurrentAction, UserStatus};
    use crate::get_database;
    use super::*;

    /// Helper function to create a real user in the database
    async fn create_test_user(prefix: &str) -> (User, String) {
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);
        
        let user_dto = UserFactory::unique_fake_user_register_dto(prefix);
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for profile test");
        
        // Get the created user from database
        let user_option = repository.find_by_email(user_dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        (user_option.unwrap(), user_dto.password.clone())
    }

    #[tokio::test]
    async fn test_profile_returns_correct_user_data() {
        // Arrange
        let (user, _) = create_test_user("profile_correct_data").await;

        // Act
        let response = profile(Extension(user.clone())).await;
        let data = response.0.data();

        // Assert: confronta direttamente con UserReadDto generato
        let expected = UserReadDto::from(user.clone());
        assert_eq!(*data, expected, "UserReadDto should match expected user data");

        // Extra check
        assert!(!data.is_online, "User should not be online by default");
        assert_eq!(data.current_action, CurrentAction::Waiting, "User should be in Waiting state by default");

        // Cleanup
        cleanup_user(user.email).await;
    }


    #[tokio::test]
    async fn test_profile_with_active_user() {
        // Arrange: Create an active user
        let (user, _) = create_test_user("profile_active_user").await;
        assert_eq!(user.user_status, UserStatus::Active, "User should be active by default");
        
        // Act: Call profile handler
        let response = profile(Extension(user.clone())).await;
        let data = response.0.data();

        // Assert: Compare the entire user response (excluding password)
        let expected = UserReadDto::from(user.clone());
        assert_eq!(data, &expected);

        // Extra check: user should not be online and should be waiting
        assert!(!data.is_online, "User should not be online by default");
        assert_eq!(data.current_action, CurrentAction::Waiting);
        
        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_profile_with_inactive_user() {
        // Arrange: create and deactivate user
        let (user, _) = create_test_user("profile_inactive_user").await;

        let db = get_database().await;
        let pool = db.get_pool();
        sqlx::query("UPDATE \"user\" SET user_status = $1 WHERE email = $2")
            .bind(UserStatus::Deleted)
            .bind(&user.email)
            .execute(pool)
            .await
            .expect("Failed to deactivate user");

        let repo = UserRepository::new(&db);
        let updated_user = repo
            .find_by_email(user.email.clone())
            .await
            .expect("User should still exist in database");

        // Act
        let response = profile(Extension(updated_user.clone())).await;
        let data = response.0.data();

        // Assert
        let expected = UserReadDto::from(updated_user.clone());
        assert_eq!(*data, expected, "Returned user data should match updated user");
        assert_eq!(data.user_status, UserStatus::Deleted, "User status should reflect deletion");

        assert!(!data.is_online, "User should not be online by default");
        assert_eq!(data.current_action, CurrentAction::Waiting, "Default action should be Waiting");

        // Cleanup
        cleanup_user(updated_user.email).await;
    }


    #[tokio::test]
    async fn test_profile_preserves_all_user_fields() {
        // Arrange: Create a user with specific data
        let (user, _) = create_test_user("profile_all_fields").await;
        
        // Act: Call profile handler
        let response = profile(Extension(user.clone())).await;
        
        // Assert: Verify all user fields are preserved (except password)
        let data = response.0.data();
        
        assert_eq!(data, &UserReadDto::from(user.clone()));
        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_profile_response_structure() {
        // Arrange: Create a real user
        let (user, _) = create_test_user("profile_response_structure").await;
        
        // Act: Call profile handler
        let response = profile(Extension(user.clone())).await;
        
        // Assert: Verify response structure is correct
        let json_response = response.0;
        let data = json_response.data();
        
        // Verify that the response contains valid UserReadDto fields
        assert!(data.id > 0, "ID should be positive");
        assert!(!data.first_name.is_empty(), "First name should not be empty");
        assert!(!data.last_name.is_empty(), "Last name should not be empty");
        assert!(!data.username.is_empty(), "Username should not be empty");
        assert!(data.email.contains('@'), "Email should be valid format");
        assert!(data.created_at <= chrono::Utc::now(), "Created date should not be in future");
        assert!(!data.address.is_empty(), "Address should not be empty");
        
        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_profile_with_updated_user() {
        // Arrange: Create user and update some fields
        let (user, _) = create_test_user("profile_updated_user").await;
        
        // Update user fields directly in database
        let db = get_database().await;
        let pool = db.get_pool();
        let new_first_name = "UpdatedFirstName";
        let new_last_name = "UpdatedLastName";
        let old_now = chrono::Utc::now();
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        let new_now = chrono::Utc::now(); // Questo è ciò che scriverai nel DB
        
        let update_result = sqlx::query(
            r#"UPDATE "user" SET first_name = $1, last_name = $2, updated_at = $3 WHERE email = $4"#
        )
        .bind(new_first_name)
        .bind(new_last_name)
        .bind(new_now.clone())
        .bind(&user.email)
        .execute(pool)
        .await;
        assert!(update_result.is_ok(), "Failed to update user");
        
        // Get the updated user from database
        let repository = UserRepository::new(&db);
        let updated_user = repository.find_by_email(user.email.clone()).await
            .expect("User should still exist in database");
        
        // Act: Call profile handler with updated user
        let response = profile(Extension(updated_user.clone())).await;
        
        // Assert: Verify updated data is returned
        let data = response.0.data();
        assert_eq!(data.first_name, new_first_name);
        assert_eq!(data.last_name, new_last_name);
        assert_eq!(data.email, updated_user.email);

        assert!(
            data.updated_at > old_now,
            "Expected updated_at to be after old_now. updated_at = {}, old_now = {}",
            data.updated_at,
            old_now
        );
        
        // Cleanup
        cleanup_user(updated_user.email).await;
    }

    #[tokio::test]
    async fn test_profile_with_different_user_types() {
        // Arrange: Create multiple users with different characteristics
        let (user1, _) = create_test_user("profile_type1").await;
        let (user2, _) = create_test_user("profile_type2").await;
        
        // Act: Call profile handler for both users
        let response1 = profile(Extension(user1.clone())).await;
        let response2 = profile(Extension(user2.clone())).await;
        
        // Assert: Verify each user gets their own data
        let data1 = response1.0.data();
        let data2 = response2.0.data();
        
        assert_ne!(data1.id, data2.id, "Users should have different IDs");
        assert_ne!(data1.email, data2.email, "Users should have different emails");
        assert_ne!(data1.username, data2.username, "Users should have different usernames");
        
        // Both should be valid responses
        assert!(data1.id > 0);
        assert!(data2.id > 0);
        assert!(data1.email.contains('@'));
        assert!(data2.email.contains('@'));
        
        // Both should have default values for auto-generated fields
        assert!(!data1.is_online);
        assert!(!data2.is_online);
        assert_eq!(data1.current_action, ruggine_server::entity::user::CurrentAction::Waiting);
        assert_eq!(data2.current_action, ruggine_server::entity::user::CurrentAction::Waiting);
        
        // Cleanup
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
    }
}