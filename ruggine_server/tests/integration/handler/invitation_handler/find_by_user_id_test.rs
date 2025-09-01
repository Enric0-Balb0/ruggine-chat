use ruggine_server::handler::invitation_handler::find_by_user_id::find_by_user_id;
use axum::{extract::State, Extension};
use crate::common::{cleanup_user_by_email, cleanup_group_chat, cleanup_invitation, create_test_user, create_test_group_chat, create_test_invitation};

#[cfg(test)]
mod find_by_user_id_handler_integration_tests {
    use ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto;
    use ruggine_server::entity::invitation::{InvitationStatus};
    use ruggine_server::error::request_error::ValidatedRequest;
    use ruggine_server::handler::invitation_handler::update_status::update_status;
    use crate::{cleanup_test_user_from_a_group_chat, create_invitation_state};
    use super::*;


    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_returns_received_invitations_successfully() {
        // Arrange: Create users, group chat and invitations
        let (sender1, _password1) = create_test_user("find_by_user_id_handler_sender1").await;
        let (sender2, _password2) = create_test_user("find_by_user_id_handler_sender2").await;
        let (recipient_user, _password3) = create_test_user("find_by_user_id_handler_recipient").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_handler_group1", sender1.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_handler_group2", sender2.id).await;
        
        // Create invitations where recipient_user is the recipient
        let invitation1 = create_test_invitation(sender1.id, recipient_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(sender2.id, recipient_user.id, group_chat2.id).await;
        
        let invitation_state = create_invitation_state().await;
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(recipient_user.clone()),
            State(invitation_state),
        ).await;

        // Assert: Should succeed and return both received invitations
        assert!(result.is_ok(), "Find by user ID should succeed");
        let invitation_response = result.unwrap().0;
        let data = invitation_response.data();

        assert_eq!(data.len(), 2, "Should return 2 received invitations");
        
        // Verify both invitations are for the recipient user
        assert!(data.iter().all(|inv| inv.to_user_id == recipient_user.id));
        
