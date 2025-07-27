use ruggine_server::handler::auth_handler::login_handler::login;
use ruggine_server::state::auth_state::AuthState;
use ruggine_server::service::token_service::TokenService;
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::dto::user_dto::UserLoginDto;
use ruggine_server::error::{api_error::ApiError, user_error::UserError, request_error::ValidatedRequest};
use ruggine_server::entity::user::User;
use ruggine_server::config::database::DatabaseTrait;
use axum::extract::State;
use std::sync::Arc;
use crate::common::cleanup_user;
use ruggine_server::factory::token_factory::TokenFactory;

#[cfg(test)]
mod login_handler_integration_tests {
    use ruggine_server::entity::user::UserStatus;

    use crate::get_database;
    use super::*;

    /// Helper function to create a real auth state with database connections
    async fn create_auth_state(jwt_secret: &str) -> AuthState {
        let db = get_database().await;
        let user_repo = Arc::new(UserRepository::new(&db));
        let user_service = Arc::new(UserService::new(&db));
        let token_service = Arc::new(TokenService::new(jwt_secret.to_string()));
        
        AuthState {
            user_repo,
            user_service,
            token_service,
        }
    }

    /// Helper function to create a real user in the database
    async fn create_test_user(prefix: &str) -> (User, String) {
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);
        
        let user_dto = UserFactory::unique_fake_user_register_dto(prefix);
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for login test");
        
        // Get the created user from database
        let user_option = repository.find_by_email(user_dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        (user_option.unwrap(), user_dto.password.clone())
    }


    #[tokio::test]
    async fn test_login_success_with_real_user() {
        // Arrange: Create a real user and auth state
        let jwt_secret = TokenFactory::get_unique_jwt_secret("login_success");
        let auth_state = create_auth_state(&jwt_secret).await;
        let (user, original_password) = create_test_user("login_success").await;
        
        // Create login request with the original password (not hashed)
        let login_dto = UserLoginDto {
            email: user.email.clone(),
            password: original_password,
        };

        // Act: Call login handler
        let result = login(
            State(auth_state),
            ValidatedRequest(login_dto),
        ).await;

        // Assert: Should succeed and return a valid token
        assert!(result.is_ok(), "Login should succeed with valid credentials");
        let token_response = result.unwrap().0;
        
        assert!(!token_response.data().token.is_empty(), "Token should not be empty");
        assert!(token_response.data().iat > 0, "Token should have a valid issued-at timestamp");
        assert!(token_response.data().exp > token_response.data().iat, "Token should expire after it was issued");
        assert!(token_response.data().exp > chrono::Utc::now().timestamp(), "Token should not be expired");

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        // Arrange: Create auth state but no user in database
        let jwt_secret = TokenFactory::get_unique_jwt_secret("user_not_found");
        let auth_state = create_auth_state(&jwt_secret).await;
        
        let login_dto = UserLoginDto {
            email: "nonexistent@example.com".to_string(),
            password: "any_password".to_string(),
        };

        // Act: Try to log in with non-existent user
        let result = login(
            State(auth_state),
            ValidatedRequest(login_dto),
        ).await;

        // Assert: Should fail with UserNotFound error
        assert!(result.is_err(), "Login should fail when user doesn't exist");
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserNotFound) => {
                // Expected error
            }
            other => panic!("Expected UserError::UserNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_login_user_not_active() {
        // Arrange: Create user and then deactivate them
        let jwt_secret = TokenFactory::get_unique_jwt_secret("user_not_active");
        let auth_state = create_auth_state(&jwt_secret).await;
        let (user, _) = create_test_user("user_not_active").await;
        
        // Deactivate the user using direct database access
        let db = get_database().await;
        let pool = db.get_pool();
        let update_result = sqlx::query("UPDATE \"user\" SET user_status = $1 WHERE email = $2")
            .bind(UserStatus::Deleted)
            .bind(&user.email)
            .execute(pool)
            .await;
        assert!(update_result.is_ok(), "Failed to deactivate user");
        
        let login_dto = UserLoginDto {
            email: user.email.clone(),
            password: "test password123".to_string(),
        };

        // Act: Try to log in with inactive user
        let result = login(
            State(auth_state),
            ValidatedRequest(login_dto),
        ).await;

        // Assert: Should fail with UserNotActive error
        assert!(result.is_err(), "Login should fail for inactive user");
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserNotActive) => {
                // Expected error
            }
            other => panic!("Expected UserError::UserNotActive, got {:?}", other),
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        // Arrange: Create a real user with known password
        let jwt_secret = TokenFactory::get_unique_jwt_secret("invalid_password");
        let auth_state = create_auth_state(&jwt_secret).await;
        let (user, _) = create_test_user("invalid_password").await;
        
