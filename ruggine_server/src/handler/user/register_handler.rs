use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::service::user_service::UserServiceTrait;
use crate::state::user_state::UserState;
use axum::{extract::State, Json};

pub async fn register(
    State(state): State<UserState>,
    ValidatedRequest(payload): ValidatedRequest<UserRegisterDto>,
) -> Result<Json<UserReadDto>, ApiError> {
    let user = state.user_service.create_user(payload).await?;
    Ok(Json(user))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
    use crate::error::user_error::UserError;
    use crate::factory::user_factory::UserFactory;
    use crate::service::user_service::MockUserServiceTrait;
    use crate::repository::user_repository::MockUserRepositoryTrait;
    use crate::state::user_state::UserState;
    use axum::extract::State;
    use axum::Json;
    use chrono::Utc;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_register_returns_created_user() {
        // Arrange: mock input and expected output
        let input = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "securepassword".to_string(),
            user_name: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
        };

        let expected_output = UserReadDto {
            id: 1,
            email: input.email.clone(),
            user_name: input.user_name.clone(),
            first_name: input.first_name.clone(),
            last_name: input.last_name.clone(),
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
            is_active: 1,
        };

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        let expected_output_clone = expected_output.clone();
        mock_service
            .expect_create_user()
            .with(eq(input.clone()))
            .returning(move |_| {
                let dto = expected_output_clone.clone();
                Box::pin(async move { Ok(dto) })
            });

        let state = UserState {
            user_service: Arc::new(mock_service),
            user_repo: Arc::new(mock_repo),
        };

        // Act: call the handler
        let result = register(State(state), ValidatedRequest(input)).await;

        // Assert
        assert!(result.is_ok());
        let Json(user) = result.unwrap();
        assert_eq!(user, expected_output);
    }

    #[tokio::test]
    async fn test_register_handles_user_already_exists_error() {
        // Arrange: mock input and service error
        let input = UserRegisterDto {
            email: "existing@example.com".to_string(),
            password: "password123".to_string(),
            user_name: "existinguser".to_string(),
            first_name: "Existing".to_string(),
            last_name: "User".to_string(),
        };

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        
        mock_service
            .expect_create_user()
            .with(eq(input.clone()))
            .returning(move |_| {
                Box::pin(async move { 
                    Err(ApiError::UserError(UserError::UserAlreadyExists))
                })
            });

        let state = UserState {
            user_service: Arc::new(mock_service),
            user_repo: Arc::new(mock_repo),
        };

        // Act: call the handler
        let result = register(State(state), ValidatedRequest(input)).await;

        // Assert: should return error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::UserError(UserError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn test_register_with_factory_data() {
        // Arrange: use factory to create test data
        let input = UserFactory::fake_user_register_dto();
        let expected_output = UserReadDto {
            id: 42,
            email: input.email.clone(),
            user_name: input.user_name.clone(),
            first_name: input.first_name.clone(),
            last_name: input.last_name.clone(),
            created_at: Utc::now(),
            updated_at: None,
            is_active: 1,
        };

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        let expected_output_clone = expected_output.clone();
        
        mock_service
            .expect_create_user()
            .with(eq(input.clone()))
            .returning(move |_| {
                let dto = expected_output_clone.clone();
                Box::pin(async move { Ok(dto) })
            });

        let state = UserState {
            user_service: Arc::new(mock_service),
            user_repo: Arc::new(mock_repo),
        };

        // Act: call the handler
        let result = register(State(state), ValidatedRequest(input)).await;

        // Assert: verify successful registration
        assert!(result.is_ok());
        let Json(user) = result.unwrap();
        assert_eq!(user.email, expected_output.email);
        assert_eq!(user.user_name, expected_output.user_name);
        assert_eq!(user.first_name, expected_output.first_name);
        assert_eq!(user.last_name, expected_output.last_name);
    }

    #[tokio::test]
    async fn test_register_propagates_service_errors() {
        // Arrange: mock input and database error
        let input = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            user_name: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
        };

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        
        mock_service
            .expect_create_user()
            .with(eq(input.clone()))
            .returning(move |_| {
                Box::pin(async move { 
                    Err(ApiError::UserError(UserError::UserNotFound))
                })
            });

        let state = UserState {
            user_service: Arc::new(mock_service),
            user_repo: Arc::new(mock_repo),
        };

        // Act: call the handler
        let result = register(State(state), ValidatedRequest(input)).await;

        // Assert: error should be propagated correctly
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::UserError(_)));
    }
}