        // Verify invitation IDs match
        let returned_ids: Vec<i32> = data.iter().map(|inv| inv.id).collect();
        assert!(returned_ids.contains(&invitation1.id));
        assert!(returned_ids.contains(&invitation2.id));

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(sender1.email).await;
        cleanup_user_by_email(sender2.email).await;
        cleanup_user_by_email(recipient_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_no_invitations() {
        // Arrange: Create a user with no invitations
        let (user_with_no_invitations, _password) = create_test_user("find_by_user_id_handler_empty").await;
        
        let invitation_state = create_invitation_state().await;
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(user_with_no_invitations.clone()),
            State(invitation_state),
        ).await;

        // Assert: Should succeed and return empty list
        assert!(result.is_ok(), "Find by user ID should succeed even with no invitations");
        let invitation_response = result.unwrap().0;
        let data = invitation_response.data();

        assert_eq!(data.len(), 0, "Should return empty list for user with no invitations");

        // Cleanup
        cleanup_user_by_email(user_with_no_invitations.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_only_received_invitations() {
        // Arrange: Create users where target user sends invitations (not receives)
        let (sender_user, _password1) = create_test_user("find_by_user_id_handler_sender_only").await;
        let (recipient1, _password2) = create_test_user("find_by_user_id_handler_recipient1").await;
        let (recipient2, _password3) = create_test_user("find_by_user_id_handler_recipient2").await;
        
        let group_chat = create_test_group_chat("find_by_user_id_handler_sender_group", sender_user.id).await;
        
        // Create invitations where sender_user is the sender (not recipient)
        let invitation1 = create_test_invitation(sender_user.id, recipient1.id, group_chat.id).await;
        let invitation2 = create_test_invitation(sender_user.id, recipient2.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        
        // Act: Call find_by_user_id handler (should only return received invitations by default)
        let result = find_by_user_id(
            Extension(sender_user.clone()),
            State(invitation_state),
        ).await;

        // Assert: Should succeed but return empty list (since sender_user has no received invitations)
        assert!(result.is_ok(), "Find by user ID should succeed");
        let invitation_response = result.unwrap().0;
        let data = invitation_response.data();

        assert_eq!(data.len(), 0, "Should return empty list since user only sent invitations, didn't receive any");

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender_user.email).await;
        cleanup_user_by_email(recipient1.email).await;
        cleanup_user_by_email(recipient2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_only_received_invitations_mixture() {
        // Arrange: Create users where target user sends invitations (not receives)
        let (sender_user, _password1) = create_test_user("find_by_user_id_handler_sender_only").await;
        let (recipient1, _password2) = create_test_user("find_by_user_id_handler_recipient1").await;
        let (recipient2, _password3) = create_test_user("find_by_user_id_handler_recipient2").await;

        let group_chat = create_test_group_chat("find_by_user_id_handler_sender_group", sender_user.id).await;

        // Create invitations where sender_user is the sender (not recipient)
        let invitation1 = create_test_invitation(sender_user.id, recipient1.id, group_chat.id).await;
        let invitation2 = create_test_invitation(sender_user.id, recipient2.id, group_chat.id).await;

        let invitation_state = create_invitation_state().await;

        // Act: Call find_by_user_id handler (should return only one invitation for recipient1)
        let result = find_by_user_id(
            Extension(recipient1.clone()),
            State(invitation_state),
        ).await;

        // Assert: Should succeed but return empty list (since sender_user has no received invitations)
        assert!(result.is_ok(), "Find by user ID should succeed");
        let invitation_response = result.unwrap().0;
        let data = invitation_response.data();

        assert_eq!(data.len(), 1, "Should return only on invitation.");

        // Verify both invitations are for the recipient user
        assert!(data.iter().all(|inv| inv.to_user_id == recipient1.id));

        // Verify invitation IDs match
        let returned_ids: Vec<i32> = data.iter().map(|inv| inv.id).collect();
        assert!(returned_ids.contains(&invitation1.id));
        assert!(!returned_ids.contains(&invitation2.id));

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender_user.email).await;
        cleanup_user_by_email(recipient1.email).await;
        cleanup_user_by_email(recipient2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_different_statuses() {
        // Arrange: Create invitations with different statuses
        let (sender, _password1) = create_test_user("find_by_user_id_handler_status_sender").await;
        let (recipient_user, _password2) = create_test_user("find_by_user_id_handler_status_recipient").await;
        
        let group_chat = create_test_group_chat("find_by_user_id_handler_status_group", sender.id).await;
        
        let invitation_state = create_invitation_state().await;
        
        // Create invitation and update its status
        let invitation_id = create_test_invitation(sender.id, recipient_user.id, group_chat.id).await.id;

        // Update status to accepted
        let update_status_response  = update_status(
            Extension(recipient_user.clone()),
            State(invitation_state.clone()),
            ValidatedRequest(InvitationUpdateStatusDto {
                status: InvitationStatus::Accepted,
                invitation_id
            })
        ).await;

        let _ = update_status_response.unwrap();
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(recipient_user.clone()),
            State(invitation_state),
        ).await;

        // Assert: Should return the accepted invitation
        assert!(result.is_ok(), "Find by user ID should succeed");
        let invitation_response = result.unwrap().0;
        let data = invitation_response.data();

        assert_eq!(data.len(), 1, "Should return 1 invitation");
        assert_eq!(data[0].status, InvitationStatus::Accepted, "Should return accepted invitation");
        assert_eq!(data[0].to_user_id, recipient_user.id);

        // Cleanup
        cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
        cleanup_invitation(invitation_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(recipient_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_ordering() {
        // Arrange: Create multiple invitations to test ordering
        let (sender1, _password1) = create_test_user("find_by_user_id_handler_order_sender1").await;
        let (sender2, _password2) = create_test_user("find_by_user_id_handler_order_sender2").await;
        let (recipient_user, _password3) = create_test_user("find_by_user_id_handler_order_recipient").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_handler_order_group", sender1.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_handler_order_group", sender2.id).await;
        
        // Create invitations with some delay to ensure different sent_at times
        let invitation_id_1 = create_test_invitation(sender1.id, recipient_user.id, group_chat1.id).await.id;
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let invitation_id_2 = create_test_invitation(sender2.id, recipient_user.id, group_chat2.id).await.id;

        let invitation_state = create_invitation_state().await;
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(recipient_user.clone()),
            State(invitation_state),
        ).await;

        // Assert: Should return invitations ordered by sent_at DESC
        assert!(result.is_ok(), "Find by user ID should succeed");
        let invitation_response = result.unwrap().0;
        let data = invitation_response.data();

        assert_eq!(data.len(), 2, "Should return 2 invitations");
        
        // Should be ordered by sent_at DESC (most recent first)
        assert!(data[0].sent_at >= data[1].sent_at, "Invitations should be ordered by sent_at DESC");

        // Cleanup
        cleanup_invitation(invitation_id_1).await;
        cleanup_invitation(invitation_id_2).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(sender1.email).await;
        cleanup_user_by_email(sender2.email).await;
        cleanup_user_by_email(recipient_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_response_structure() {
        // Arrange: Create a simple invitation to test response structure
        let (sender, _password1) = create_test_user("find_by_user_id_handler_struct_sender").await;
        let (recipient_user, _password2) = create_test_user("find_by_user_id_handler_struct_recipient").await;
        
        let group_chat = create_test_group_chat("find_by_user_id_handler_struct_group", sender.id).await;
        let invitation = create_test_invitation(sender.id, recipient_user.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(recipient_user.clone()),
            State(invitation_state),
        ).await;

        // Assert: Verify response structure and DTO mapping
        assert!(result.is_ok(), "Find by user ID should succeed");
        let invitation_response = result.unwrap().0;
        let data = invitation_response.data();

        assert_eq!(data.len(), 1, "Should return 1 invitation");
        
        let returned_invitation = &data[0];
        assert_eq!(returned_invitation.id, invitation.id);
        assert_eq!(returned_invitation.from_user_id, invitation.from_user_id);
        assert_eq!(returned_invitation.to_user_id, invitation.to_user_id);
        assert_eq!(returned_invitation.group_chat_id, invitation.group_chat_id);
        assert_eq!(returned_invitation.status, invitation.status);
        assert!(returned_invitation.responded_at.is_none());

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(recipient_user.email).await;
    }
}
