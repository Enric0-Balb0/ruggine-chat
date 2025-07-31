use crate::dto::user_dto::{UserReadDto, ProfileUpdateDto};
use crate::entity::user::{User, UpdateUser};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest, user_error::UserError};
use crate::response::api_response::ApiSuccessResponse;
use crate::state::user_state::UserState;
use axum::{extract::State, Extension, Json};
use validator::Validate;

#[utoipa::path(
    patch,
    path = "/api/user/profile",
    request_body = ProfileUpdateDto,
    responses(
        (status = 200, description = "User profile updated successfully", body = ApiSuccessResponseUserReadDto),
        (status = 400, description = "Bad Request - Invalid input data"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User"
)]
pub async fn update_profile(
    Extension(current_user): Extension<User>,
    State(state): State<UserState>,
    ValidatedRequest(payload): ValidatedRequest<ProfileUpdateDto>,
) -> Result<Json<ApiSuccessResponse<UserReadDto>>, ApiError> {
    // Update the user profile
    let updated_user = state.user_service.update_user_profile(current_user.id, payload).await?;

    Ok(Json(ApiSuccessResponse::send(updated_user)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::user_factory::UserFactory;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::error::{api_error::ApiError, user_error::UserError};
    use axum::{extract::State, Extension};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_success() {
        // Arrange
        let user = UserFactory::fake_user();
        let update_dto = UserFactory::fake_user_update_dto();
        let mut expected_user = user.clone();
        expected_user.first_name = "JohnUpdate".to_string();
        expected_user.last_name = "DoeUpdate".to_string();

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        
        mock_service
            .expect_update_user_profile()
            .with(eq(user.id), function(|update_user: &ProfileUpdateDto| {
                UpdateUser::from_dto(update_user.clone()).has_updates()
            }))
            .times(1)
            .returning(move |_, _| {
                let user_dto = UserReadDto::from(expected_user.clone());
                Box::pin(async move { Ok(user_dto) })
            });

        let state = UserState {
            user_service: Arc::new(mock_service),
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = update_profile(
            Extension(user.clone()),
            State(state),
            ValidatedRequest(update_dto),
        )
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.data().first_name, "JohnUpdate");
    }


    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_no_updates() {
        // Arrange
        let user = UserFactory::fake_user();
        let empty_update_dto = UserFactory::fake_user_update_dto_empty();

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        
        mock_service
            .expect_update_user_profile()
            .with(eq(user.id), function(|update_dto: &ProfileUpdateDto| {
                UpdateUser::from_dto(update_dto.clone()).has_updates() == false
            }))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { 
                    Err(ApiError::UserError(UserError::NoFieldsToUpdate))
                })
            });
        
        let state = UserState {
            user_service: Arc::new(mock_service),
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = update_profile(
            Extension(user),
            State(state),
            ValidatedRequest(empty_update_dto),
        ).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::UserError(UserError::NoFieldsToUpdate) => {},
            other => panic!("Expected UserError::NoFieldsToUpdate, got: {:?}", other),
        }
    }


    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_service_error() {
        // Arrange
        let user = UserFactory::fake_user();
        let update_dto = UserFactory::fake_user_update_dto();

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        
        mock_service
            .expect_update_user_profile()
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { 
                    Err(ApiError::UserError(UserError::UserNotFound))
                })
            });

        let state = UserState {
            user_service: Arc::new(mock_service),
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = update_profile(
            Extension(user),
            State(state),
            ValidatedRequest(update_dto),
        ).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserNotFound) => {},
            _ => panic!("Expected UserError::UserNotFound"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_partial_update() {
        // Arrange
        let user = UserFactory::fake_user();
        let partial_update_dto = UserFactory::fake_user_update_dto_partial();
        let mut expected_user = user.clone();
        expected_user.first_name = "JohnUpdate".to_string();
        expected_user.address = "465 Oak Ave Update".to_string();

        let mut mock_service = MockUserServiceTrait::new();
        let mock_repo = MockUserRepositoryTrait::new();
        
        mock_service
            .expect_update_user_profile()
            .with(eq(user.id), function(|update_dto: &ProfileUpdateDto| {
                update_dto.first_name.is_some() &&
                    update_dto.last_name.is_none() &&
                    update_dto.address.is_some()
            }))
            .times(1)
            .returning(move |_, _| {
                let user_dto = UserReadDto::from(expected_user.clone());
                Box::pin(async move { Ok(user_dto) })
            });

        let state = UserState {
            user_service: Arc::new(mock_service),
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = update_profile(
            Extension(user.clone()),
            State(state),
            ValidatedRequest(partial_update_dto),
        ).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.data().id, user.id);
        assert_eq!(response.data().first_name, "JohnUpdate");
    }
}
