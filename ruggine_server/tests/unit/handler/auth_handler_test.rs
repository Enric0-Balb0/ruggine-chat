use chrono::Utc;
use ruggine_server::dto::{token_dto::TokenReadDto, user_dto::UserLoginDto};
use ruggine_server::entity::user::User;
use ruggine_server::error::token_error::TokenError;
use ruggine_server::repository::user_repository::UserRepositoryTrait;
use ruggine_server::service::token_service::TokenServiceTrait;
use ruggine_server::service::user_service::UserServiceTrait;
use ruggine_server::state::auth_state::AuthState;
use std::sync::Arc;
use async_trait::async_trait;
use sqlx::Error as SqlxError;
use ruggine_server::entity::user::NewUser;
use jsonwebtoken::{TokenData, errors::Result as JwtResult};
use ruggine_server::dto::token_dto::TokenClaimsDto;
use axum::Json;
use axum::extract::State;
use ruggine_server::handler::auth::login_handler::login;
use ruggine_server::error::request_error::ValidatedRequest;
use ruggine_server::error::api_error::ApiError;

// Mock implementations for testing
#[derive(Clone)]
struct MockUserRepository {
    find_by_email_result: Option<User>,
}

#[async_trait]
impl UserRepositoryTrait for MockUserRepository {
    async fn find_by_email(&self, _email: String) -> Option<User> {
        self.find_by_email_result.clone()
    }

    async fn find(&self, _id: u64) -> Result<User, SqlxError> {
        unimplemented!()
    }

    async fn insert(&self, _new_user: NewUser) -> Result<u64, SqlxError> {
        unimplemented!()
    }
}

impl MockUserRepository {
    fn with_user(user: User) -> Self {
        Self {
            find_by_email_result: Some(user),
        }
    }

    fn with_no_user() -> Self {
        Self {
            find_by_email_result: None,
        }
    }
}

#[derive(Clone)]
struct MockTokenService {
    generate_token_result: Result<TokenReadDto, String>, // Use String instead of TokenError for simplicity
}

impl TokenServiceTrait for MockTokenService {
    fn retrieve_token_claims(&self, _token: &str) -> JwtResult<TokenData<TokenClaimsDto>> {
        unimplemented!()
    }

    fn generate_token(&self, _user: User) -> Result<TokenReadDto, TokenError> {
        match &self.generate_token_result {
            Ok(token) => Ok(token.clone()),
            Err(err) => Err(TokenError::TokenCreationError(err.clone())),
        }
    }
}

impl MockTokenService {
    fn with_success() -> Self {
        Self {
            generate_token_result: Ok(TokenReadDto {
                token: "mock_successful_token".to_string(),
                iat: 1234567890,
                exp: 1234567890 + 3600,
            }),
        }
    }

    fn with_error(error: String) -> Self {
        Self {
            generate_token_result: Err(error),
        }
    }
}

#[derive(Clone)]
struct MockUserService {
    verify_password_result: bool,
}

#[async_trait]
impl UserServiceTrait for MockUserService {
    async fn create_user(&self, _payload: ruggine_server::dto::user_dto::UserRegisterDto) -> Result<ruggine_server::dto::user_dto::UserReadDto, ApiError> {
        unimplemented!()
    }

    fn verify_password(&self, _user: &User, _password: &str) -> bool {
        self.verify_password_result
    }
}

impl MockUserService {
    fn with_password_verification(result: bool) -> Self {
        Self {
            verify_password_result: result,
        }
    }
}

#[derive(Clone)]
struct MockAuthState {
    pub token_service: MockTokenService,
    pub user_repo: MockUserRepository,
    pub user_service: MockUserService,
}

impl MockAuthState {
    fn new(
        token_service: MockTokenService,
        user_repo: MockUserRepository,
        user_service: MockUserService,
    ) -> Self {
        Self {
            token_service,
            user_repo,
            user_service,
        }
    }
}

