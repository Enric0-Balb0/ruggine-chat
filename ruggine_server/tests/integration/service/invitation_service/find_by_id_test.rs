use ruggine_server::service::invitation_service::{InvitationService, InvitationServiceTrait};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::invitation_error::InvitationError;
use ruggine_server::entity::invitation::InvitationStatus;
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, create_test_group_chat, create_test_invitation, cleanup_invitation};

#[cfg(test)]
mod invitation_service_find_by_id_integration_tests {
    use ruggine_server::config::database::DatabaseTrait;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange: Create real invitation in database
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create users and group
        let (from_user, _) = create_test_user("find_by_id_from").await;
        let (to_user, _) = create_test_user("find_by_id_to").await;
        let group_chat = create_test_group_chat("find_by_id_group", from_user.id).await;
        
        // Create invitation using common helper
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        // Act: Find the invitation by ID
        let result1 = invitation_service.find_by_id(invitation.id, to_user.id).await;
        let result2 = invitation_service.find_by_id(invitation.id, from_user.id).await;

        // Assert: Verify invitation was found successfully
        assert!(result1.is_ok(), "Failed to find invitation: {:?}", result1);
        let mut invitation_read_dto = result1.unwrap();
        
        assert_eq!(invitation_read_dto.id, invitation.id);
        assert_eq!(invitation_read_dto.from_user_id, from_user.id);
        assert_eq!(invitation_read_dto.to_user_id, to_user.id);
        assert_eq!(invitation_read_dto.group_chat_id, group_chat.id);
        assert_eq!(invitation_read_dto.status, InvitationStatus::Pending);
        assert!(invitation_read_dto.responded_at.is_none());
        assert_eq!(invitation_read_dto.sent_at, invitation.sent_at);

        assert!(result2.is_ok(), "Failed to find invitation: {:?}", result2);
        invitation_read_dto = result2.unwrap();

        assert_eq!(invitation_read_dto.id, invitation.id);
        assert_eq!(invitation_read_dto.from_user_id, from_user.id);
        assert_eq!(invitation_read_dto.to_user_id, to_user.id);
        assert_eq!(invitation_read_dto.group_chat_id, group_chat.id);
        assert_eq!(invitation_read_dto.status, InvitationStatus::Pending);
        assert!(invitation_read_dto.responded_at.is_none());
        assert_eq!(invitation_read_dto.sent_at, invitation.sent_at);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange: Use nonexistent invitation ID
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let nonexistent_id = -1;

        // Act: Try to find nonexistent invitation
        let result = invitation_service.find_by_id(nonexistent_id, nonexistent_id).await;

