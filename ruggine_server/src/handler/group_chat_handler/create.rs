use crate::dto::group_chat_dto::{GroupChatCreateDto, GroupChatReadDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::response::api_response::ApiSuccessResponse;
use crate::state::group_chat_state::GroupChatState;
use axum::{extract::State, Extension, Json};
use crate::entity::user::User;

#[utoipa::path(
    post,
    path = "/api/group_chat/create",
    request_body = GroupChatCreateDto,
    responses(
        (status = 200, description = "Group chat created successfully", body = ApiSuccessResponseGroupChatReadDto),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 422, description = "User id not existing anymore"),
        (status = 500, description = "Internal server error")
    ),
    tag = "GroupChat",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create(
    Extension(current_user): Extension<User>,
    State(state): State<GroupChatState>,
    ValidatedRequest(payload): ValidatedRequest<GroupChatCreateDto>,
) -> Result<Json<ApiSuccessResponse<GroupChatReadDto>>, ApiError> {
    let group_chat = state
        .group_chat_service
        .create(payload, current_user.id)
        .await?;
    
    Ok(Json(ApiSuccessResponse::send(group_chat)))
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::repository::group_chat_repository::group_chat_repository_trait::MockGroupChatRepositoryTrait;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use axum::extract::State;
    use axum::Json;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_success() {
        // Arrange
        let input = GroupChatFactory::fake_group_chat_create_dto();
        let expected_output = GroupChatFactory::fake_group_chat_read_dto();
        let user_id = 1;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_repo = MockUserRepositoryTrait::new();

        let expected_output_clone = expected_output.clone();
        mock_service
            .expect_create()
            .with(eq(input.clone()), eq(user_id))
            .returning(move |_, _| {
                let dto = expected_output_clone.clone();
                Box::pin(async move { Ok(dto) })
            });

        let state = GroupChatState::with_dependencies(
            Arc::new(mock_service),
            Arc::new(mock_group_repo),
            Arc::new(mock_user_repo),
        );

        // Act
        let result = create_group_chat(State(state), claims, ValidatedRequest(input)).await;

        // Assert
        assert!(result.is_ok());
        let Json(response) = result.unwrap();
        assert_eq!(*response.data(), expected_output);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_user_not_found() {
        // Arrange
        let input = GroupChatFactory::fake_group_chat_create_dto();
        let user_id = 999; // Non-existent user

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_repo = MockUserRepositoryTrait::new();

        mock_service
            .expect_create()
            .with(eq(input.clone()), eq(user_id))
            .returning(move |_, _| {
                Box::pin(async move {
                    Err(ApiError::UserError(crate::error::user_error::UserError::UserNotFound))
                })
            });

        let state = GroupChatState::with_dependencies(
            Arc::new(mock_service),
            Arc::new(mock_group_repo),
            Arc::new(mock_user_repo),
        );

        let claims = Claims { user_id };

        // Act
        let result = create_group_chat(State(state), claims, ValidatedRequest(input)).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::UserError(_)));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_database_error() {
        // Arrange
        let input = GroupChatFactory::fake_group_chat_create_dto();
        let user_id = 1;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_repo = MockUserRepositoryTrait::new();

        mock_service
            .expect_create()
            .with(eq(input.clone()), eq(user_id))
            .returning(move |_, _| {
                Box::pin(async move {
                    Err(ApiError::DatabaseError("Database connection failed".to_string()))
                })
            });

        let state = GroupChatState::with_dependencies(
            Arc::new(mock_service),
            Arc::new(mock_group_repo),
            Arc::new(mock_user_repo),
        );

        let claims = Claims { user_id };

        // Act
        let result = create_group_chat(State(state), claims, ValidatedRequest(input)).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DatabaseError(_)));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_with_factory_data() {
        // Arrange - using unique factory data
        let input = GroupChatFactory::unique_fake_group_chat_create_dto("test_handler");
        let expected_output = GroupChatFactory::fake_group_chat_read_dto();
        let user_id = 42;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_repo = MockUserRepositoryTrait::new();

        let expected_output_clone = expected_output.clone();
        mock_service
            .expect_create()
            .with(eq(input.clone()), eq(user_id))
            .returning(move |_, _| {
                let dto = expected_output_clone.clone();
                Box::pin(async move { Ok(dto) })
            });

        let state = GroupChatState::with_dependencies(
            Arc::new(mock_service),
            Arc::new(mock_group_repo),
            Arc::new(mock_user_repo),
        );

        let claims = Claims { user_id };

        // Act
        let result = create_group_chat(State(state), claims, ValidatedRequest(input.clone())).await;

        // Assert
        assert!(result.is_ok());
        let Json(response) = result.unwrap();

        // Verify the input data was correctly used
        assert!(input.name.contains("test_handler"));
        assert!(input.description.contains("test_handler"));
        assert_eq!(*response.data(), expected_output);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_propagates_service_errors() {
        // Arrange
        let input = GroupChatFactory::fake_group_chat_create_dto();
        let user_id = 1;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_repo = MockUserRepositoryTrait::new();

        mock_service
            .expect_create()
            .with(eq(input.clone()), eq(user_id))
            .returning(move |_, _| {
                Box::pin(async move {
                    Err(ApiError::DatabaseError("Constraint violation".to_string()))
                })
            });

        let state = GroupChatState::with_dependencies(
            Arc::new(mock_service),
            Arc::new(mock_group_repo),
            Arc::new(mock_user_repo),
        );

        let claims = Claims { user_id };

        // Act
        let result = create_group_chat(State(state), claims, ValidatedRequest(input)).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        if let ApiError::DatabaseError(msg) = error {
            assert_eq!(msg, "Constraint violation");
        } else {
            panic!("Expected DatabaseError, got: {:?}", error);
        }
    }
}

 */