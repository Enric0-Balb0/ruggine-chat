use ruggine_server::service::invitation_service::{InvitationService, InvitationServiceTrait};
use ruggine_server::factory::invitation_factory::InvitationFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::invitation_error::InvitationError;
use ruggine_server::error::group_chat_error::GroupChatError;
use ruggine_server::entity::invitation::InvitationStatus;
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, cleanup_invitation};

#[cfg(test)]
mod invitation_service_send_integration_tests {
    use crate::{clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id, create_test_group_chat_with_invitation_and_membership};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_success() {
        // Arrange: Create real users and group in database
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create admin user and regular user
        let (admin_user, _) = create_test_user("invite_send_admin").await;
        let (target_user, _) = create_test_user("invite_send_target").await;
        
        // Create group chat with admin as creator
        let group_chat = create_test_group_chat_with_invitation_and_membership("invite_send_group", admin_user.id).await;
        
        // Create invitation DTO
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat.id);

        // Act: Send the invitation
        let result = invitation_service.send(invitation_dto, admin_user.id).await;

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
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_invitation(invitation_read_dto.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_user_not_found() {
        // Arrange: Create admin user and group, but no target user
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_send_admin_no_target").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("send_invitation_user_not_found", admin_user.id).await;

        let nonexistent_user_id = 99999;
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(nonexistent_user_id, group_chat.id);

        // Act: Try to send invitation to nonexistent user
        let result = invitation_service.send(invitation_dto, admin_user.id).await;

        // Assert: Should fail with InvitedUserNotFound
        assert!(result.is_err(), "Should fail when target user doesn't exist");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitedUserNotFound) => {
                // Expected error
            }
            e => panic!("Expected InvitedUserNotFound error, got: {:?}", e),
        }

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_group_not_found() {
        // Arrange: Create users but no group
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_send_admin_no_group").await;
        let (target_user, _) = create_test_user("invite_send_target_no_group").await;
        
        let nonexistent_group_id = 99999;
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, nonexistent_group_id);

        // Act: Try to send invitation for nonexistent group
        let result = invitation_service.send(invitation_dto, admin_user.id).await;

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
    async fn test_send_invitation_user_not_authorized() {
        // Arrange: Create admin, regular user, and group where regular user is not admin
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_send_real_admin").await;
        let (regular_user, _) = create_test_user("invite_send_regular").await;
        let (target_user, _) = create_test_user("invite_send_target_auth").await;
        
        // Create group with admin_user as creator
        let group_chat = create_test_group_chat_with_invitation_and_membership("invite_send_group_auth", admin_user.id).await;

        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat.id);

        // Act: Try to send invitation as regular user (not admin)
        let result = invitation_service.send(invitation_dto, regular_user.id).await;

        // Assert: Should fail with UserNotAuthorized
        assert!(result.is_err(), "Should fail when user is not group admin");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized(_)) => {
                // Expected error
            }
            e => panic!("Expected UserNotAuthorized error, got: {:?}", e),
        }

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(regular_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_already_pending() {
        // Arrange: Create users, group, and existing invitation
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_send_admin_pending").await;
        let (target_user, _) = create_test_user("invite_send_target_pending").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("invite_send_group_pending", admin_user.id).await;

        // Create first invitation
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat.id);
        let first_result = invitation_service.send(invitation_dto.clone(), admin_user.id).await;
        assert!(first_result.is_ok(), "First invitation should succeed");
        let first_invitation = first_result.unwrap();

        // Act: Try to send another invitation to same user for same group
        let result = invitation_service.send(invitation_dto, admin_user.id).await;

        // Assert: Should fail with AlreadyInvitationPending
        assert!(result.is_err(), "Should fail when invitation already exists");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::AlreadyInvitationPending) => {
                // Expected error
            }
            e => panic!("Expected AlreadyInvitationPending error, got: {:?}", e),
        }

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_invitation(first_invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_multiple_groups_same_users() {
        // Arrange: Create users and multiple groups
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_send_admin_multi").await;
        let (target_user, _) = create_test_user("invite_send_target_multi").await;
        let group_chat1 = create_test_group_chat_with_invitation_and_membership("invite_send_group_multi1", admin_user.id).await;
        let group_chat2 = create_test_group_chat_with_invitation_and_membership("invite_send_group_multi2", admin_user.id).await;
        
        // Create invitation DTOs for different groups
        let invitation_dto1 = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat1.id);
        let invitation_dto2 = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat2.id);

        // Act: Send invitations to same user for different groups
        let result1 = invitation_service.send(invitation_dto1, admin_user.id).await;
        let result2 = invitation_service.send(invitation_dto2, admin_user.id).await;

        // Assert: Both should succeed
        assert!(result1.is_ok(), "First invitation should succeed");
        assert!(result2.is_ok(), "Second invitation should succeed");
        
        let invitation1 = result1.unwrap();
        let invitation2 = result2.unwrap();
        
        assert_eq!(invitation1.group_chat_id, group_chat1.id);
        assert_eq!(invitation2.group_chat_id, group_chat2.id);
        assert_eq!(invitation1.to_user_id, target_user.id);
        assert_eq!(invitation2.to_user_id, target_user.id);

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat1.id).await;
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat2.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_different_users_same_group() {
        // Arrange: Create multiple users and one group
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (admin_user, _) = create_test_user("invite_send_admin_diff").await;
        let (target_user1, _) = create_test_user("invite_send_target_diff1").await;
        let (target_user2, _) = create_test_user("invite_send_target_diff2").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("invite_send_group_diff", admin_user.id).await;
        
        // Create invitation DTOs for different users
        let invitation_dto1 = InvitationFactory::fake_invitation_create_dto_with_ids(target_user1.id, group_chat.id);
        let invitation_dto2 = InvitationFactory::fake_invitation_create_dto_with_ids(target_user2.id, group_chat.id);

        // Act: Send invitations to different users for same group
        let result1 = invitation_service.send(invitation_dto1, admin_user.id).await;
        let result2 = invitation_service.send(invitation_dto2, admin_user.id).await;

        // Assert: Both should succeed
        assert!(result1.is_ok(), "First invitation should succeed");
        assert!(result2.is_ok(), "Second invitation should succeed");
        
        let invitation1 = result1.unwrap();
        let invitation2 = result2.unwrap();
        
        assert_eq!(invitation1.to_user_id, target_user1.id);
        assert_eq!(invitation2.to_user_id, target_user2.id);
        assert_eq!(invitation1.group_chat_id, group_chat.id);
        assert_eq!(invitation2.group_chat_id, group_chat.id);

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user1.email).await;
        cleanup_user(target_user2.email).await;
    }
}
