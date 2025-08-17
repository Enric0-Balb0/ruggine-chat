use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::dto::invitation_dto::{InvitationCreateDto, InvitationReadDto};
use crate::entity::group_membership::{MemberRole, MembershipStatus};
use crate::error::api_error::ApiError;
use crate::error::invitation_error::InvitationError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::user_error::UserError;
use crate::entity::invitation::NewInvitation;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::service::invitation_service::InvitationService;

impl InvitationService {
    pub async fn send_internal(&self, invitation: InvitationCreateDto, from_user_id: i32) -> Result<InvitationReadDto, ApiError> {
        // Verify that the recipient user exists
        if let Err(_) = self.user_service.find_by_id(invitation.to_user_id).await {
            return Err(ApiError::InvitationError(InvitationError::InvitedUserNotFound));
        }

        // Verify that the group chat exists and get its details
        let group_chat = match self.group_chat_service.find_by_id(invitation.group_chat_id).await {
            Ok(group) => group,
            Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound)) => {
                return Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound));
            }
            Err(e) => return Err(e),
        };

        //  Check if user is already in the group chat
        match self.group_membership_service
            .find_active_by_user_id_and_group_id(invitation.to_user_id, group_chat.id)
            .await
        {
            Ok(membership) => {
                // L'utente è già nel gruppo
                if membership.membership_status == MembershipStatus::Active {
                    return Err(ApiError::InvitationError(InvitationError::UserAlreadyInGroup));
                }
            }
            Err(err) => {
                // Se l'errore NON è che la membership non è stata trovata, propaga l'errore
                if !matches!(err, ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) {
                    return Err(err);
                }
                // Se invece è NotFound, va tutto bene: l'utente non è ancora nel gruppo
            }
        }


        // TODO: Check from membership if you are admin
        match self.group_membership_service
            .find_active_by_user_id_and_group_id(from_user_id, group_chat.id)
            .await
        {
            Ok(membership) => {
                // If the from_user is the group he must be admin
                if membership.role != MemberRole::Admin || membership.membership_status != MembershipStatus::Active {
                    return Err(
                        ApiError::InvitationError(
                            InvitationError::UserNotAuthorized("Sender is not an admin of the group".to_string())
                        )
                    );
                }
                // Otherwise it is okay
            }
            Err(err) => {
                // Se l'errore NON è che la membership non è stata trovata, propaga l'errore
                if matches!(err, ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) {
                    return Err(
                        ApiError::InvitationError(
                            InvitationError::UserNotAuthorized("Sender is not in the group".to_string())
                        )
                    );
                }
                return Err(err);
            }
        }

        // Check if there's already a pending invitation between these users for this group
        match self.invitation_repo.find_pending_invitation_between_users(from_user_id, invitation.to_user_id, invitation.group_chat_id).await {
            Ok(Some(_)) => {
                return Err(ApiError::InvitationError(InvitationError::AlreadyInvitationPending));
            }
            Ok(None) => {
                // Not pending invitation, we can proceed
            }
            Err(e) => {
                return Err(ApiError::DbError(DbError::SomethingWentWrong(e.to_string())));
            }
        }

        // Create the new invitation
        let new_invitation = NewInvitation {
            from_user_id,
            to_user_id: invitation.to_user_id,
            group_chat_id: invitation.group_chat_id,
            role_at_join: invitation.role_at_join,
        };

        // Insert the invitation
        match self.invitation_repo.insert(new_invitation).await {
            Ok(invitation_id) => {
                // Retrieve the created invitation to return it
                match self.invitation_repo.find_by_id_and_user_id(invitation_id, from_user_id).await {
                    Ok(created_invitation) => Ok(InvitationReadDto::from(created_invitation)),
                    Err(e) => Err(ApiError::DbError(DbError::SomethingWentWrong(format!("Failed to retrieve created invitation: {}", e)))),
                }
            }
            Err(e) => {
                // Handle specific database errors
                let error_message = e.to_string();
                if error_message.contains("unique_pending_invitation") || error_message.contains("23505") {
                    Err(ApiError::InvitationError(InvitationError::AlreadyInvitationPending))
                } else if error_message.contains("foreign key") || error_message.contains("23503") {
                    if error_message.contains("to_user_id") {
                        Err(ApiError::InvitationError(InvitationError::InvitedUserNotFound))
                    } else {
                        Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound))
                    }
                } else {
                    Err(ApiError::DbError(DbError::SomethingWentWrong(format!("Failed to create invitation: {}", e))))
                }
            }
        }
    }
}