        // Assert: Should fail with InvitationNotFound
        assert!(result.is_err(), "Should fail when invitation doesn't exist");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationNotFound) => {
                // Expected error
            }
            e => panic!("Expected InvitationNotFound error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_different_invitations() {
        // Arrange: Create multiple invitations
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create users and groups
        let (from_user1, _) = create_test_user("find_by_id_from1").await;
        let (from_user2, _) = create_test_user("find_by_id_from2").await;
        let (to_user1, _) = create_test_user("find_by_id_to1").await;
        let (to_user2, _) = create_test_user("find_by_id_to2").await;
        let group_chat1 = create_test_group_chat("find_by_id_group1", from_user1.id).await;
        let group_chat2 = create_test_group_chat("find_by_id_group2", from_user2.id).await;
        
        // Create different invitations
        let invitation1 = create_test_invitation(from_user1.id, to_user1.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(from_user2.id, to_user2.id, group_chat2.id).await;

        // Act: Find both invitations
        let result1 = invitation_service.find_by_id(invitation1.id, from_user1.id).await;
        let result2 = invitation_service.find_by_id(invitation2.id, from_user2.id).await;
        let result3 = invitation_service.find_by_id(invitation1.id, to_user1.id).await;
        let result4 = invitation_service.find_by_id(invitation2.id, to_user2.id).await;

        // Assert: Both should be found with correct data
        assert!(result1.is_ok(), "Failed to find first invitation");
        assert!(result2.is_ok(), "Failed to find second invitation");
        assert!(result3.is_ok(), "Failed to find third invitation");
        assert!(result4.is_ok(), "Failed to find fourth invitation");
        
        let invitation_dto1 = result1.unwrap();
        let invitation_dto2 = result2.unwrap();
        let invitation_dto3 = result3.unwrap();
        let invitation_dto4 = result4.unwrap();
        
        // Verify first invitation
        assert_eq!(invitation_dto1.id, invitation1.id);
        assert_eq!(invitation_dto1.from_user_id, from_user1.id);
        assert_eq!(invitation_dto1.to_user_id, to_user1.id);
        assert_eq!(invitation_dto1.group_chat_id, group_chat1.id);
        
        // Verify second invitation
        assert_eq!(invitation_dto2.id, invitation2.id);
        assert_eq!(invitation_dto2.from_user_id, from_user2.id);
        assert_eq!(invitation_dto2.to_user_id, to_user2.id);
        assert_eq!(invitation_dto2.group_chat_id, group_chat2.id);
        
        // Verify they are different
        assert_ne!(invitation_dto1.id, invitation_dto2.id);

        // Verify result1 is equals to result3 and result2 is equals result4
        assert_eq!(invitation_dto1, invitation_dto3);
        assert_eq!(invitation_dto2, invitation_dto4);

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(from_user1.email).await;
        cleanup_user(from_user2.email).await;
        cleanup_user(to_user1.email).await;
        cleanup_user(to_user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_with_responded_invitation() {
        // Arrange: Create invitation and manually update its status
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (from_user, _) = create_test_user("find_by_id_responded_from").await;
        let (to_user, _) = create_test_user("find_by_id_responded_to").await;
        let group_chat = create_test_group_chat("find_by_id_responded_group", from_user.id).await;
        
        // Create invitation
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        
        // Manually update invitation status to accepted
        sqlx::query!(
            "UPDATE \"invitation\" SET status = 'accepted', responded_at = NOW() WHERE id = $1",
            invitation.id
        )
        .execute(db.get_pool())
        .await
        .expect("Failed to update invitation status");

        // Act: Find the responded invitation
        let result = invitation_service.find_by_id(invitation.id, to_user.id).await;

        // Assert: Should find invitation with updated status
        assert!(result.is_ok(), "Failed to find responded invitation");
        let invitation_read_dto = result.unwrap();
        
        assert_eq!(invitation_read_dto.id, invitation.id);
        assert_eq!(invitation_read_dto.status, InvitationStatus::Accepted);
        assert!(invitation_read_dto.responded_at.is_some(), "Responded_at should be set");

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_after_deletion() {
        // Arrange: Create invitation then delete it
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (from_user, _) = create_test_user("find_by_id_deleted_from").await;
        let (to_user, _) = create_test_user("find_by_id_deleted_to").await;
        let group_chat = create_test_group_chat("find_by_id_deleted_group", from_user.id).await;
        
        // Create invitation
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        let invitation_id = invitation.id;
        
        // Delete the invitation
        cleanup_invitation(invitation_id).await;

        // Act: Try to find deleted invitation
        let result = invitation_service.find_by_id(invitation_id, to_user.id).await;

        // Assert: Should fail with InvitationNotFound
        assert!(result.is_err(), "Should fail when invitation is deleted");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationNotFound) => {
                // Expected error
            }
            e => panic!("Expected InvitationNotFound error, got: {:?}", e),
        }

        // Cleanup (invitation already deleted)
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_multiple_sequential_calls() {
        // Arrange: Create invitation
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (from_user, _) = create_test_user("find_by_id_sequential_from").await;
        let (to_user, _) = create_test_user("find_by_id_sequential_to").await;
        let group_chat = create_test_group_chat("find_by_id_sequential_group", from_user.id).await;
        
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        // Act: Call find_by_id multiple times
        let result1 = invitation_service.find_by_id(invitation.id, from_user.id).await;
        let result2 = invitation_service.find_by_id(invitation.id, to_user.id).await;
        let result3 = invitation_service.find_by_id(invitation.id, from_user.id).await;

        // Assert: All calls should succeed and return same data
        assert!(result1.is_ok(), "First call should succeed");
        assert!(result2.is_ok(), "Second call should succeed");
        assert!(result3.is_ok(), "Third call should succeed");
        
        let invitation_dto1 = result1.unwrap();
        let invitation_dto2 = result2.unwrap();
        let invitation_dto3 = result3.unwrap();
        
        // All should return identical data
        assert_eq!(invitation_dto1.id, invitation_dto2.id);
        assert_eq!(invitation_dto2.id, invitation_dto3.id);
        assert_eq!(invitation_dto1.from_user_id, invitation_dto2.from_user_id);
        assert_eq!(invitation_dto1.to_user_id, invitation_dto3.to_user_id);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }
}
