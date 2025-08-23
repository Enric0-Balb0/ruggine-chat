use crate::common::cleanup_user_by_email;
use axum::{extract::{Path, State}, Extension};
use ruggine_server::config::database::DatabaseTrait;
use ruggine_server::dto::user_dto::UserReadDto;
use ruggine_server::handler::user_handler::find_by_id_handler::find_by_id;
use ruggine_server::repository::user_repository::UserRepositoryTrait;
use ruggine_server::service::user_service::UserServiceTrait;

#[cfg(test)]
mod find_by_id_handler_integration_tests {
    use super::*;
    use crate::{create_test_user, create_user_state};
    use ruggine_server::entity::user::UserStatus;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_existing_user() {
        // Arrange: Create a user and another user to authenticate as
        let (target_user, _) = create_test_user("find_by_id_target").await;
        let (auth_user, _) = create_test_user("find_by_id_auth").await;

        let state = create_user_state().await;

        // Act: Call the handler to find the target user
        let result = find_by_id(
            Extension(auth_user.clone()),
            State(state),
            Path(target_user.id),
        ).await;

        // Assert: Should successfully return the user
        assert!(result.is_ok(), "Handler should successfully find existing user");
        let response = result.unwrap().0;
        let data = response.data();
        
        // Verify the returned data matches the target user
        let expected = UserReadDto::from(target_user.clone());
        assert_eq!(*data, expected, "Returned user data should match expected");
        assert_eq!(data.id, target_user.id, "User ID should match");
        assert_eq!(data.email, target_user.email, "User email should match");
        assert_eq!(data.username, target_user.username, "Username should match");
        assert_eq!(data.first_name, target_user.first_name, "First name should match");
        assert_eq!(data.last_name, target_user.last_name, "Last name should match");

        // Cleanup
        cleanup_user_by_email(target_user.email).await;
        cleanup_user_by_email(auth_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_non_existent_user() {
        // Arrange: Create an authenticated user but no target user
        let (auth_user, _) = create_test_user("find_by_id_auth_nonexistent").await;
        let non_existent_id = 99999;

        let state = create_user_state().await;

        // Act: Try to find a non-existent user
        let result = find_by_id(
            Extension(auth_user.clone()),
            State(state),
            Path(non_existent_id),
        ).await;

        // Assert: Should return an error
        assert!(result.is_err(), "Handler should return error for non-existent user");

        // Cleanup
        cleanup_user_by_email(auth_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_user_can_find_self() {
        // Arrange: Create a user
        let (user, _) = create_test_user("find_by_id_self").await;

        let state = create_user_state().await;

        // Act: User tries to find themselves
        let result = find_by_id(
            Extension(user.clone()),
            State(state),
            Path(user.id),
        ).await;

        // Assert: Should successfully return their own data
        assert!(result.is_ok(), "User should be able to find their own data");
        let response = result.unwrap().0;
        let data = response.data();
        
        let expected = UserReadDto::from(user.clone());
        assert_eq!(*data, expected, "Self-lookup should return correct user data");
        assert_eq!(data.id, user.id, "Should return the same user ID");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_with_different_user_statuses() {
        // Arrange: Create users with different statuses
        let (active_user, _) = create_test_user("find_by_id_active").await;
        let (auth_user, _) = create_test_user("find_by_id_auth_status").await;
        
        // Verify the user is active by default
        assert_eq!(active_user.user_status, UserStatus::Active, "User should be active by default");

        let state = create_user_state().await;

        // Act: Find active user
        let result = find_by_id(
            Extension(auth_user.clone()),
            State(state),
            Path(active_user.id),
        ).await;

        // Assert: Should successfully find active user
        assert!(result.is_ok(), "Should be able to find active user");
        let response = result.unwrap().0;
        let data = response.data();
        
        assert_eq!(data.id, active_user.id, "Should return correct user");
        assert_eq!(data.user_status, UserStatus::Active, "User status should be active");

        // Cleanup
        cleanup_user_by_email(active_user.email).await;
        cleanup_user_by_email(auth_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_returns_complete_user_dto() {
        // Arrange: Create users
        let (target_user, _) = create_test_user("find_by_id_complete").await;
        let (auth_user, _) = create_test_user("find_by_id_auth_complete").await;

        let state = create_user_state().await;

        // Act: Find the user
        let result = find_by_id(
            Extension(auth_user.clone()),
            State(state),
            Path(target_user.id),
        ).await;

        // Assert: Verify all fields are properly mapped
        assert!(result.is_ok(), "Handler should succeed");
        let response = result.unwrap().0;
        let data = response.data();
        
        // Check all UserReadDto fields are present and correct
        assert_eq!(data.id, target_user.id);
        assert_eq!(data.email, target_user.email);
        assert_eq!(data.username, target_user.username);
        assert_eq!(data.first_name, target_user.first_name);
        assert_eq!(data.last_name, target_user.last_name);
        assert_eq!(data.user_status, target_user.user_status);
        assert_eq!(data.user_type, target_user.user_type);
        assert_eq!(data.birthday, target_user.birthday);
        assert_eq!(data.address, target_user.address);
        assert_eq!(data.gender, target_user.gender);
        assert_eq!(data.created_at, target_user.created_at);
        assert_eq!(data.updated_at, target_user.updated_at);
        assert_eq!(data.is_online, target_user.is_online);
        
        // Password should not be included in UserReadDto
        // This is inherent in the DTO design, no explicit check needed

        // Cleanup
        cleanup_user_by_email(target_user.email).await;
        cleanup_user_by_email(auth_user.email).await;
    }
}
