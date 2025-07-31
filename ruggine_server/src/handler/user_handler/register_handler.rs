use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::response::api_response::ApiSuccessResponse;
use crate::state::user_state::UserState;
use axum::{extract::State, Json};

#[utoipa::path(
    post,
    path = "/api/user/register",
    request_body = UserRegisterDto,
    responses(
        (status = 200, description = "User registered successfully", body = ApiSuccessResponseUserReadDto),
        (status = 400, description = "Invalid request data"),
        (status = 409, description = "User already exists")
    ),
    tag = "User"
)]
pub async fn register(
    State(state): State<UserState>,
    ValidatedRequest(payload): ValidatedRequest<UserRegisterDto>,
) -> Result<Json<ApiSuccessResponse<UserReadDto>>, ApiError> {
    let user = state.user_service.create_user(payload).await?;
    Ok(Json(ApiSuccessResponse::send(user)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::user_error::UserError;
    use crate::factory::user_factory::UserFactory;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::state::user_state::UserState;
    use axum::extract::State;
    use axum::Json;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio_shared_rt::test(shared)]
    async fn test_register_returns_created_user() {
        // Arrange: mock input and expected output
        let input = UserFactory::fake_user_register_dto();

        let expected_output = UserFactory::fake_read_user_dto();

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
        assert_eq!(*user.data(), expected_output);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_register_handles_user_already_exists_error() {
        // Arrange: mock input and service error
        let input = UserFactory::fake_user_register_dto();

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        
        mock_service
            .expect_create_user()
            .with(eq(input.clone()))
            .returning(move |_| {
                Box::pin(async move { 
                    Err(ApiError::UserError(UserError::UserAlreadyExists("Username or email already taken".to_string())))
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
        assert!(matches!(error, ApiError::UserError(UserError::UserAlreadyExists(_))));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_register_with_factory_data() {
        // Arrange: use factory to create test data
        let input = UserFactory::fake_user_register_dto();
        let expected_output = UserFactory::fake_read_user_dto();

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
        assert_eq!(user.data().email, expected_output.email);
        assert_eq!(user.data().username, expected_output.username);
        assert_eq!(user.data().first_name, expected_output.first_name);
        assert_eq!(user.data().last_name, expected_output.last_name);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_register_propagates_service_errors() {
        // Arrange: mock input and database error
        let input = UserFactory::fake_user_register_dto();

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
