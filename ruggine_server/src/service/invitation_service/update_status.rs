use crate::dto::invitation_dto::{InvitationUpdateStatusDto, InvitationUpdateResponseDto};
use crate::dto::group_membership_dto::GroupMembershipCreateDto;
use crate::error::api_error::ApiError;
use crate::error::invitation_error::InvitationError;
use crate::error::db_error::DbError;
use crate::entity::invitation::{UpdateInvitationStatus, InvitationStatus};
use crate::service::invitation_service::InvitationService;
use chrono::Utc;

impl InvitationService {
    pub async fn update_status_internal(&self, payload: InvitationUpdateStatusDto, auth_user_id: i32) -> Result<InvitationUpdateResponseDto, ApiError> {
        // Validate that the status is either accepted or rejected
        if let Err(e) = payload.validate_status() {
            return Err(ApiError::InvitationError(InvitationError::InvalidStatus(e)));
        }

        // Find the invitation to verify user authorization
        let invitation = match self.invitation_repo.find_by_id(payload.invitation_id).await {
            Ok(invitation) => invitation,
            Err(_) => return Err(ApiError::InvitationError(InvitationError::InvitationNotFound)),
        };

        // Verify that the current user is the recipient of the invitation
        if invitation.to_user_id != auth_user_id {
            return Err(ApiError::InvitationError(InvitationError::UserNotAuthorized));
        }

        // Verify that the invitation is still pending
        if invitation.status != InvitationStatus::Pending {
            return Err(ApiError::InvitationError(InvitationError::InvitationAlreadyResponded));
        }

        // Create update status entity
        let update_invitation_status = UpdateInvitationStatus {
            invitation_id: payload.invitation_id,
            status: payload.status.clone(),
            responded_at: Utc::now(),
        };

        // Update the invitation status
        let updated_invitation = match self.invitation_repo.update_status(payload.invitation_id, update_invitation_status).await {
            Ok(invitation) => invitation,
            Err(e) => return Err(ApiError::DbError(DbError::SomethingWentWrong(e.to_string()))),
        };

        // If the status is accepted, create a group membership
        if payload.status == InvitationStatus::Accepted {
            let membership_dto = GroupMembershipCreateDto {
                invitation_id: invitation.to_user_id,
            };

            if let Err(e) = self.group_membership_service.create_checked(membership_dto, auth_user_id).await {
                eprintln!("Failed to create group membership: {:?}", e);
                return Err(e);
            }
        }

        Ok(InvitationUpdateResponseDto::from(updated_invitation))
    }
}

