use crate::dto::group_chat_dto::GroupChatReadDto;
use crate::error::api_error::ApiError;
use crate::response::api_response::ApiSuccessResponse;
use crate::state::group_chat_state::GroupChatState;
use crate::entity::user::User;
use axum::{extract::{Path, State}, Extension, Json};

#[utoipa::path(
    get,
    path = "/api/group_chat/{id}",
    params(
        ("id" = i32, Path, description = "Group chat ID to retrieve")
    ),
    responses(
        (status = 200, description = "Group chat retrieved successfully", body = ApiSuccessResponseGroupChatReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Group chat not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "GroupChat",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_by_id(
    Extension(_current_user): Extension<User>,
    State(state): State<GroupChatState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiSuccessResponse<GroupChatReadDto>>, ApiError> {
    let group_chat = state.group_chat_service.find_by_id(id).await?;
    Ok(Json(ApiSuccessResponse::send(group_chat)))
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::factory::user_factory::UserFactory;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::repository::group_chat_repository::group_chat_repository_trait::MockGroupChatRepositoryTrait;
    use crate::state::group_chat_state::GroupChatState;
    use crate::error::api_error::ApiError;
    use crate::error::db_error::DbError;
    use axum::{Extension, extract::{Path, State}};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_by_id_success() {
        // Arrange
        let user = UserFactory::fake_user();
        let expected_group_dto = GroupChatFactory::fake_group_chat_read_dto();
        let group_id = expected_group_dto.id;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_repo = MockGroupChatRepositoryTrait::new();

        mock_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let dto = GroupChatFactory::fake_group_chat_read_dto();
                Box::pin(async move { Ok(dto) })
            });

        let state = GroupChatState {
            group_chat_service: Arc::new(mock_service),
            group_chat_repo: Arc::new(mock_repo),
        };

        // Act
        let result = find_by_id(
            Extension(user),
            State(state),
            Path(group_id),
        ).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.data().id, expected_group_dto.id);
        assert_eq!(response.data().name, expected_group_dto.name);
        assert_eq!(response.data().description, expected_group_dto.description);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        // Arrange
        let user = UserFactory::fake_user();
        let non_existent_id = 999;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_repo = MockGroupChatRepositoryTrait::new();

        mock_service
            .expect_find_by_id()
            .with(eq(non_existent_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { 
                    Err(ApiError::DbError(DbError::SomethingWentWrong("Group chat not found".to_string())))
                })
            });

        let state = GroupChatState {
            group_chat_service: Arc::new(mock_service),
            group_chat_repo: Arc::new(mock_repo),
        };

        // Act
        let result = find_by_id(
            Extension(user),
            State(state),
            Path(non_existent_id),
        ).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {},
            _ => panic!("Expected DbError::SomethingWentWrong"),
        }
    }

    #[tokio::test]
    async fn test_find_by_id_service_error() {
        // Arrange
        let user = UserFactory::fake_user();
        let group_id = 1;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_repo = MockGroupChatRepositoryTrait::new();

        mock_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { 
                    Err(ApiError::DbError(DbError::SomethingWentWrong("Database connection failed".to_string())))
                })
            });

        let state = GroupChatState {
            group_chat_service: Arc::new(mock_service),
            group_chat_repo: Arc::new(mock_repo),
        };

        // Act
        let result = find_by_id(
            Extension(user),
            State(state),
            Path(group_id),
        ).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(msg)) => {
                assert_eq!(msg, "Database connection failed");
            },
            _ => panic!("Expected DbError::SomethingWentWrong"),
        }
    }

    #[tokio::test]
    async fn test_find_by_id_with_different_users() {
        // Arrange
        let user1 = UserFactory::unique_fake_user("find_test_1", crate::entity::user::UserStatus::Active);
        let user2 = UserFactory::unique_fake_user("find_test_2", crate::entity::user::UserStatus::Active);
        let group_dto = GroupChatFactory::fake_group_chat_read_dto();
        let group_id = group_dto.id;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_repo = MockGroupChatRepositoryTrait::new();

        // Both users should be able to retrieve the same group
        mock_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(2)
            .returning(move |_| {
                let dto = GroupChatFactory::fake_group_chat_read_dto();
                Box::pin(async move { Ok(dto) })
            });

        let state = GroupChatState {
            group_chat_service: Arc::new(mock_service),
            group_chat_repo: Arc::new(mock_repo),
        };

        // Act
        let result1 = find_by_id(
            Extension(user1),
            State(state.clone()),
            Path(group_id),
        ).await;

        let result2 = find_by_id(
            Extension(user2),
            State(state),
            Path(group_id),
        ).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let response1 = result1.unwrap().0;
        let response2 = result2.unwrap().0;
        
        assert_eq!(response1.data().id, group_dto.id);
        assert_eq!(response2.data().id, group_dto.id);
        assert_eq!(response1.data().name, response2.data().name);
    }

    #[tokio::test]
    async fn test_find_by_id_invalid_id() {
        // Arrange
        let user = UserFactory::fake_user();
        let invalid_id = -1;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_repo = MockGroupChatRepositoryTrait::new();

        mock_service
            .expect_find_by_id()
            .with(eq(invalid_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { 
                    Err(ApiError::DbError(DbError::SomethingWentWrong("Invalid group chat ID".to_string())))
                })
            });

        let state = GroupChatState {
            group_chat_service: Arc::new(mock_service),
            group_chat_repo: Arc::new(mock_repo),
        };

        // Act
        let result = find_by_id(
            Extension(user),
            State(state),
            Path(invalid_id),
        ).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(msg)) => {
                assert_eq!(msg, "Invalid group chat ID");
            },
            _ => panic!("Expected DbError::SomethingWentWrong"),
        }
    }

    #[tokio::test]
    async fn test_find_by_id_response_structure() {
        // Arrange
        let user = UserFactory::fake_user();
        let group_dto = GroupChatFactory::fake_group_chat_read_dto();
        let group_id = group_dto.id;

        let mut mock_service = MockGroupChatServiceTrait::new();
        let mock_repo = MockGroupChatRepositoryTrait::new();

        mock_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let dto = GroupChatFactory::fake_group_chat_read_dto();
                Box::pin(async move { Ok(dto) })
            });

        let state = GroupChatState {
            group_chat_service: Arc::new(mock_service),
            group_chat_repo: Arc::new(mock_repo),
        };

        // Act
        let result = find_by_id(
            Extension(user),
            State(state),
            Path(group_id),
        ).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap().0;
        let data = response.data();
        
        // Verify response structure
        assert!(data.id > 0, "ID should be positive");
        assert!(!data.name.is_empty(), "Name should not be empty");
        assert!(!data.description.is_empty(), "Description should not be empty");
        assert!(data.created_by > 0, "Created by should be positive");
    }
}
*/