#[cfg(test)]
mod invitation_service_send_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::entity::group_membership::{MemberRole, MembershipStatus};
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::factory::user_factory::UserFactory;
    use crate::dto::group_chat_dto::GroupChatReadDto;
    use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
    use chrono::Utc;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use std::sync::Arc;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_send_internal_success() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let from_user_id = 1;
        let to_user_id = 2;
        let group_chat_id = 1;
        
        let invitation_dto = InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: MemberRole::Member,
        };

        let group_chat_dto = GroupChatReadDto {
            id: group_chat_id,
            name: "Test Group".to_string(),
            description: "Test Description".to_string(),
            created_by: from_user_id, // User is admin
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut user_dto = UserFactory::fake_read_user_dto();
        user_dto.id = to_user_id;

        let expected_invitation = InvitationFactory::fake_invitation_from_ids(from_user_id, to_user_id, group_chat_id);
        let expected_invitation_clone = expected_invitation.clone();

        // Mock user service to return the user exists
        mock_user_service
            .expect_find_by_id()
            .with(eq(to_user_id))
            .times(1)
            .returning(move |_| {
                let user = user_dto.clone();
                Box::pin(async move { Ok(user) })
            });

        // Mock group chat service to return the group exists and user is admin
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .times(1)
            .returning(move |_| {
                let group = group_chat_dto.clone();
                Box::pin(async move { Ok(group) })
            });

        // Mock group membership service - recipient is NOT already in the group
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) })
            });

        // Mock group membership service - sender IS in the group and is admin
        let admin_membership_row = GroupMembershipWithInvitationRow {
            id: 1,
            role: MemberRole::Admin,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 100,
            user_id: from_user_id,
            group_chat_id,
        };
        let admin_membership_dto = crate::dto::group_membership_dto::GroupMembershipReadDto::from(admin_membership_row);
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(from_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                let membership = admin_membership_dto.clone();
                Box::pin(async move { Ok(membership) })
            });

        // Mock no pending invitation exists
        mock_invitation_repo
            .expect_find_pending_invitation_between_users()
            .with(eq(from_user_id), eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _, _| {
                Box::pin(async move { Ok(None) })
            });

        // Mock successful insertion
        mock_invitation_repo
            .expect_insert()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(expected_invitation.id) })
            });

        // Mock successful retrieval of created invitation
        mock_invitation_repo
            .expect_find_by_id_and_user_id()
            .with(eq(expected_invitation.id), eq(from_user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = expected_invitation_clone.clone();
                Box::pin(async move { Ok(invitation) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.send_internal(invitation_dto, from_user_id).await;

        // Assert
        assert!(result.is_ok());
        let invitation_read_dto = result.unwrap();
        assert_eq!(invitation_read_dto.from_user_id, from_user_id);
        assert_eq!(invitation_read_dto.to_user_id, to_user_id);
        assert_eq!(invitation_read_dto.group_chat_id, group_chat_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_internal_user_not_found() {
        // Arrange
        let mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let from_user_id = 1;
        let to_user_id = 999;
        let group_chat_id = 1;
        
        let invitation_dto = InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: MemberRole::Member,
        };

        // Mock user service to return user not found
        mock_user_service
            .expect_find_by_id()
            .with(eq(to_user_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(ApiError::UserError(UserError::UserNotFound)) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.send_internal(invitation_dto, from_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitedUserNotFound) => {
                // Expected error
            }
            _ => panic!("Expected InvitedUserNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_internal_user_not_authorized() {
        // Arrange
        let mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let from_user_id = 1;
        let to_user_id = 2;
        let group_chat_id = 1;
        let admin_user_id = 3; // Different from from_user_id
        
        let invitation_dto = InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: MemberRole::Admin,
        };

        let group_chat_dto = GroupChatReadDto {
            id: group_chat_id,
            name: "Test Group".to_string(),
            description: "Test Description".to_string(),
            created_by: admin_user_id, // Different user is admin
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut user_dto = UserFactory::fake_read_user_dto();
        user_dto.id = to_user_id;

        // Mock user service to return the user exists
        mock_user_service
            .expect_find_by_id()
            .with(eq(to_user_id))
            .times(1)
            .returning(move |_| {
                let user = user_dto.clone();
                Box::pin(async move { Ok(user) })
            });

        // Mock group chat service to return the group exists but user is not admin
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .times(1)
            .returning(move |_| {
                let group = group_chat_dto.clone();
                Box::pin(async move { Ok(group) })
            });

        // Mock group membership service - recipient is NOT already in the group
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) })
            });

        // Mock group membership service - sender is in the group but NOT an admin (just a member)
        let member_membership_row = GroupMembershipWithInvitationRow {
            id: 1,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 101,
            user_id: from_user_id,
            group_chat_id,
        };
        let member_membership_dto = crate::dto::group_membership_dto::GroupMembershipReadDto::from(member_membership_row);
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(from_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                let membership = member_membership_dto.clone();
                Box::pin(async move { Ok(membership) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.send_internal(invitation_dto, from_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized(_)) => {
                // Expected error
            }
            other => panic!("Expected UserNotAuthorized error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_internal_already_pending_invitation() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let from_user_id = 1;
        let to_user_id = 2;
        let group_chat_id = 1;
        
        let invitation_dto = InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: MemberRole::Admin,
        };

        let group_chat_dto = GroupChatReadDto {
            id: group_chat_id,
            name: "Test Group".to_string(),
            description: "Test Description".to_string(),
            created_by: from_user_id, // User is admin
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut user_dto = UserFactory::fake_read_user_dto();
        user_dto.id = to_user_id;

        let existing_invitation = InvitationFactory::fake_invitation_from_ids(from_user_id, to_user_id, group_chat_id);
        let existing_invitation_clone = existing_invitation.clone();

        // Mock user service to return the user exists
        mock_user_service
            .expect_find_by_id()
            .with(eq(to_user_id))
            .times(1)
            .returning(move |_| {
                let user = user_dto.clone();
                Box::pin(async move { Ok(user) })
            });

        // Mock group chat service to return the group exists and user is admin
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .times(1)
            .returning(move |_| {
                let group = group_chat_dto.clone();
                Box::pin(async move { Ok(group) })
            });

        // Mock group membership service - recipient is NOT already in the group
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) })
            });

        // Mock group membership service - sender IS in the group and is admin
        let admin_membership_row2 = GroupMembershipWithInvitationRow {
            id: 2,
            role: MemberRole::Admin,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 102,
            user_id: from_user_id,
            group_chat_id,
        };
        let admin_membership_dto2 = crate::dto::group_membership_dto::GroupMembershipReadDto::from(admin_membership_row2);
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(from_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                let membership = admin_membership_dto2.clone();
                Box::pin(async move { Ok(membership) })
            });

        // Mock existing pending invitation
        mock_invitation_repo
            .expect_find_pending_invitation_between_users()
            .with(eq(from_user_id), eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _, _| {
                let invitation = existing_invitation_clone.clone();
                Box::pin(async move { Ok(Some(invitation)) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.send_internal(invitation_dto, from_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::AlreadyInvitationPending) => {
                // Expected error
            }
            _ => panic!("Expected AlreadyInvitationPending error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_internal_user_already_in_group_active() {
        // Arrange
        let mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let from_user_id = 1;
        let to_user_id = 2;
        let group_chat_id = 1;
        
        let invitation_dto = InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: MemberRole::Member,
        };

        let group_chat_dto = GroupChatReadDto {
            id: group_chat_id,
            name: "Test Group".to_string(),
            description: "Test Description".to_string(),
            created_by: from_user_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut user_dto = UserFactory::fake_read_user_dto();
        user_dto.id = to_user_id;

        // Mock user service to return the user exists
        mock_user_service
            .expect_find_by_id()
            .with(eq(to_user_id))
            .times(1)
            .returning(move |_| {
                let user = user_dto.clone();
                Box::pin(async move { Ok(user) })
            });

        // Mock group chat service to return the group exists
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .times(1)
            .returning(move |_| {
                let group = group_chat_dto.clone();
                Box::pin(async move { Ok(group) })
            });

        // Mock group membership service - recipient IS already in the group with active status
        let active_membership_row = GroupMembershipWithInvitationRow {
            id: 1,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 100,
            user_id: to_user_id,
            group_chat_id,
        };
        let active_membership_dto = crate::dto::group_membership_dto::GroupMembershipReadDto::from(active_membership_row);
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                let membership = active_membership_dto.clone();
                Box::pin(async move { Ok(membership) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.send_internal(invitation_dto, from_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserAlreadyInGroup) => {
                // Expected error
            }
            other => panic!("Expected UserAlreadyInGroup error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_internal_sender_not_in_group() {
        // Arrange
        let mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let from_user_id = 1;
        let to_user_id = 2;
        let group_chat_id = 1;
        
        let invitation_dto = InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: MemberRole::Member,
        };

        let group_chat_dto = GroupChatReadDto {
            id: group_chat_id,
            name: "Test Group".to_string(),
            description: "Test Description".to_string(),
            created_by: 3, // Different user created the group
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut user_dto = UserFactory::fake_read_user_dto();
        user_dto.id = to_user_id;

        // Mock user service to return the user exists
        mock_user_service
            .expect_find_by_id()
            .with(eq(to_user_id))
            .times(1)
            .returning(move |_| {
                let user = user_dto.clone();
                Box::pin(async move { Ok(user) })
            });

        // Mock group chat service to return the group exists
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .times(1)
            .returning(move |_| {
                let group = group_chat_dto.clone();
                Box::pin(async move { Ok(group) })
            });

        // Mock group membership service - recipient is NOT already in the group
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) })
            });

        // Mock group membership service - sender is NOT in the group
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(from_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.send_internal(invitation_dto, from_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized(msg)) => {
                assert!(msg.contains("Sender is not in the group"));
            }
            other => panic!("Expected UserNotAuthorized error with 'not in the group' message, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_internal_sender_inactive_membership() {
        // Arrange
        let mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();

        let from_user_id = 1;
        let to_user_id = 2;
        let group_chat_id = 1;
        
        let invitation_dto = InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: MemberRole::Member,
        };

        let group_chat_dto = GroupChatReadDto {
            id: group_chat_id,
            name: "Test Group".to_string(),
            description: "Test Description".to_string(),
            created_by: from_user_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut user_dto = UserFactory::fake_read_user_dto();
        user_dto.id = to_user_id;

        // Mock user service to return the user exists
        mock_user_service
            .expect_find_by_id()
            .with(eq(to_user_id))
            .times(1)
            .returning(move |_| {
                let user = user_dto.clone();
                Box::pin(async move { Ok(user) })
            });

        // Mock group chat service to return the group exists
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .times(1)
            .returning(move |_| {
                let group = group_chat_dto.clone();
                Box::pin(async move { Ok(group) })
            });

        // Mock group membership service - recipient is NOT already in the group
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) })
            });

        // Mock group membership service - sender is in the group but has inactive status
        let inactive_admin_membership_row = GroupMembershipWithInvitationRow {
            id: 1,
            role: MemberRole::Admin,
            joined_at: Utc::now(),
            left_at: Some(Utc::now()), // Has left the group
            membership_status: MembershipStatus::Left,
            invitation_id: 100,
            user_id: from_user_id,
            group_chat_id,
        };
        let inactive_admin_membership_dto = crate::dto::group_membership_dto::GroupMembershipReadDto::from(inactive_admin_membership_row);
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(from_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| {
                let membership = inactive_admin_membership_dto.clone();
                Box::pin(async move { Ok(membership) })
            });

        let service = InvitationService::with(
            Arc::new(mock_invitation_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
            Arc::new(mock_group_membership_service),
        );

        // Act
        let result = service.send_internal(invitation_dto, from_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized(msg)) => {
                assert!(msg.contains("Sender is not an admin of the group"));
            }
            other => panic!("Expected UserNotAuthorized error with 'not an admin' message, got: {:?}", other),
        }
    }
}