use crate::dto::user_dto::UserReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::service::user_service::UserService;

impl UserService {
    /// Find user by username and return optional UserReadDto
    /// Returns None if user not found, Some(UserReadDto) if found, or ApiError if database error occurs
    pub async fn find_by_username_internal(&self, username: String) -> Result<Option<UserReadDto>, ApiError> {
        match self.user_repo.find_by_username(username).await {
            Ok(Some(user)) => Ok(Some(UserReadDto::from(user))),
            Ok(None) => Ok(None),
            Err(e) => {
                Err(ApiError::DbError(DbError::SomethingWentWrong(format!("Failed to find user by username: {}", e))))
            }
        }
    }
}

#[cfg(test)]
mod find_by_username_service_tests {
    use super::*;
    use crate::entity::user::UserStatus;
    use crate::factory::user_factory::UserFactory;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::service::user_service::UserService;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_success() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        let expected_user = UserFactory::fake_user();
        let username = expected_user.username.clone();
        let expected_dto = UserReadDto::from(expected_user.clone());

        mock_repo
            .expect_find_by_username()
            .with(eq(username.clone()))
            .times(1)
            .returning(move |_| {
                let user = expected_user.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = service.find_by_username_internal(username).await;

        // Assert
        assert!(result.is_ok());
        let dto_option = result.unwrap();
        assert!(dto_option.is_some());
        let dto = dto_option.unwrap();
        assert_eq!(dto, expected_dto);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_not_found() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        let non_existent_username = "nonexistent_user".to_string();

        mock_repo
            .expect_find_by_username()
            .with(eq(non_existent_username.clone()))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(None) }));

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = service.find_by_username_internal(non_existent_username).await;

        // Assert
        assert!(result.is_ok());
        let dto_option = result.unwrap();
        assert!(dto_option.is_none());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_database_error() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        let test_username = "test_username".to_string();

        mock_repo
            .expect_find_by_username()
            .with(eq(test_username.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(sqlx::Error::PoolClosed) })
            });

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act
        let result = service.find_by_username_internal(test_username).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_with_factory_users() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        
        // Create multiple users using factory
        let user1 = UserFactory::unique_fake_user("mock_find_by_username_1", UserStatus::Active);
        let user2 = UserFactory::unique_fake_user("mock_find_by_username_2", UserStatus::Active);
        let user3 = UserFactory::unique_fake_user("mock_find_by_username_3", UserStatus::Active);

        let expected_dto1 = UserReadDto::from(user1.clone());
        let expected_dto2 = UserReadDto::from(user2.clone());
        let expected_dto3 = UserReadDto::from(user3.clone());

        // Setup expectations for each user
        let username1 = user1.username.clone();
        let username2 = user2.username.clone();
        let username3 = user3.username.clone();

        mock_repo
            .expect_find_by_username()
            .with(eq(username1.clone()))
            .times(1)
            .returning(move |_| {
                let user = user1.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        mock_repo
            .expect_find_by_username()
            .with(eq(username2.clone()))
            .times(1)
            .returning(move |_| {
                let user = user2.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        mock_repo
            .expect_find_by_username()
            .with(eq(username3.clone()))
            .times(1)
            .returning(move |_| {
                let user = user3.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act & Assert for user1
        let result1 = service.find_by_username_internal(username1).await;
        assert!(result1.is_ok());
        let dto1 = result1.unwrap().unwrap();
        assert_eq!(dto1, expected_dto1);

        // Act & Assert for user2
        let result2 = service.find_by_username_internal(username2).await;
        assert!(result2.is_ok());
        let dto2 = result2.unwrap().unwrap();
        assert_eq!(dto2, expected_dto2);

        // Act & Assert for user3
        let result3 = service.find_by_username_internal(username3).await;
        assert!(result3.is_ok());
        let dto3 = result3.unwrap().unwrap();
        assert_eq!(dto3, expected_dto3);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_with_different_statuses() {
        // Arrange
        let mut mock_repo = MockUserRepositoryTrait::new();
        
        // Test with different user statuses
        let active_user = UserFactory::unique_fake_user("active_username", UserStatus::Active);
        let inactive_user = UserFactory::unique_fake_user("inactive_username", UserStatus::Deleted);

        let active_dto = UserReadDto::from(active_user.clone());
        let inactive_dto = UserReadDto::from(inactive_user.clone());

        let active_username = active_user.username.clone();
        let inactive_username = inactive_user.username.clone();

        mock_repo
            .expect_find_by_username()
            .with(eq(active_username.clone()))
            .times(1)
            .returning(move |_| {
                let user = active_user.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        mock_repo
            .expect_find_by_username()
            .with(eq(inactive_username.clone()))
            .times(1)
            .returning(move |_| {
                let user = inactive_user.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        let service = UserService {
            user_repo: Arc::new(mock_repo),
        };

        // Act & Assert for active user
        let active_result = service.find_by_username_internal(active_username).await;
        assert!(active_result.is_ok());
        let active_found = active_result.unwrap().unwrap();
        assert_eq!(active_found, active_dto);
        assert_eq!(active_found.user_status, UserStatus::Active);

        // Act & Assert for inactive user
        let inactive_result = service.find_by_username_internal(inactive_username).await;
        assert!(inactive_result.is_ok());
        let inactive_found = inactive_result.unwrap().unwrap();
        assert_eq!(inactive_found, inactive_dto);
        assert_eq!(inactive_found.user_status, UserStatus::Deleted);
    }
}