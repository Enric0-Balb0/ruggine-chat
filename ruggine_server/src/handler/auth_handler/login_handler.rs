use crate::dto::{token_dto::TokenReadDto, user_dto::UserLoginDto};
use crate::entity::user::UserStatus;
use crate::error::{api_error::ApiError,request_error::ValidatedRequest, user_error::UserError};
use crate::response::api_response::ApiSuccessResponse;
use crate::state::auth_state::AuthState;
use axum::{extract::State, Json};
use crate::error::db_error::DbError;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = UserLoginDto,
    responses(
        (status = 200, description = "Login successful", body = ApiSuccessResponseTokenReadDto),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Invalid credentials"),
        (status =404, description = "User not found"),
        (status = 403, description = "User not active")
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AuthState>,
    ValidatedRequest(payload): ValidatedRequest<UserLoginDto>,
) -> Result<Json<ApiSuccessResponse<TokenReadDto>>, ApiError> {
    let user = state
        .user_repo()
        .find_by_email(payload.email.clone())
        .await
        .map_err(|e| {
            DbError::SomethingWentWrong(format!(
                "Something went wrong finding user with email {} in the auth middleware, got: {:?}",
                payload.email.clone(), e
            ))
        })?
        .ok_or(UserError::UserNotFound)?;
    
    if user.user_status != UserStatus::Active {
        return Err(UserError::UserNotActive.into());
    }

    match state.user_service().verify_password(&user, &payload.password) {
        true => Ok(Json(ApiSuccessResponse::send(state.token_service().generate_token(user)?))),
        false => Err(UserError::InvalidPassword)?,
    }
}

#[cfg(test)]
mod login_tests {
    use super::*;
    use crate::entity::user::{Gender, User, UserStatus};
    use crate::dto::token_dto::TokenReadDto;
    use crate::error::user_error::UserError;
    // Import the auto-generated mocks
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::service::token_service::MockTokenServiceTrait;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use mockall::predicate::*;
    use std::sync::Arc;
    use chrono::Utc;

    // MOCK IMPLEMENTATIONS WITH #[automock]
    // Instead of manually defining mocks, we use the auto-generated ones from #[automock]
    // This means if the traits change, our mocks automatically update too!
    
    // ADVANTAGES OF #[automock]:
    // ✅ Automatic synchronization: If you add/remove/change methods in traits, mocks update automatically
    // ✅ No manual maintenance: No need to manually update mock implementations
    // ✅ Type safety: Compile-time errors if trait signatures change
    // ✅ Less code: No need to write mock! {} blocks manually
    // ✅ Consistent behavior: All mocks follow the same pattern
    
    // Auto-generated mocks are available as:
    // - MockUserRepositoryTrait (from user_repository.rs with #[automock])
    // - MockUserServiceTrait (from user_service.rs with #[automock]) 
    // - MockTokenServiceTrait (from token_service.rs with #[automock])
    
    // Usage example:
    // let mut mock = MockUserRepositoryTrait::new();
    // mock.expect_find_by_email().returning(|_| Box::pin(async { None }));

    // HELPER FUNCTIONS
    // These functions create test data that we'll use across multiple tests
    
