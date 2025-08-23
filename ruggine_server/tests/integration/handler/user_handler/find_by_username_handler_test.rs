use axum::{
    extract::{Path, State}
    ,
    Extension,
};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::handler::user_handler::find_by_username_handler::find_by_username;
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::state::user_state::UserState;
use tower::ServiceExt;

#[cfg(test)]
mod find_by_username_handler_integration_tests {
    use super::*;
    use crate::common::{cleanup_user_by_email, get_database};
    use crate::create_test_user;
    use ruggine_server::entity::user::UserStatus;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_handler_success() {
        // Arrange
        let (target_user, _) = create_test_user("handler_success").await;
        let test_username = target_user.username.clone();
        let test_email = target_user.email.clone();
        let current_user = UserFactory::fake_user(); // Authenticated user
        
        let db = get_database().await;
        let state = UserState::new(&db);

        // Act
        let result = find_by_username(
            Extension(current_user),
            State(state),
            Path(test_username.clone()),
        ).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap().0;
        let found_user = response.data().clone().unwrap();
        
        assert_eq!(found_user.username, test_username);
        assert_eq!(found_user.email, target_user.email);
        assert_eq!(found_user.first_name, target_user.first_name);
        assert_eq!(found_user.last_name, target_user.last_name);

        // Cleanup
        cleanup_user_by_email(test_email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_handler_not_found() {
        // Arrange
        let current_user = UserFactory::fake_user();
        let non_existent_username = UserFactory::fake_username("nonexistent_handler");
        
        let db = get_database().await;
        let state = UserState::new(&db);

        // Act
        let result = find_by_username(
            Extension(current_user),
            State(state),
            Path(non_existent_username),
        ).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.0.data().is_none());
    }


    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_handler_case_sensitivity() {
        // Arrange
        let (target_user, _) = create_test_user("handler_case_sensitivity").await;
        let test_username = target_user.username.clone();
        let test_email = target_user.email.clone();
        let current_user = UserFactory::fake_user();
        
        let db = get_database().await;
        let state = UserState::new(&db);

        // Act - Test with uppercase username
        let uppercase_username = test_username.to_uppercase();
        let result = find_by_username(
            Extension(current_user),
            State(state),
            Path(uppercase_username),
        ).await;

        // Assert - Should not find user (case sensitive)
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.0.data().is_none(), "Username search should be case sensitive");

        // Cleanup
        cleanup_user_by_email(test_email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_handler_multiple_users() {
        // Arrange - Create multiple users
        let (user1, _) = create_test_user("handler_multiple1").await;
        let username1 = user1.username.clone();
        let email1 = user1.email.clone();
        let (user2, _) = create_test_user("handler_multiple2").await;
        let username2 = user2.username.clone();
        let email2 = user2.email.clone();
        let current_user = UserFactory::fake_user();
        
        let db = get_database().await;
        let state = UserState::new(&db);

        // Act & Assert - Find first user
        let result1 = find_by_username(
            Extension(current_user.clone()),
            State(state.clone()),
            Path(username1.clone()),
        ).await;

        assert!(result1.is_ok());
        let response1 = result1.unwrap();
        let found_user1 = response1.0.data().clone().unwrap();
        assert_eq!(found_user1.username, username1);
        assert_eq!(found_user1.email, user1.email);

        // Act & Assert - Find second user
        let result2 = find_by_username(
            Extension(current_user),
            State(state),
            Path(username2.clone()),
        ).await;

        assert!(result2.is_ok());
        let response2 = result2.unwrap();
        let found_user2 = response2.0.data().clone().unwrap();
        assert_eq!(found_user2.username, username2);
        assert_eq!(found_user2.email, user2.email);

        // Verify users are different
        assert_ne!(found_user1.id, found_user2.id);
        assert_ne!(found_user1.email, found_user2.email);

        // Cleanup
        cleanup_user_by_email(email1).await;
        cleanup_user_by_email(email2).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_handler_returns_complete_dto() {
        // Arrange
        let (target_user, _) = create_test_user("handler_complete").await;
        let test_username = target_user.username.clone();
        let test_email = target_user.email.clone();
        let current_user = UserFactory::fake_user();
        
        let db = get_database().await;
        let state = UserState::new(&db);

        // Act
        let result = find_by_username(
            Extension(current_user),
            State(state),
            Path(test_username.clone()),
        ).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let user_dto = response.0.data().clone().unwrap();

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

        // Password should not be included in UserReadDto
        // This is inherent in the DTO design, no explicit check needed

        // Cleanup
        cleanup_user_by_email(test_email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_handler_with_special_characters() {
        // Arrange - Create user with special characters in username
        let db = get_database().await;
        let service = UserService::new(&db);

        let dto = UserFactory::unique_fake_user_register_dto("handler_special");
        let special_username = format!("{}_test", dto.username);
        let email = dto.email.clone();
        
        let mut special_dto = dto;
        special_dto.username = special_username.clone();
        
        let create_result = service.create_user(special_dto).await;
        assert!(create_result.is_ok(), "Failed to create user with special characters");

        let current_user = UserFactory::fake_user();
        let state = UserState::new(&db);

        // Act
        let result = find_by_username(
            Extension(current_user),
            State(state),
            Path(special_username.clone()),
        ).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        let user_dto = response.0.data().clone().unwrap();
        assert_eq!(user_dto.username, special_username);

        // Cleanup
        cleanup_user_by_email(email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_handler_response_structure() {
        // Arrange
        let (target_user, _) = create_test_user("handler_structure").await;
        let test_username = target_user.username.clone();
        let test_email = target_user.email.clone();
        let current_user = UserFactory::fake_user();
        
        let db = get_database().await;
        let state = UserState::new(&db);

        // Act
        let result = find_by_username(
            Extension(current_user),
            State(state),
            Path(test_username.clone()),
        ).await;

        // Assert response structure
        assert!(result.is_ok());
        let response = result.unwrap();
        
        // Verify it's wrapped in Json
        let api_response = response.0;
        
        // Verify the data is present
        assert!(api_response.data().is_some());
        
        let user_dto = api_response.data().clone().unwrap();
        assert_eq!(user_dto.username, test_username);

        // Cleanup
        cleanup_user_by_email(test_email).await;
    }
}
