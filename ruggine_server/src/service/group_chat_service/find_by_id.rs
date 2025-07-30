use crate::dto::group_chat_dto::GroupChatReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_chat_error::GroupChatError;
use crate::service::group_chat_service::GroupChatService;

impl GroupChatService {
    pub async fn find_by_id_internal(&self, id: i32) -> Result<GroupChatReadDto, ApiError> {
        let group_chat = self.group_chat_repo.find_by_id(id).await.map_err(|e| {
            let db_error = match e {
                sqlx::Error::RowNotFound => ApiError::GroupChatError(GroupChatError::GroupChatNotFound),

                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        if code == "23505" {
                            ApiError::DbError(DbError::UniqueConstraintViolation(db_err.to_string()))
                        } else {
                            ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                        }
                    } else {
                        ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                    }
                }

                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            };
            db_error
        })?;

        Ok(GroupChatReadDto::from(group_chat))
    }
}

#[cfg(test)]
mod find_by_id_service_tests {
    use super::*;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::repository::group_chat_repository::group_chat_repository_trait::MockGroupChatRepositoryTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::service::group_chat_service::GroupChatService;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        
        let expected_group = GroupChatFactory::fake_group_chat();
        let group_id = expected_group.id;
        let expected_dto = GroupChatReadDto::from(expected_group.clone());

        mock_group_repo
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let group = GroupChatFactory::fake_group_chat();
                Box::pin(async move { Ok(group) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );

        // Act
        let result = service.find_by_id_internal(group_id).await;

        // Assert
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.id, expected_dto.id);
        assert_eq!(dto.name, expected_dto.name);
        assert_eq!(dto.description, expected_dto.description);
        assert_eq!(dto.created_by, expected_dto.created_by);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let non_existent_id = 999;

        mock_group_repo
            .expect_find_by_id()
            .with(eq(non_existent_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(sqlx::Error::RowNotFound) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );

        // Act
        let result = service.find_by_id_internal(non_existent_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::GroupChatError(GroupChatError::GroupChatNotFound)));
    }

    #[tokio::test]
    async fn test_find_by_id_database_error() {
        // Arrange
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let group_id = 1;

        mock_group_repo
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(sqlx::Error::PoolClosed) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );

        // Act
        let result = service.find_by_id_internal(group_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }

    #[tokio::test]
    async fn test_find_by_id_with_different_groups() {
        // Arrange
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let group1 = GroupChatFactory::fake_group_chat();
        let mut group2 = GroupChatFactory::fake_group_chat();
        group2.id = 2;
        group2.name = "Different Group".to_string();

        let group1_clone = group1.clone();
        let group2_clone = group2.clone();

        mock_group_repo
            .expect_find_by_id()
            .with(eq(group1.id))
            .times(1)
            .returning(move |_| {
                let group = group1_clone.clone();
                Box::pin(async move { Ok(group) })
            });

        mock_group_repo
            .expect_find_by_id()
            .with(eq(group2.id))
            .times(1)
            .returning(move |_| {
                let group = group2_clone.clone();
                Box::pin(async move { Ok(group) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );

        // Act
        let result1 = service.find_by_id_internal(group1.id).await;
        let result2 = service.find_by_id_internal(group2.id).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let dto1 = result1.unwrap();
        let dto2 = result2.unwrap();

        assert_eq!(dto1.id, group1.id);
        assert_eq!(dto2.id, group2.id);
        assert_ne!(dto1.name, dto2.name);
    }

    #[tokio::test]
    async fn test_find_by_id_invalid_id() {
        // Arrange
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let invalid_id = -1;

        mock_group_repo
            .expect_find_by_id()
            .with(eq(invalid_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(sqlx::Error::RowNotFound) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );

        // Act
        let result = service.find_by_id_internal(invalid_id).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::GroupChatError(GroupChatError::GroupChatNotFound)));
    }
}