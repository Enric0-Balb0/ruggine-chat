#[cfg(test)]
mod group_membership_create_checked_tests {
    use std::sync::Arc;
    use crate::common::{
        get_database, create_test_user, create_test_group_chat,
        create_test_invitation, cleanup_user, cleanup_group_chat,
        cleanup_group_membership
    };
    use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
    use ruggine_server::dto::group_membership_dto::GroupMembershipCreateDto;
    use ruggine_server::error::api_error::ApiError;
    use ruggine_server::error::group_membership_error::GroupMembershipError;
    use ruggine_server::repository::invitation_repository::InvitationRepository;
    use ruggine_server::service::group_chat_service::GroupChatService;
    use ruggine_server::service::invitation_service::InvitationService;
    use ruggine_server::service::user_service::UserService;
    use crate::cleanup_invitation;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_success() {
        // Arrange: setup users, group, and a pending invitation
        let db = get_database().await;

        let service = Arc::new(GroupMembershipService::new(&db));
        let invitation_service = InvitationService::with(
            Arc::new(InvitationRepository::new(&db)),
            Arc::new(GroupChatService::new(&db)),
            Arc::new(UserService::new(&db)),
            service.clone(),
        );
        service.set_invitation_service(Arc::new(invitation_service.clone()));

        let (from_user, _) = create_test_user("internal_success_from").await;
        let (to_user, _) = create_test_user("internal_success_to").await;
        let group = create_test_group_chat("internal_success_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group.id).await;

        let dto = GroupMembershipCreateDto {
            invitation_id: invitation.id,
        };

        // Act: Call create_checked
        let result = service.create_checked(dto.clone(), to_user.id).await;

        // Assert: Should succeed
        assert!(result.is_ok(), "Expected success, got error: {:?}", result);
        let membership = result.unwrap();
        assert_eq!(membership.group_chat_id, group.id);
        assert_eq!(membership.user_id, to_user.id);

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    // TODO
    /*#[tokio_shared_rt::test(shared)]
    async fn test_create_checked_fails_if_invitation_not_pending() {
        // Arrange: create expired invitation
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        let (from_user, _) = create_test_user("expired_inv_from").await;
        let (to_user, _) = create_test_user("expired_inv_to").await;
        let group = create_test_group_chat("expired_inv_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group.id).await;

        expire_invitation(invitation.id).await;

        let dto = GroupMembershipCreateDto {
            invitation_id: invitation.id,
        };

        // Act
        let result = service.create_checked(dto, to_user.id).await;

        // Assert: Should return PendingInvitationNotFound error
        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::PendingInvitationNotFound))
        ));

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    } */

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_fails_if_invitation_does_not_exist() {
        // Arrange
        let db = get_database().await;

        let service = Arc::new(GroupMembershipService::new(&db));
        let invitation_service = InvitationService::with(
            Arc::new(InvitationRepository::new(&db)),
            Arc::new(GroupChatService::new(&db)),
            Arc::new(UserService::new(&db)),
            service.clone(),
        );
        service.set_invitation_service(Arc::new(invitation_service.clone()));

        let (user, _) = create_test_user("nonexistent_inv_user").await;

        let dto = GroupMembershipCreateDto {
            invitation_id: -1, // Non-existent ID
        };

        // Act
        let result = service.create_checked(dto, user.id).await;

        // Assert
        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::PendingInvitationNotFound))
        ));

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_fails_if_user_already_in_group() {
        // Arrange: Create user, group, valid invitation, and create membership first
        let db = get_database().await;

        let service = Arc::new(GroupMembershipService::new(&db));
        let invitation_service = InvitationService::with(
            Arc::new(InvitationRepository::new(&db)),
            Arc::new(GroupChatService::new(&db)),
            Arc::new(UserService::new(&db)),
            service.clone(),
        );
        service.set_invitation_service(Arc::new(invitation_service.clone()));

        let (owner, _) = create_test_user("duplicate_owner_internal").await;
        let (member, _) = create_test_user("duplicate_member_internal").await;
        let group = create_test_group_chat("duplicate_group_internal", owner.id).await;
        let invitation = create_test_invitation(owner.id, member.id, group.id).await;

        let dto = GroupMembershipCreateDto {
            invitation_id: invitation.id,
        };

        // First creation
        let first_result = service.create_checked(dto.clone(), member.id).await;
        assert!(first_result.is_ok());

        // Second attempt (should fail)
        let second_result = service.create_checked(dto.clone(), member.id).await;
        assert!(matches!(
            second_result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyInGroup))
        ));

        // Cleanup
        cleanup_group_membership(first_result.unwrap().id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(owner.email).await;
        cleanup_user(member.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_invalid_invitation_id() {
        // Arrange: Use valid user but fake invitation ID (to simulate FK violation)
        let db = get_database().await;

        let service = Arc::new(GroupMembershipService::new(&db));
        let invitation_service = InvitationService::with(
            Arc::new(InvitationRepository::new(&db)),
            Arc::new(GroupChatService::new(&db)),
            Arc::new(UserService::new(&db)),
            service.clone(),
        );
        service.set_invitation_service(Arc::new(invitation_service.clone()));

        let (user, _) = create_test_user("fk_violation_user").await;

        let dto = GroupMembershipCreateDto {
            invitation_id: -1, // Invalid FK reference
        };

        // Act
        let result = service.create_checked(dto, user.id).await;

        // Assert: Expect DbError with ForeignKeyViolation
        match result {
            Ok(val) => panic!("Expected error but got success: {:?}", val),
            Err(ApiError::GroupMembershipError(GroupMembershipError::PendingInvitationNotFound)) => {
                // Success
            },
            Err(other) => panic!("Unexpected error type: {:?}", other),
        }

        // Cleanup
        cleanup_user(user.email).await;
    }
}
