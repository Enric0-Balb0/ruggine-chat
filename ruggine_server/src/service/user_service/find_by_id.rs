use crate::dto::user_dto::UserReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::service::user_service::UserService;

impl UserService {
    pub async fn find_by_id_internal(&self, id: i32) -> Result<UserReadDto, ApiError> {
        let user = self.user_repo.find(id).await.map_err(|e| {
            // Qui fai la conversione da sqlx::Error a DbError
            let db_error = match e {
                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        if code == "23505" {
                            return ApiError::DbError(DbError::UniqueConstraintViolation(db_err.to_string()));
                        }
                    }
                    ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                }
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            };
            db_error
        })?;

        Ok(UserReadDto::from(user))
    }
}

#[cfg(test)]
mod find_by_id_service_tests {
    use super::*;
    use crate::factory::user_factory::UserFactory;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::service::user_service::UserService;
    use mockall::predicate::*;
    use std::sync::Arc;
    use crate::entity::user::UserStatus;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        let expected_user = UserFactory::fake_user();
        let user_id = expected_user.id;
        let expected_dto = UserReadDto::from(expected_user.clone());

        mock_repo
            .expect_find()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                let value = expected_user.clone();
                Box::pin(async move { Ok(value.clone()) })
            });

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = service.find_by_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto, expected_dto);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        let non_existent_id = 999;

        mock_repo
            .expect_find()
            .with(eq(non_existent_id))
            .times(1)
            .returning(move |_| Box::pin(async move {Err(sqlx::Error::RowNotFound)}));

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = service.find_by_id_internal(non_existent_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_database_error() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        let user_id = 1;

        mock_repo
            .expect_find()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {Err(sqlx::Error::RowNotFound)})
            });

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = service.find_by_id_internal(user_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }
    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_with_factory_users() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        
        // Create multiple users using factory
        let user1 = UserFactory::unique_fake_user("mock_find_by_id_1", UserStatus::Active);
        let user2 = UserFactory::unique_fake_user("mock_find_by_id_2", UserStatus::Active);
        let user3 = UserFactory::unique_fake_user("mock_find_by_id_3", UserStatus::Active);

        let expected_dto1 = UserReadDto::from(user1.clone());
        let expected_dto2 = UserReadDto::from(user2.clone());
        let expected_dto3 = UserReadDto::from(user3.clone());

        // Setup expectations for each user

        mock_repo
            .expect_find()
            .with(eq(1))
            .times(1)
            .returning(move |_| {
                let value = user1.clone();
                Box::pin(async move { Ok(value.clone())})
            });

        mock_repo
            .expect_find()
            .with(eq(2))
            .times(1)
            .returning(move |_| {
                let value = user2.clone();
                Box::pin(async move { Ok(value.clone())})
            });

        mock_repo
            .expect_find()
            .with(eq(3))
            .times(1)
            .returning(move |_| {
                let value = user3.clone();
                Box::pin(async move { Ok(value.clone())})
            });

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result1 = service.find_by_id_internal(1).await;
        let result2 = service.find_by_id_internal(2).await;
        let result3 = service.find_by_id_internal(3).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        
        assert_eq!(result1.unwrap(), expected_dto1);
        assert_eq!(result2.unwrap(), expected_dto2);
        assert_eq!(result3.unwrap(), expected_dto3);
    }
}
