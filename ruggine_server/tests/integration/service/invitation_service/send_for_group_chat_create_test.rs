use ruggine_server::service::invitation_service::{InvitationService, InvitationServiceTrait};
use ruggine_server::factory::invitation_factory::InvitationFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::invitation_error::InvitationError;
use ruggine_server::error::group_chat_error::GroupChatError;
use ruggine_server::entity::invitation::InvitationStatus;
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, cleanup_invitation};

#[cfg(test)]
mod invitation_service_send_for_group_chat_create_integration_tests {
    use ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto;
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use crate::{clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id, create_test_group_chat_with_invitation_and_membership};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_success() {
        // Arrange: Create real users and group in database
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create admin user and regular user
        let (admin_user, _) = create_test_user("invite_gc_create_admin").await;
        let target_user = admin_user.clone();
        
        // Create group chat with admin as creator (no membership created yet)
        let group_chat = crate::common::create_test_group_chat("invite_gc_create_group", admin_user.id).await;
        
        // Create invitation DTO
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat.id);

        // Act: Send the invitation
        let result = invitation_service.send_for_group_chat_create(invitation_dto, admin_user.id).await;

        // Assert: Verify invitation was sent successfully
        assert!(result.is_ok(), "Failed to send invitation: {:?}", result);
        let invitation_read_dto = result.unwrap();

        assert!(invitation_read_dto.id > 0, "Invitation ID should be positive");
        assert_eq!(invitation_read_dto.from_user_id, admin_user.id);
        assert_eq!(invitation_read_dto.to_user_id, target_user.id);
        assert_eq!(invitation_read_dto.group_chat_id, group_chat.id);
        assert_eq!(invitation_read_dto.status, InvitationStatus::Pending);
        assert!(invitation_read_dto.responded_at.is_none());
        assert!(invitation_read_dto.sent_at <= chrono::Utc::now());

        // Cleanup: Delete the test data
        cleanup_invitation(invitation_read_dto.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_user_not_found() {
        // Arrange: Create admin user and group, but no target user
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_gc_create_admin_no_target").await;
        let group_chat = crate::common::create_test_group_chat("send_gc_create_invitation_user_not_found", admin_user.id).await;

        let nonexistent_user_id = 99999;
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(nonexistent_user_id, group_chat.id);

        // Act: Try to send invitation to nonexistent user
        let result = invitation_service.send_for_group_chat_create(invitation_dto, admin_user.id).await;

        // Assert: Should fail with InvitedUserNotFound
        assert!(result.is_err(), "Should fail when target user doesn't exist");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitedUserNotFound) => {
                // Expected error
            }
            e => panic!("Expected InvitedUserNotFound error, got: {:?}", e),
        }

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_from_user_not_equals_to_user() {
        // Arrange: Create admin user and group, but no target user
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);

        let (admin_user, _) = create_test_user("invite_gc_create_admin_no_to_user").await;
        let (other_user, _) = create_test_user("invite_gc_create_other_to_user").await;
        let group_chat = crate::common::create_test_group_chat("send_gc_create_invitation_other_to_user", admin_user.id).await;

        let other_user_id = other_user.id;
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(other_user_id, group_chat.id);

        // Act: Try to send invitation to nonexistent user
        let result = invitation_service.send_for_group_chat_create(invitation_dto, admin_user.id).await;