// Helper function to create a test user
fn create_test_user(is_active: i8) -> User {
    User {
        id: 1,
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        user_name: "johndoe".to_string(),
        email: "john@example.com".to_string(),
        password: "$2b$12$example_hashed_password".to_string(),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        is_active,
    }
}

// Helper function to create a valid login payload
fn create_valid_login_payload() -> UserLoginDto {
    UserLoginDto {
        email: "john@example.com".to_string(),
        password: "password123".to_string(),
    }
}

impl From<MockAuthState> for AuthState {
    fn from(mock: MockAuthState) -> Self {
        Self {
            token_service: Arc::new(mock.token_service),
            user_repo: Arc::new(mock.user_repo),
            user_service: Arc::new(mock.user_service), // se necessario implementare UserServiceTrait per MockUserService
        }
    }
}

#[cfg(test)]
mod login_handler_unit_tests {
    use ruggine_server::error::user_error::UserError;

    use super::*;

    #[tokio::test]
    async fn test_login_success() {
        let user = create_test_user(1);
        let user_repo = MockUserRepository::with_user(user.clone());
        let user_service = MockUserService::with_password_verification(true);
        let token_service = MockTokenService::with_success();

        let mock_state = MockAuthState::new(token_service, user_repo, user_service);
        let auth_state = AuthState::from(mock_state);

        let payload = create_valid_login_payload();

        let result = login(
            State(auth_state),
            ValidatedRequest(payload),
        ).await;

        assert!(result.is_ok());
        let Json(token_dto) = result.unwrap();
        assert_eq!(token_dto.token, "mock_successful_token");
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let user_repo = MockUserRepository::with_no_user();
        let user_service = MockUserService::with_password_verification(true);
        let token_service = MockTokenService::with_success();

        let mock_state = MockAuthState::new(token_service, user_repo, user_service);
        let auth_state = AuthState::from(mock_state);
        let payload = create_valid_login_payload();

        let result = login(
            State(auth_state),
            ValidatedRequest(payload),
        ).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ApiError::UserError(UserError::UserNotFound) => {}
                e => panic!("Expected UserNotFound error, got {:?}", e),
            }
        }
    }

    #[tokio::test]
    async fn test_login_user_not_active() {
        let user = create_test_user(0); // inactive user
        let user_repo = MockUserRepository::with_user(user);
        let user_service = MockUserService::with_password_verification(true);
        let token_service = MockTokenService::with_success();

        let mock_state = MockAuthState::new(token_service, user_repo, user_service);
        let auth_state = AuthState::from(mock_state);
        let payload = create_valid_login_payload();

        let result = login(
            State(auth_state),
            ValidatedRequest(payload),
        ).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ApiError::UserError(UserError::UserNotActive) => {}
                e => panic!("Expected UserNotActive error, got {:?}", e),
            }
        }
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let user = create_test_user(1);
        let user_repo = MockUserRepository::with_user(user.clone());
        let user_service = MockUserService::with_password_verification(false); // wrong password
        let token_service = MockTokenService::with_success();

        let mock_state = MockAuthState::new(token_service, user_repo, user_service);
        let auth_state = AuthState::from(mock_state);
        let payload = create_valid_login_payload();

        let result = login(
            State(auth_state),
            ValidatedRequest(payload),
        ).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ApiError::UserError(UserError::InvalidPassword) => {}
                e => panic!("Expected InvalidPassword error, got {:?}", e),
            }
        }
    }

    #[tokio::test]
    async fn test_login_token_generation_failure() {
        let user = create_test_user(1);
        let user_repo = MockUserRepository::with_user(user.clone());
        let user_service = MockUserService::with_password_verification(true);
        let token_service = MockTokenService::with_error("Failed to create token".to_string());

        let mock_state = MockAuthState::new(token_service, user_repo, user_service);
        let auth_state = AuthState::from(mock_state);
        let payload = create_valid_login_payload();

        let result = login(
            State(auth_state),
            ValidatedRequest(payload),
        ).await;

        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                ApiError::TokenError(_) => {}  // Adjust if your ApiError has a different variant
                e => panic!("Expected TokenError, got {:?}", e),
            }
        }
    }

}