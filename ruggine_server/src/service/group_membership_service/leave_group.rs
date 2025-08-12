use crate::dto::group_membership_dto::{GroupMembershipReadDto, LeaveGroupMembershipDto};
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::db_error::DbError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    pub async fn leave_group_internal(&self, payload: LeaveGroupMembershipDto, auth_user_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        // First verify the membership exists and belongs to the authenticated user
        let membership = match self
            .group_membership_repo
            .find_by_id_and_user_id(payload.id, auth_user_id)
            .await
        {
            Ok(membership) => membership,
            Err(sqlx::Error::RowNotFound) => {
                return Err(ApiError::GroupMembershipError(
                    GroupMembershipError::GroupMembershipNotFound,
                ))
            }
            Err(e) => {
                return Err(ApiError::DbError(DbError::SomethingWentWrong(e.to_string())))
            }
        };

        // Check if the user has already left the group
        if membership.membership_status == MembershipStatus::Left {
            return Err(ApiError::GroupMembershipError(
                GroupMembershipError::UserAlreadyLeftGroup,
            ));
        }

        // Convert the DTO to an update entity
        let update_membership = payload.to_update_group_membership();

        // Update the membership to set left status
        match self.group_membership_repo.update(update_membership).await {
            Ok(()) => {
                // Retrieve the updated membership to return it
                match self
                    .group_membership_repo
                    .find_by_id_and_user_id(payload.id, auth_user_id)
                    .await
                {
                    Ok(updated_membership) => Ok(GroupMembershipReadDto::from(updated_membership)),
                    Err(e) => Err(ApiError::DbError(DbError::SomethingWentWrong(
                        format!("Failed to retrieve updated membership: {}", e)
                    ))),
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                Err(ApiError::GroupMembershipError(
                    GroupMembershipError::GroupMembershipNotFound,
                ))
            }
            Err(e) => {
                // Handle specific database errors
                let error_message = e.to_string();
                if error_message.contains("23503") {
                    // Foreign key violation
                    Err(ApiError::DbError(DbError::ForeignKeyViolation(error_message)))
                } else {
                    Err(ApiError::DbError(DbError::SomethingWentWrong(
                        format!("Failed to leave group: {}", e)
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod group_membership_service_leave_group_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::entity::group_membership::{MemberRole, MembershipStatus};
    use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
    use crate::service::group_membership_service::GroupMembershipServiceTrait;
    use chrono::Utc;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::service::invitation_service::invitation_service_trait::MockInvitationServiceTrait;
    use std::sync::{Arc};

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_internal_success() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let membership_id = 10;
        
        let leave_dto = LeaveGroupMembershipDto { 
            id: membership_id 
        };

        let existing_membership = GroupMembershipWithInvitationRow {
            id: membership_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 1,
            user_id: auth_user_id,
            group_chat_id: 1,
        };

        let updated_membership = GroupMembershipWithInvitationRow {
            id: membership_id,
            role: MemberRole::Member,
            joined_at: existing_membership.joined_at,
            left_at: Some(Utc::now()),
            membership_status: MembershipStatus::Left,
            invitation_id: 1,
            user_id: auth_user_id,
            group_chat_id: 1,
        };

        // Mock finding existing membership
        mock_group_membership_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let membership = existing_membership.clone();
                Box::pin(async move { Ok(membership) })
            });

        // Mock successful update
        mock_group_membership_repo
            .expect_update()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(()) })
            });

        // Mock finding updated membership
        mock_group_membership_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let membership = updated_membership.clone();
                Box::pin(async move { Ok(membership) })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_group_membership_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act
        let result = service.leave_group_internal(leave_dto, auth_user_id).await;

        // Assert
        assert!(result.is_ok());
        let membership_dto = result.unwrap();
        assert_eq!(membership_dto.id, membership_id);
        assert_eq!(membership_dto.user_id, auth_user_id);
        assert_eq!(membership_dto.membership_status, MembershipStatus::Left);
        assert!(membership_dto.left_at.is_some());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_internal_membership_not_found() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let membership_id = 999;
        
        let leave_dto = LeaveGroupMembershipDto { 
            id: membership_id 
        };

        // Mock membership not found
        mock_group_membership_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(sqlx::Error::RowNotFound) })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_group_membership_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act
        let result = service.leave_group_internal(leave_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            e => panic!("Expected GroupMembershipNotFound error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_internal_user_already_left() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let membership_id = 10;
        
        let leave_dto = LeaveGroupMembershipDto { 
            id: membership_id 
        };

        let already_left_membership = GroupMembershipWithInvitationRow {
            id: membership_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: Some(Utc::now()),
            membership_status: MembershipStatus::Left, // Already left
            invitation_id: 1,
            user_id: auth_user_id,
            group_chat_id: 1,
        };

        // Mock finding membership that already left
        mock_group_membership_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let membership = already_left_membership.clone();
                Box::pin(async move { Ok(membership) })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_group_membership_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act
        let result = service.leave_group_internal(leave_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyLeftGroup) => {
                // Expected error
            }
            e => panic!("Expected UserAlreadyLeftGroup error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_internal_update_fails() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let membership_id = 10;
        
        let leave_dto = LeaveGroupMembershipDto { 
            id: membership_id 
        };

        let existing_membership = GroupMembershipWithInvitationRow {
            id: membership_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 1,
            user_id: auth_user_id,
            group_chat_id: 1,
        };

        // Mock finding existing membership
        mock_group_membership_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let membership = existing_membership.clone();
                Box::pin(async move { Ok(membership) })
            });

        // Mock update failure
        mock_group_membership_repo
            .expect_update()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(sqlx::Error::RowNotFound) })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_group_membership_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act
        let result = service.leave_group_internal(leave_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            e => panic!("Expected GroupMembershipNotFound error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_internal_database_error() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let membership_id = 10;
        
        let leave_dto = LeaveGroupMembershipDto { 
            id: membership_id 
        };

        let existing_membership = GroupMembershipWithInvitationRow {
            id: membership_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 1,
            user_id: auth_user_id,
            group_chat_id: 1,
        };

        // Mock finding existing membership
        mock_group_membership_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let membership = existing_membership.clone();
                Box::pin(async move { Ok(membership) })
            });

        // Mock database error on update
        mock_group_membership_repo
            .expect_update()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { 
                    Err(sqlx::Error::Database(Box::new(crate::utils::mock_database_error::MockDatabaseError::new("23000".to_string()))))
                })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_group_membership_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act
        let result = service.leave_group_internal(leave_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error
            }
            e => panic!("Expected DbError::SomethingWentWrong error, got: {:?}", e),
        }
    }
}
