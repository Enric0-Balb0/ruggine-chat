use crate::dto::invitation_dto::InvitationReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::invitation_error::InvitationError;
use crate::service::invitation_service::InvitationService;

impl InvitationService {
    pub async fn find_by_id_internal(&self, id: i32, user_id: i32) -> Result<InvitationReadDto, ApiError> {
        match self.invitation_repo.find_by_id(id, user_id).await {
            Ok(invitation) => Ok(InvitationReadDto::from(invitation)),
            Err(sqlx::Error::RowNotFound) => {
                Err(ApiError::InvitationError(InvitationError::InvitationNotFound))
            }
            Err(e) => {
                Err(ApiError::DbError(DbError::SomethingWentWrong(format!("Failed to find invitation: {}", e))))
            }
        }
    }
}

#[cfg(test)]
mod invitation_service_find_by_id_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use std::sync::Arc;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_internal_success() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let invitation_id = 1;
        let user_id = 1;
        let expected_invitation = InvitationFactory::fake_invitation();
        let expected_invitation_clone = expected_invitation.clone();

        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id), eq(user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = expected_invitation_clone.clone();
                Box::pin(async move { Ok(invitation) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );

        // Act
        let result = service.find_by_id_internal(invitation_id, user_id).await;

        // Assert
        assert!(result.is_ok());
        let invitation_read_dto = result.unwrap();
        assert_eq!(invitation_read_dto, InvitationReadDto::from(expected_invitation));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_internal_invitation_not_found() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let invitation_id = -1;
        let user_id = -1;

        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id), eq(user_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(sqlx::Error::RowNotFound) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );

        // Act
        let result = service.find_by_id_internal(invitation_id, user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationNotFound) => {
                // Expected error
            }
            _ => panic!("Expected InvitationNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_internal_database_error() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let invitation_id = 1;
        let user_id = 1;

        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id), eq(user_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(sqlx::Error::PoolClosed) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );

        // Act
        let result = service.find_by_id_internal(invitation_id, user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(_) => {
                // Expected error
            }
            _ => panic!("Expected DbError"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_internal_different_invitation() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let invitation_id = 5;
        let user_id = 4;
        let expected_invitation = InvitationFactory::fake_invitation_from_ids(3, user_id, 2);
        let expected_invitation_clone = expected_invitation.clone();

        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id), eq(user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = expected_invitation_clone.clone();
                Box::pin(async move { Ok(invitation) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );

        // Act
        let result = service.find_by_id_internal(invitation_id, user_id).await;

        // Assert
        assert!(result.is_ok());
        let invitation_read_dto = result.unwrap();
        assert_eq!(invitation_read_dto.from_user_id, 3);
        assert_eq!(invitation_read_dto.to_user_id, 4);
        assert_eq!(invitation_read_dto.group_chat_id, 2);
    }
}