        // Assert: Should fail with UserNotAuthorized
        assert!(result.is_err(), "Should fail when target user doesn't exist");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized(_)) => {
                // Expected error
            }
            e => panic!("Expected UserNotAuthorized error, got: {:?}", e),
        }

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_group_not_found() {
        // Arrange: Create users but no group
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_gc_create_admin_no_group").await;
        let target_user = admin_user.clone();
        
        let nonexistent_group_id = 99999;
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, nonexistent_group_id);

        // Act: Try to send invitation for nonexistent group
        let result = invitation_service.send_for_group_chat_create(invitation_dto, admin_user.id).await;

        // Assert: Should fail with GroupChatNotFound
        assert!(result.is_err(), "Should fail when group doesn't exist");
        match result.unwrap_err() {
            ApiError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                // Expected error
            }
            e => panic!("Expected GroupChatNotFound error, got: {:?}", e),
        }

        // Cleanup
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_user_not_authorized() {
        // Arrange: Create admin, regular user, and group where regular user is not admin
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_gc_create_real_admin").await;
        let (regular_user, _) = create_test_user("invite_gc_create_regular").await;
        let target_user = admin_user.clone();
        
        // Create group with admin_user as creator
        let group_chat = crate::common::create_test_group_chat("invite_gc_create_group_auth", admin_user.id).await;

        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat.id);

        // Act: Try to send invitation as regular user (not admin/creator)
        let result = invitation_service.send_for_group_chat_create(invitation_dto, regular_user.id).await;

        // Assert: Should fail with UserNotAuthorized
        assert!(result.is_err(), "Should fail when user is not group creator");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized(_)) => {
                // Expected error
            }
            e => panic!("Expected UserNotAuthorized error, got: {:?}", e),
        }

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(regular_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_user_already_in_group() {
        // Arrange: Create users, group, and existing membership
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_gc_create_admin_existing").await;
        let target_user = admin_user.clone();
        
        // Create group with admin as creator (this will create membership for admin)
        let group_chat = create_test_group_chat_with_invitation_and_membership("invite_gc_create_group_existing", admin_user.id).await;

        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat.id);

        // Act: Try to send invitation to user already in group
        let result = invitation_service.send_for_group_chat_create(invitation_dto, admin_user.id).await;

        // Assert: Should fail with UserAlreadyInGroup
        assert!(result.is_err(), "Should fail when user is already in group");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserAlreadyInGroup) => {
                // Expected error
            }
            e => panic!("Expected UserAlreadyInGroup error, got: {:?}", e),
        }

        // Cleanup: Clean up membership and invitation first, then group and users
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_already_invitation_pending() {
        // Arrange: Create users, group, and existing pending invitation
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_gc_create_admin_pending").await;
        let target_user = admin_user.clone();
        
        // Create group with admin as creator
        let group_chat = crate::common::create_test_group_chat("invite_gc_create_group_pending", admin_user.id).await;

        // Create existing pending invitation (no membership)
        let _existing_invitation = crate::common::create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat.id);

        // Act: Try to send another invitation
        let result = invitation_service.send_for_group_chat_create(invitation_dto, admin_user.id).await;

        // Assert: Should fail with AlreadyInvitationPending
        assert!(result.is_err(), "Should fail when invitation is already pending");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::AlreadyInvitationPending) => {
                // Expected error
            }
            e => panic!("Expected AlreadyInvitationPending error, got: {:?}", e),
        }

        // Cleanup
        cleanup_invitation(_existing_invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_with_admin_role() {
        // Arrange: Create users and group
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_gc_create_admin_role_admin").await;
        let target_user = admin_user.clone();
        
        // Create group with admin as creator
        let group_chat = crate::common::create_test_group_chat("invite_gc_create_group_role_admin", admin_user.id).await;
        
        // Create invitation DTO with Admin role
        let mut invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat.id);
        invitation_dto.role_at_join = ruggine_server::entity::group_membership::MemberRole::Admin;

        // Act: Send the invitation
        let result = invitation_service.send_for_group_chat_create(invitation_dto, admin_user.id).await;

        // Assert: Verify invitation was sent successfully with admin role
        assert!(result.is_ok(), "Failed to send admin role invitation: {:?}", result);
        let invitation_read_dto = result.unwrap();

        assert!(invitation_read_dto.id > 0, "Invitation ID should be positive");
        assert_eq!(invitation_read_dto.from_user_id, admin_user.id);
        assert_eq!(invitation_read_dto.to_user_id, target_user.id);
        assert_eq!(invitation_read_dto.group_chat_id, group_chat.id);
        assert_eq!(invitation_read_dto.role_at_join, ruggine_server::entity::group_membership::MemberRole::Admin);
        assert_eq!(invitation_read_dto.status, InvitationStatus::Pending);
        assert!(invitation_read_dto.responded_at.is_none());

        // Cleanup
        cleanup_invitation(invitation_read_dto.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_for_group_chat_create_user_with_inactive_membership() {
        // Arrange: Create users and group, then add user with inactive membership
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let invitation_service = service_init.invitation_service();
        let group_membership_service = service_init.group_membership_service();
        
        let (admin_user, _) = create_test_user("invite_gc_create_admin_inactive").await;

        // Create group with admin as creator
        let group_chat = create_test_group_chat_with_invitation_and_membership("invite_send_group_left", admin_user.id).await;

        // Simulate admin leaving the group by updating membership status
        let admin_group_membership = group_membership_service.find_by_user_id_and_group_id(
            admin_user.id,
            group_chat.id
        ).await.unwrap();
        let leave_group_membership_dto = LeaveGroupMembershipDto{ id:admin_group_membership.id };
        let _ = group_membership_service.leave_group(leave_group_membership_dto, admin_user.id).await.unwrap();

        // Create new invitation DTO
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(admin_user.id, group_chat.id);

        // Act: Send the invitation to user who previously left
        let result = invitation_service.send_for_group_chat_create(invitation_dto, admin_user.id).await;

        // Assert: Should succeed since user is no longer actively in the group
        assert!(result.is_ok(), "Should succeed for user who left the group: {:?}", result);
        let invitation_read_dto = result.unwrap();

        assert!(invitation_read_dto.id > 0, "Invitation ID should be positive");
        assert_eq!(invitation_read_dto.from_user_id, admin_user.id);
        assert_eq!(invitation_read_dto.to_user_id, admin_user.id);
        assert_eq!(invitation_read_dto.group_chat_id, group_chat.id);
        assert_eq!(invitation_read_dto.status, InvitationStatus::Pending);
        assert!(invitation_read_dto.responded_at.is_none());

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_invitation(invitation_read_dto.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
    }
}