/* #[cfg(test)]
mod invitation_service_update_status_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::dto::group_membership_dto::GroupMembershipReadDto;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use std::sync::Arc;
    use sqlx::Error as SqlxError;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_internal_success_accept() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let current_user_id = 2;
        let group_chat_id = 3;

        let pending_invitation = InvitationFactory::fake_invitation_from_ids(1, current_user_id, group_chat_id);

        let invitation_id = pending_invitation.id;
        let payload = InvitationUpdateStatusDto {
            invitation_id,
            status: InvitationStatus::Accepted,
        };

        let mut accepted_invitation = pending_invitation.clone();
        accepted_invitation.status = InvitationStatus::Accepted;
        accepted_invitation.responded_at = Some(Utc::now());

        let group_membership_dto = GroupMembershipReadDto::from(
            GroupMembershipFactory::fake_group_membership_from_ids(current_user_id, group_chat_id)
        );

        // Mock finding the invitation
        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id))
            .times(1)
            .returning(move |_| {
                let invitation = pending_invitation.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock updating the invitation status
        mock_invitation_repo
            .expect_update_status()
            .with(eq(invitation_id), function(|update: &UpdateInvitationStatus| {
                update.status == InvitationStatus::Accepted
            }))
            .times(1)
            .returning(move |_, _| {
                let invitation = accepted_invitation.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock creating group membership
        mock_group_membership_service
            .expect_create()
            .with(function(move |dto: &GroupMembershipCreateDto| {
                dto.invitation_id == current_user_id && dto.group_chat_id == group_chat_id
            }))
            .times(1)
            .returning(move |_| {
                let membership = group_membership_dto.clone();
                Box::pin(async move { Ok(membership) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.update_status_internal(payload, current_user_id).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, invitation_id);
        assert_eq!(response.status, InvitationStatus::Accepted);
        assert!(response.responded_at <= Utc::now());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_internal_success_reject() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let current_user_id = 2;
        let group_chat_id = 3;

        let pending_invitation = InvitationFactory::fake_invitation_from_ids(1, current_user_id, group_chat_id);

        let invitation_id = pending_invitation.id;
        let payload = InvitationUpdateStatusDto {
            invitation_id,
            status: InvitationStatus::Rejected,
        };

        let mut rejected_invitation = pending_invitation.clone();
        rejected_invitation.status = InvitationStatus::Rejected;
        rejected_invitation.responded_at = Some(Utc::now());

        // Mock finding the invitation
        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id))
            .times(1)
            .returning(move |_| {
                let invitation = pending_invitation.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock updating the invitation status
        mock_invitation_repo
            .expect_update_status()
            .with(eq(invitation_id), function(|update: &UpdateInvitationStatus| {
                update.status == InvitationStatus::Rejected
            }))
            .times(1)
            .returning(move |_, _| {
                let invitation = rejected_invitation.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Group membership service should not be called for rejected invitations
        mock_group_membership_service.expect_create().times(0);

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.update_status_internal(payload, current_user_id).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, invitation_id);
        assert_eq!(response.status, InvitationStatus::Rejected);
        assert!(response.responded_at <= Utc::now());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_internal_invitation_not_found() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let current_user_id = 2;
        let invitation_id = 999;
        
        let payload = InvitationUpdateStatusDto {
            invitation_id,
            status: InvitationStatus::Accepted,
        };

        // Mock invitation not found
        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id))
            .times(1)
            .returning(|_| Box::pin(async { Err(SqlxError::RowNotFound) }));

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.update_status_internal(payload, current_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationNotFound) => {
                // Success
            },
            _ => panic!("Expected InvitationNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_internal_user_not_authorized() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let current_user_id = 2;
        let wrong_user_id = 3; // Different user
        let invitation_id = 1;
        let group_chat_id = 1;
        
        let payload = InvitationUpdateStatusDto {
            invitation_id,
            status: InvitationStatus::Accepted,
        };

        let invitation = InvitationFactory::fake_invitation_from_ids(1, wrong_user_id, group_chat_id);

        // Mock finding the invitation for wrong user
        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id))
            .times(1)
            .returning(move |_| {
                let invitation = invitation.clone();
                Box::pin(async move { Ok(invitation) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.update_status_internal(payload, current_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized) => {
                // Success
            },
            _ => panic!("Expected UserNotAuthorized error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_internal_invitation_already_responded() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let current_user_id = 2;
        let invitation_id = 1;
        let group_chat_id = 1;
        
        let payload = InvitationUpdateStatusDto {
            invitation_id,
            status: InvitationStatus::Accepted,
        };

        let mut already_accepted_invitation = InvitationFactory::fake_invitation_from_ids(1, current_user_id, group_chat_id);
        already_accepted_invitation.status = InvitationStatus::Accepted;
        already_accepted_invitation.responded_at = Some(Utc::now());

        // Mock finding the already responded invitation
        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id))
            .times(1)
            .returning(move |_| {
                let invitation = already_accepted_invitation.clone();
                Box::pin(async move { Ok(invitation) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.update_status_internal(payload, current_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationAlreadyResponded) => {
                // Success
            },
            _ => panic!("Expected InvitationAlreadyResponded error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_internal_invalid_status_pending() {
        // Arrange
        let mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let current_user_id = 2;
        let invitation_id = 1;
        
        let payload = InvitationUpdateStatusDto {
            invitation_id,
            status: InvitationStatus::Pending, // Invalid status for update
        };

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.update_status_internal(payload, current_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvalidStatus(_)) => {
                // Success
            },
            _ => panic!("Expected InvalidStatus error"),
        }
    }
}

 */
