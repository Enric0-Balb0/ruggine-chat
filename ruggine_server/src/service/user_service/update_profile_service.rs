use crate::dto::user_dto::UserReadDto;
use crate::entity::user::UpdateUser;
use crate::error::{api_error::ApiError, db_error::DbError, user_error::UserError};
use crate::service::user_service::UserService;
use sqlx::Error as SqlxError;
use tracing::error;

impl UserService {
    pub async fn update_user_profile_internal(&self, user_id: i32, update_user: UpdateUser) -> Result<UserReadDto, ApiError> {
        let updated_user = self.user_repo.update_profile(user_id, update_user).await;

        match updated_user {
            Ok(user) => Ok(UserReadDto::from(user)),
            Err(e) => match e {
                SqlxError::Database(db_err) => {
                    match db_err.code() {
                        Some(code) if code == "23505" => {
                            Err(DbError::UniqueConstraintViolation(db_err.to_string()))?
                        }
                        _ => Err(DbError::SomethingWentWrong(db_err.to_string()))?
                    }
                }
                SqlxError::RowNotFound => {
                    Err(UserError::UserNotFound)?
                }
                _ => {
                    error!("Update user profile error: {}", e.to_string());
                    Err(DbError::SomethingWentWrong(e.to_string()))?
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::user_factory::UserFactory;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::service::user_service::UserService;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_user_profile_success() {
        // Arrange
        let user = UserFactory::fake_user();
        let update_user = UpdateUser::from_dto(UserFactory::fake_user_update_dto());
        let expected_user = user.clone();

        let mut mock_repo = MockUserRepositoryTrait::new();
        mock_repo
            .expect_update_profile()
            .with(eq(user.id), function(|update: &UpdateUser| update.has_updates()))
            .times(1)
            .returning(move |_, _| {
                let user_clone = expected_user.clone();
                Box::pin(async move { Ok(user_clone) })
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act
        let result = service.update_user_profile_internal(user.id, update_user).await;

        // Assert
        assert!(result.is_ok());
        let user_dto = result.unwrap();
        assert_eq!(user_dto.id, user.id);
    }

    #[tokio::test]
    async fn test_update_user_profile_user_not_found() {
        // Arrange
        let user_id = 999;
        let update_user = UpdateUser::from_dto(UserFactory::fake_user_update_dto());

        let mut mock_repo = MockUserRepositoryTrait::new();
        mock_repo
            .expect_update_profile()
            .times(1)
            .returning(|_, _| {
                Box::pin(async {
                    Err(SqlxError::RowNotFound)
                })
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act
        let result = service.update_user_profile_internal(user_id, update_user).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::UserError(UserError::UserNotFound) => {
                // Success
            },
            _ => panic!("Expected UserNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_user_profile_database_error() {
        // Arrange
        let user = UserFactory::fake_user();
        let update_user = UpdateUser::from_dto(UserFactory::fake_user_update_dto());

        let mut mock_repo = MockUserRepositoryTrait::new();
        mock_repo
            .expect_update_profile()
            .times(1)
            .returning(|_, _| {
                Box::pin(async {
                    Err(SqlxError::Configuration("Connection error".into()))
                })
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act
        let result = service.update_user_profile_internal(user.id, update_user).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(_) => {},
            _ => panic!("Expected DatabaseError"),
        }
    }

    #[tokio::test]
    async fn test_update_user_profile_partial_update() {
        // Arrange
        let user = UserFactory::fake_user();
        let update_user = UpdateUser::from_dto(UserFactory::fake_user_update_dto_partial());
        let expected_user = user.clone();

        let mut mock_repo = MockUserRepositoryTrait::new();
        mock_repo
            .expect_update_profile()
            .with(eq(user.id), function(|update: &UpdateUser| {
                update.first_name.is_some() && update.last_name.is_none()
            }))
            .times(1)
            .returning(move |_, _| {
                let user_clone = expected_user.clone();
                Box::pin(async move { Ok(user_clone) })
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act
        let result = service.update_user_profile_internal(user.id, update_user).await;

        // Assert
        assert!(result.is_ok());
    }
}