        let login_dto = UserLoginDto {
            email: user.email.clone(),
            password: "wrong_password".to_string(), // Incorrect password
        };

        // Act: Try to log in with wrong password
        let result = login(
            State(auth_state),
            ValidatedRequest(login_dto),
        ).await;

        // Assert: Should fail with InvalidPassword error
        assert!(result.is_err(), "Login should fail with wrong password");
        match result.unwrap_err() {
            ApiError::UserError(UserError::InvalidPassword) => {
                // Expected error
            }
            other => panic!("Expected UserError::InvalidPassword, got {:?}", other),
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_login_empty_email() {
        // Arrange: Create auth state
        let jwt_secret = TokenFactory::get_unique_jwt_secret("empty_email");
        let auth_state = create_auth_state(&jwt_secret).await;
        
        let login_dto = UserLoginDto {
            email: "".to_string(), // Empty email
            password: "any_password".to_string(),
        };

        // Act: Try to log in with empty email
        let result = login(
            State(auth_state),
            ValidatedRequest(login_dto),
        ).await;

        // Assert: Should fail with UserNotFound error
        assert!(result.is_err(), "Login should fail with empty email");
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserNotFound) => {
                // Expected error
            }
            other => panic!("Expected UserError::UserNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_login_empty_password() {
        // Arrange: Create a real user
        let jwt_secret = TokenFactory::get_unique_jwt_secret("empty_password");
        let auth_state = create_auth_state(&jwt_secret).await;
        let (user, _) = create_test_user("empty_password").await;
        
        let login_dto = UserLoginDto {
            email: user.email.clone(),
            password: "".to_string(), // Empty password
        };

        // Act: Try to log in with empty password
        let result = login(
            State(auth_state),
            ValidatedRequest(login_dto),
        ).await;

        // Assert: Should fail with InvalidPassword error
        assert!(result.is_err(), "Login should fail with empty password");
        match result.unwrap_err() {
            ApiError::UserError(UserError::InvalidPassword) => {
                // Expected error
            }
            other => panic!("Expected UserError::InvalidPassword, got {:?}", other),
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_login_case_sensitive_email() {
        // Arrange: Create a real user with lowercase email
        let jwt_secret = TokenFactory::get_unique_jwt_secret("case_sensitive");
        let auth_state = create_auth_state(&jwt_secret).await;
        let (user, password) = create_test_user("case_sensitive").await;
        
        // Try to log in with uppercase version of email
        let login_dto = UserLoginDto {
            email: user.email.to_uppercase(), // Different case
            password,
        };

        // Act: Try to log in with different case email
        let result = login(
            State(auth_state),
            ValidatedRequest(login_dto),
        ).await;

        // Assert: Should be ok (assuming case-sensitive email lookup)
        assert!(result.is_err(), "Login should not pass with different case email");

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_login_multiple_users_with_different_credentials() {
        // Arrange: Create multiple users with different credentials
        let jwt_secret = TokenFactory::get_unique_jwt_secret("multiple_users");
        let auth_state = create_auth_state(&jwt_secret).await;
        
        let (user1, password1) = create_test_user("multi_user1").await;
        let (user2, password2) = create_test_user("multi_user2").await;
        
        // Test login for first user
        let login_dto1 = UserLoginDto {
            email: user1.email.clone(),
            password: password1.clone(),
        };

        let result1 = login(
            State(auth_state.clone()),
            ValidatedRequest(login_dto1),
        ).await;

        // Test login for second user
        let login_dto2 = UserLoginDto {
            email: user2.email.clone(),
            password: password2.clone(),
        };

        let result2 = login(
            State(auth_state),
            ValidatedRequest(login_dto2),
        ).await;

        // Assert: Both logins should succeed but return different tokens
        assert!(result1.is_ok(), "First user login should succeed");
        assert!(result2.is_ok(), "Second user login should succeed");
        
        let token1 = result1.unwrap().0;
        let token2 = result2.unwrap().0;

        assert_ne!(token1.data().token, token2.data().token, "Different users should get different tokens");

        // Cleanup
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
    }
}
