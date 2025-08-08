use crate::dto::invitation_dto::InvitationReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::service::invitation_service::InvitationService;
use crate::entity::invitation::UserInvitationFilter;

impl InvitationService {
    /// Find all invitations for a specific user (by default, only received invitations)
    pub async fn find_by_user_id_internal(&self, user_id: i32) -> Result<Vec<InvitationReadDto>, ApiError> {
        match self.invitation_repo.find_by_user_id(user_id, UserInvitationFilter::AsRecipient).await {
            Ok(invitations) => {
                let invitation_dtos: Vec<InvitationReadDto> = invitations
                    .into_iter()
                    .map(InvitationReadDto::from)
                    .collect();
                Ok(invitation_dtos)
            }
            Err(e) => {
                Err(ApiError::DbError(DbError::SomethingWentWrong(format!("Failed to find invitations for user: {}", e))))
            }
        }
    }
}

#[cfg(test)]
mod invitation_service_find_by_user_id_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use std::sync::Arc;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::entity::invitation::UserInvitationFilter;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_internal_success_as_recipient() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let user_id = 1;
        // Only received invitations (user is recipient)
        let invitation1 = InvitationFactory::fake_invitation_from_ids(10, user_id, 2); // received by user
        let invitation2 = InvitationFactory::fake_invitation_from_ids(11, user_id, 3); // received by user
        let expected_invitations = vec![invitation1.clone(), invitation2.clone()];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(UserInvitationFilter::AsRecipient))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let invitation_read_dtos = result.unwrap();
        assert_eq!(invitation_read_dtos.len(), 2);
        assert_eq!(invitation_read_dtos[0], InvitationReadDto::from(invitation1));
        assert_eq!(invitation_read_dtos[1], InvitationReadDto::from(invitation2));
        // Verify all are received invitations
        assert!(invitation_read_dtos.iter().all(|dto| dto.to_user_id == user_id));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_internal_empty_result() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let user_id = 999;
        let expected_invitations = vec![];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(UserInvitationFilter::AsRecipient))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let invitation_read_dtos = result.unwrap();
        assert_eq!(invitation_read_dtos.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_internal_database_error() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let user_id = 1;

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(UserInvitationFilter::AsRecipient))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(sqlx::Error::PoolClosed) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

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
    async fn test_find_by_user_id_internal_uses_correct_filter() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let user_id = 5;
        // Only invitations where user is recipient
        let invitation1 = InvitationFactory::fake_invitation_from_ids(20, user_id, 2); // received by user
        let invitation2 = InvitationFactory::fake_invitation_from_ids(21, user_id, 3); // received by user
        let expected_invitations = vec![invitation1.clone(), invitation2.clone()];
        let expected_invitations_clone = expected_invitations.clone();

        // Verify that the service uses AsRecipient filter by default
        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(UserInvitationFilter::AsRecipient))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let invitation_read_dtos = result.unwrap();
        assert_eq!(invitation_read_dtos.len(), 2);
        
        // Verify mapping from entity to DTO
        for (i, dto) in invitation_read_dtos.iter().enumerate() {
            assert_eq!(*dto, InvitationReadDto::from(expected_invitations[i].clone()));
            assert_eq!(dto.to_user_id, user_id); // All should be received by user
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_internal_dto_conversion() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let user_id = 7;
        let invitation = InvitationFactory::fake_invitation_from_ids(30, 4, user_id);
        let expected_invitations = vec![invitation.clone()];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(UserInvitationFilter::AsRecipient))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let invitation_read_dtos = result.unwrap();
        assert_eq!(invitation_read_dtos.len(), 1);
        
        let dto = &invitation_read_dtos[0];
        let expected_dto = InvitationReadDto::from(invitation);
        
        assert_eq!(dto.id, expected_dto.id);
        assert_eq!(dto.from_user_id, expected_dto.from_user_id);
        assert_eq!(dto.to_user_id, expected_dto.to_user_id);
        assert_eq!(dto.group_chat_id, expected_dto.group_chat_id);
        assert_eq!(dto.status, expected_dto.status);
        assert_eq!(dto.sent_at, expected_dto.sent_at);
        assert_eq!(dto.responded_at, expected_dto.responded_at);
    }
}
