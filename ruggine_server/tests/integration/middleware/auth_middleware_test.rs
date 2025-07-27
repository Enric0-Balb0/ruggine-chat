use ruggine_server::middleware::auth_middleware::{auth_inner};
use ruggine_server::entity::user::all_user_types;
use ruggine_server::state::token_state::TokenState;
use ruggine_server::service::token_service::{TokenService, TokenServiceTrait};
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::factory::token_factory::TokenFactory;
use ruggine_server::error::{api_error::ApiError, token_error::TokenError, user_error::UserError};
use ruggine_server::entity::user::{User, UserType};
use axum::{
    body::Body,
    http::{Request, header},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use ruggine_server::config::database::DatabaseTrait;
use crate::common::cleanup_user;

#[cfg(test)]
mod auth_middleware_integration_tests {
    use ruggine_server::entity::user::UserStatus;

    use crate::get_database;
    use super::*;

    /// Helper function to create a real token state with database connections
    async fn create_token_state(jwt_secret: &str) -> TokenState {
        let db = get_database().await;
        let token_service = Arc::new(TokenService::new(jwt_secret.to_string()));
        let user_repo = Arc::new(UserRepository::new(&db));

        TokenState {
            token_service,
            user_repo,
        }
    }

    /// Helper function to create a real user and generate a valid token
    async fn create_user_and_token(prefix: &str) -> (User, String, TokenState) {
        let jwt_secret = TokenFactory::get_unique_jwt_secret(prefix);
        let state = create_token_state(&jwt_secret).await;

        // Create a real user in the database
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let user_dto = UserFactory::unique_fake_user_register_dto(prefix);
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for auth test");

        // Get the created user from database
        let user_option = repository.find_by_email(user_dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        // Generate a valid token for this user
        let token_result = state.token_service.generate_token(user.clone());
        assert!(token_result.is_ok(), "Failed to generate token");
        let token_data = token_result.unwrap();

        (user, token_data.token, state)
    }

    #[tokio::test]
    async fn test_auth_inner_success_with_real_user_and_token() {
        // Arrange: Create a real user and valid token
        let (user, token, state) = create_user_and_token("auth_success").await;

        let req = Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner with real token and user
        let result = auth_inner(&state, req, all_user_types()).await;

        // Assert: Should succeed and inject user into request
        assert!(result.is_ok(), "Auth should succeed with valid token and user");
        let req_with_user = result.unwrap();

        let injected_user = req_with_user.extensions().get::<User>();
        assert!(injected_user.is_some(), "User should be injected into request");
        assert_eq!(injected_user.unwrap().id, user.id, "Injected user should match original user");
        assert_eq!(injected_user.unwrap().email, user.email, "Injected user email should match");

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_auth_inner_missing_authorization_header() {
        // Arrange: Create token state but no authorization header
        let jwt_secret = TokenFactory::get_unique_jwt_secret("missing_header");
        let state = create_token_state(&jwt_secret).await;

        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner without authorization header
        let result = auth_inner(&state, req, all_user_types()).await;

        // Assert: Should fail with MissingToken error
        assert!(result.is_err(), "Auth should fail without authorization header");
        match result.unwrap_err() {
            ApiError::TokenError(TokenError::MissingToken) => {
                // Expected error
            }
            other => panic!("Expected TokenError::MissingToken, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_inner_invalid_authorization_header_format() {
        // Arrange: Create token state with malformed authorization header
        let jwt_secret = TokenFactory::get_unique_jwt_secret("invalid_header");
        let state = create_token_state(&jwt_secret).await;

        let req = Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, "InvalidHeaderFormat")
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner with invalid header format
        let result = auth_inner(&state, req, all_user_types()).await;

        // Assert: Should fail with MissingToken error
        assert!(result.is_err(), "Auth should fail with invalid header format");
        match result.unwrap_err() {
            ApiError::TokenError(TokenError::MissingToken) => {
                // Expected error
            }
            other => panic!("Expected TokenError::MissingToken, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_inner_invalid_token_wrong_secret() {
        // Arrange: Create a token with one secret, try to verify with another
        let (user, token, _) = create_user_and_token("wrong_secret").await;

        // Create a different token state with different JWT secret
        let different_jwt_secret = TokenFactory::get_unique_jwt_secret("different_secret");
        let different_state = create_token_state(&different_jwt_secret).await;

        let req = Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Act: Try to verify token with wrong secret
        let result = auth_inner(&different_state, req, all_user_types()).await;

        // Assert: Should fail with InvalidToken error
        assert!(result.is_err(), "Auth should fail with token signed by different secret");
        match result.unwrap_err() {
            ApiError::TokenError(TokenError::InvalidToken(_)) => {
                // Expected error
            }
            other => panic!("Expected TokenError::InvalidToken, got {:?}", other),
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_auth_inner_user_not_found_deleted_user() {
        // Arrange: Create user and token, then delete the user
        let (user, token, state) = create_user_and_token("deleted_user").await;

        // Delete the user from database while keeping the valid token
        cleanup_user(user.email.clone()).await;

        let req = Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Act: Try to authenticate with token of deleted user
        let result = auth_inner(&state, req, all_user_types()).await;

        // Assert: Should fail with UserNotFound error
        assert!(result.is_err(), "Auth should fail when user is not found in database");
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserNotFound) => {
                // Expected error
            }
            other => panic!("Expected UserError::UserNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_inner_user_not_active() {
        // Setup

        let (user, token, state) = create_user_and_token("inactive_user").await;
        let db = get_database().await;
        let pool = db.get_pool();

        // Disattiva l’utente (senza cancellarlo)
        let update_result = sqlx::query("UPDATE \"user\" SET user_status = $1 WHERE email = $2")
            .bind(UserStatus::Deleted)
            .bind(&user.email)
            .execute(pool)
            .await;
        assert!(update_result.is_ok(), "Failed to deactivate user");

        let req = Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Act
        let result = auth_inner(&state, req, all_user_types()).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserNotActive) => {
                // Success
            }
            other => panic!("Expected UserNotActive, got {:?}", other),
        }

        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_auth_inner_malformed_bearer_token() {
        // Arrange: Create token state with malformed Bearer token
        let jwt_secret = TokenFactory::get_unique_jwt_secret("malformed_token");
        let state = create_token_state(&jwt_secret).await;

        let req = Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, "Bearer invalid.token.format")
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner with malformed token
        let result = auth_inner(&state, req, all_user_types()).await;

        // Assert: Should fail with InvalidToken error
        assert!(result.is_err(), "Auth should fail with malformed token");
        match result.unwrap_err() {
            ApiError::TokenError(TokenError::InvalidToken(_)) => {
                // Expected error
            }
            other => panic!("Expected TokenError::InvalidToken, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_inner_empty_bearer_token() {
        // Arrange: Create token state with empty Bearer token
        let jwt_secret = TokenFactory::get_unique_jwt_secret("empty_token");
        let state = create_token_state(&jwt_secret).await;

        let req = Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, "Bearer ")
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner with empty token
        let result = auth_inner(&state, req, all_user_types()).await;

        // Assert: Should fail with InvalidToken error
        assert!(result.is_err(), "Auth should fail with empty token");
        match result.unwrap_err() {
            ApiError::TokenError(TokenError::InvalidToken(_)) => {
                // Expected error
            }
            other => panic!("Expected TokenError::InvalidToken, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_inner_token_without_bearer_prefix() {
        // Arrange: Create token state with token that doesn't have Bearer prefix
        let jwt_secret = TokenFactory::get_unique_jwt_secret("no_bearer");
        let state = create_token_state(&jwt_secret).await;

        let req = Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, "somevalidtokenbutnobearer")
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner with token without Bearer prefix
        let result = auth_inner(&state, req, all_user_types()).await;

        // Assert: Should fail with MissingToken error
        assert!(result.is_err(), "Auth should fail with token without Bearer prefix");
        match result.unwrap_err() {
            ApiError::TokenError(TokenError::MissingToken) => {
                // Expected error
            }
            other => panic!("Expected TokenError::MissingToken, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_auth_inner_with_allowed_user_type_success() {
        // Arrange: Create a real user with Admin type and valid token
        let (mut user, token, state) = create_user_and_token("auth_admin_success").await;
        
        // Update user type to Admin
        let db = get_database().await;
        let pool = db.get_pool();
        sqlx::query(r#"UPDATE "user" SET user_type = $1 WHERE id = $2"#)
            .bind(UserType::Admin)
            .bind(user.id)
            .execute(pool)
            .await
            .expect("Failed to update user type");
        
        user.user_type = UserType::Admin; // Update local copy

        let req = Request::builder()
            .uri("/admin-protected")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner with Admin type allowed
        let allowed_types = vec![UserType::Admin];
        let result = auth_inner(&state, req, allowed_types).await;

        // Assert: Should succeed for Admin user
        assert!(result.is_ok(), "Auth should succeed for Admin user with Admin type allowed");
        let req_with_user = result.unwrap();
        
        let injected_user = req_with_user.extensions().get::<User>();
        assert!(injected_user.is_some(), "User should be injected into request");
        assert_eq!(injected_user.unwrap().user_type, UserType::Admin, "Injected user should be Admin");

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_auth_inner_with_disallowed_user_type_fails() {
        // Arrange: Create a real user with User type and valid token
        let (mut user, token, state) = create_user_and_token("auth_user_fail").await;
        
        // Ensure user type is User (should be default)
        let db = get_database().await;
        let pool = db.get_pool();
        sqlx::query(r#"UPDATE "user" SET user_type = $1 WHERE id = $2"#)
            .bind(UserType::EndUser)
            .bind(user.id)
            .execute(pool)
            .await
            .expect("Failed to update user type");
            
        user.user_type = UserType::EndUser; // Update local copy

        let req = Request::builder()
            .uri("/admin-protected")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner with only Admin type allowed
        let allowed_types = vec![UserType::Admin];
        let result = auth_inner(&state, req, allowed_types).await;

        // Assert: Should fail for User trying to access Admin-only resource
        assert!(result.is_err(), "Auth should fail for User trying to access Admin-only resource");
        match result.unwrap_err() {
            ApiError::UserError(UserError::InsufficientPermissions) => {
                // Expected error
            }
            other => panic!("Expected UserError::InsufficientPermissions, got {:?}", other),
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_auth_inner_with_multiple_allowed_user_types() {
        // Arrange: Create a real user with User type and valid token
        let (user, token, state) = create_user_and_token("auth_multi_types").await;

        let req = Request::builder()
            .uri("/user-or-admin-protected")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Act: Call auth_inner with both User and Admin types allowed
        let allowed_types = vec![UserType::Admin, UserType::EndUser];
        let result = auth_inner(&state, req, allowed_types).await;

        // Assert: Should succeed for User when both User and Admin are allowed
        assert!(result.is_ok(), "Auth should succeed for User when both User and Admin types are allowed");

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio::test]
    async fn test_all_user_types_helper_function() {
        // Act: Call the helper function
        let all_types = all_user_types();

        // Assert: Should return all available user types
        assert_eq!(all_types.len(), 3, "Should return exactly 3 user types");
        assert!(all_types.contains(&UserType::Admin), "Should contain Admin type");
        assert!(all_types.contains(&UserType::Developer), "Should contain Developer type");
        assert!(all_types.contains(&UserType::EndUser), "Should contain EndUser type");
    }

}