    /// Creates a test user with the specified active status
    /// @param user_status
    fn create_test_user(user_status: UserStatus) -> User {
        User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status, // This determines if user is active or not
            user_type: Default::default(), // Default user type
            birthday: Utc::now().naive_utc().into(),
            is_online: false,
            address: "123 Main St".to_string(),
            gender: Gender::Other,
        }
    }

    /// Creates a sample JWT token response for testing
    fn create_test_token() -> TokenReadDto {
        TokenReadDto {
            token: "test_token".to_string(),
            iat: Utc::now().timestamp(),        // issued at timestamp
            exp: Utc::now().timestamp() + 1800, // expires in 30 minutes
        }
    }

    /// Creates an AuthState with our auto-generated mock objects instead of real services
    /// This allows us to control exactly what these services return in our tests
    fn create_auth_state_with_mocks(
        user_repo: MockUserRepositoryTrait,
        user_service: MockUserServiceTrait,
        token_service: MockTokenServiceTrait,
    ) -> AuthState {
        AuthState {
            user_repo: Arc::new(user_repo),
            user_service: Arc::new(user_service),
            token_service: Arc::new(token_service),
        }
    }

    // TEST CASE 1: Successful Login
    // This test verifies that a login works correctly when all conditions are met:
    // - User exists in database
    // - User is active
    // - Password is correct
    // - Token generation succeeds
    #[tokio_shared_rt::test(shared)]
    async fn test_login_success() {
        // ARRANGE - Set up the test data and mock expectations
        let mut mock_user_repo = MockUserRepositoryTrait::new(); // Auto-generated mock!
        let mut mock_user_service = MockUserServiceTrait::new(); // Auto-generated mock!
        let mut mock_token_service = MockTokenServiceTrait::new(); // Auto-generated mock!
        
        let test_user = create_test_user(UserStatus::Active);
        let expected_token = create_test_token();

        // MOCK EXPECTATIONS - Tell the mocks what to return when called
        
        // When find_by_email is called with "john@example.com", return our test user
        mock_user_repo
            .expect_find_by_email()
            .with(eq("john@example.com".to_string())) // Expect this exact email
            .times(1)                                  // Expect to be called exactly once
            .returning(move |_| {
                let user = test_user.clone();
                Box::pin(async move { Ok(Some(user)) }) // Return Future for async function
            });

        // When verify_password is called with any user and "correct_password", return true
        mock_user_service
            .expect_verify_password()
            .with(always(), eq("correct_password")) // any user, specific password
            .times(1)                               // called exactly once
            .returning(|_, _| true);                // password verification succeeds

        // When generate_token is called with any user, return our test token
        mock_token_service
            .expect_generate_token()
            .times(1)
            .returning({
                let token = expected_token.clone();
                move |_| Ok(token.clone()) // Return successful token generation
            });

        // Create the auth state with our mocked services
        let auth_state = create_auth_state_with_mocks(mock_user_repo, mock_user_service, mock_token_service);
        
        // Create the login request data
        let login_dto = UserLoginDto {
            email: "john@example.com".to_string(),
            password: "correct_password".to_string(),
        };

        // ACT - Execute the function we're testing
        let result = login(
            axum::extract::State(auth_state),
            crate::error::request_error::ValidatedRequest(login_dto),
        ).await;

        // ASSERT - Verify the results are what we expect
        assert!(result.is_ok(), "Login should succeed with valid credentials");
        let token_response = result.unwrap().0;
        assert_eq!(token_response.data().token, "test_token", "Should return the expected token");
        assert!(token_response.data().iat > 0, "Token should have a valid issued-at timestamp");
        assert!(token_response.data().exp > token_response.data().iat, "Token should expire after it was issued");
    }

    // TEST CASE 2: User Not Found
    // This test verifies that login fails correctly when the user doesn't exist
    #[tokio_shared_rt::test(shared)]
    async fn test_login_user_not_found() {
        // ARRANGE
        let mut mock_user_repo = MockUserRepositoryTrait::new(); // Auto-generated mock!
        let mock_user_service = MockUserServiceTrait::new(); // Won't be called in this test
        let mock_token_service = MockTokenServiceTrait::new(); // Won't be called in this test

        // MOCK EXPECTATIONS
        // When find_by_email is called with non-existent email, return None
        mock_user_repo
            .expect_find_by_email()
            .with(eq("nonexistent@example.com".to_string()))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) })); // User not found - return None wrapped in Future

        let auth_state = create_auth_state_with_mocks(mock_user_repo, mock_user_service, mock_token_service);
        
        let login_dto = UserLoginDto {
            email: "nonexistent@example.com".to_string(),
            password: "any_password".to_string(),
        };

        // ACT
        let result = login(
            axum::extract::State(auth_state),
            crate::error::request_error::ValidatedRequest(login_dto),
        ).await;

        // ASSERT - Verify we get the correct error
        assert!(result.is_err(), "Login should fail when user doesn't exist");
        let error = result.unwrap_err();
        match error {
            crate::error::api_error::ApiError::UserError(user_err) => {
                assert!(matches!(user_err, UserError::UserNotFound), "Should return UserNotFound error");
            }
            _ => panic!("Expected UserError::UserNotFound, got different error type"),
        }
    }

    // TEST CASE 3: Inactive User
    // This test verifies that login fails when user exists but is inactive
    #[tokio_shared_rt::test(shared)]
    async fn test_login_user_not_active() {
        // ARRANGE
        let mut mock_user_repo = MockUserRepositoryTrait::new(); // Auto-generated mock!
        let mock_user_service = MockUserServiceTrait::new(); // Won't be called - we fail before password check
        let mock_token_service = MockTokenServiceTrait::new(); // Won't be called
        
        let inactive_user = create_test_user(UserStatus::Deleted); // Inactive user

        // MOCK EXPECTATIONS
        // Return an inactive user when email is found
        mock_user_repo
            .expect_find_by_email()
            .with(eq("john@example.com".to_string()))
            .times(1)
            .returning(move |_| {
                let user = inactive_user.clone();
                Box::pin(async move { Ok(Some(user)) }) // User found but inactive
            });

        let auth_state = create_auth_state_with_mocks(mock_user_repo, mock_user_service, mock_token_service);
        
        let login_dto = UserLoginDto {
            email: "john@example.com".to_string(),
            password: "correct_password".to_string(),
        };

        // ACT
        let result = login(
            axum::extract::State(auth_state),
            crate::error::request_error::ValidatedRequest(login_dto),
        ).await;

        // ASSERT
        assert!(result.is_err(), "Login should fail for inactive user");
        let error = result.unwrap_err();
        match error {
            crate::error::api_error::ApiError::UserError(user_err) => {
                assert!(matches!(user_err, UserError::UserNotActive), "Should return UserNotActive error");
            }
            _ => panic!("Expected UserError::UserNotActive, got different error type"),
        }
    }

    // TEST CASE 4: Invalid Password
    // This test verifies that login fails when password is incorrect
    #[tokio_shared_rt::test(shared)]
    async fn test_login_invalid_password() {
        // ARRANGE
        let mut mock_user_repo = MockUserRepositoryTrait::new(); // Auto-generated mock!
        let mut mock_user_service = MockUserServiceTrait::new(); // Auto-generated mock!
        let mock_token_service = MockTokenServiceTrait::new(); // Won't be called - we fail at password verification
        
        let test_user = create_test_user(UserStatus::Active); // Active user

        // MOCK EXPECTATIONS
        // User exists and is active
        mock_user_repo
            .expect_find_by_email()
            .with(eq("john@example.com".to_string()))
            .times(1)
            .returning(move |_| {
                let user = test_user.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        // Password verification fails
        mock_user_service
            .expect_verify_password()
            .with(always(), eq("wrong_password"))
            .times(1)
            .returning(|_, _| false); // Password verification fails

        let auth_state = create_auth_state_with_mocks(mock_user_repo, mock_user_service, mock_token_service);
        
        let login_dto = UserLoginDto {
            email: "john@example.com".to_string(),
            password: "wrong_password".to_string(),
        };

        // ACT
        let result = login(
            axum::extract::State(auth_state),
            crate::error::request_error::ValidatedRequest(login_dto),
        ).await;

        // ASSERT
        assert!(result.is_err(), "Login should fail with wrong password");
        let error = result.unwrap_err();
        match error {
            crate::error::api_error::ApiError::UserError(user_err) => {
                assert!(matches!(user_err, UserError::InvalidPassword), "Should return InvalidPassword error");
            }
            _ => panic!("Expected UserError::InvalidPassword, got different error type"),
        }
    }

    // TEST CASE 5: Token Generation Failure
    // This test verifies that login fails when JWT token generation fails
    #[tokio_shared_rt::test(shared)]
    async fn test_login_token_generation_failure() {
        // ARRANGE
        let mut mock_user_repo = MockUserRepositoryTrait::new(); // Auto-generated mock!
        let mut mock_user_service = MockUserServiceTrait::new(); // Auto-generated mock!
        let mut mock_token_service = MockTokenServiceTrait::new(); // Auto-generated mock!
        
        let test_user = create_test_user(UserStatus::Active); // Active user

        // MOCK EXPECTATIONS
        // User exists and is active
        mock_user_repo
            .expect_find_by_email()
            .with(eq("john@example.com".to_string()))
            .times(1)
            .returning(move |_| {
                let user = test_user.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        // Password verification succeeds
        mock_user_service
            .expect_verify_password()
            .with(always(), eq("correct_password"))
            .times(1)
            .returning(|_, _| true);

        // Token generation fails
        mock_token_service
            .expect_generate_token()
            .times(1)
            .returning(|_| Err(crate::error::token_error::TokenError::TokenCreationError("Test error".to_string())));

        let auth_state = create_auth_state_with_mocks(mock_user_repo, mock_user_service, mock_token_service);
        
        let login_dto = UserLoginDto {
            email: "john@example.com".to_string(),
            password: "correct_password".to_string(),
        };

        // ACT
        let result = login(
            axum::extract::State(auth_state),
            crate::error::request_error::ValidatedRequest(login_dto),
        ).await;

        // ASSERT
        assert!(result.is_err(), "Login should fail when token generation fails");
        let error = result.unwrap_err();
        match error {
            crate::error::api_error::ApiError::TokenError(token_err) => {
                assert!(matches!(token_err, crate::error::token_error::TokenError::TokenCreationError(_)), 
                    "Should return TokenCreationError");
            }
            _ => panic!("Expected TokenError::TokenCreationError, got different error type"),
        }
    }
}