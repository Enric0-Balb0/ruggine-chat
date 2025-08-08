use ruggine_server::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use ruggine_server::entity::invitation::{InvitationStatus, UpdateInvitationStatus};
use crate::common::{cleanup_user, create_test_user, cleanup_group_chat, create_test_group_chat, cleanup_invitation, create_test_invitation};
use chrono::Utc;

#[cfg(test)]
mod invitation_repository_update_integration_tests {
    use crate::get_database;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_accept_success() {
        // Arrange
        let (from_user, _) = create_test_user("update_from_user_accept").await;
        let (to_user, _) = create_test_user("update_to_user_accept").await;
        let group_chat = create_test_group_chat("update_group_accept", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let update_status = UpdateInvitationStatus {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };

        // Act
        let result = repository.update_status(invitation.id, update_status.clone()).await;

        // Assert
        assert!(result.is_ok(), "Update status should succeed");
        let updated_invitation = result.unwrap();
        assert_eq!(updated_invitation.id, invitation.id);
        assert_eq!(updated_invitation.status, InvitationStatus::Accepted);
        assert!(updated_invitation.responded_at.is_some());
        assert_eq!(updated_invitation.from_user_id, from_user.id);
        assert_eq!(updated_invitation.to_user_id, to_user.id);
        assert_eq!(updated_invitation.group_chat_id, group_chat.id);

        // Verify the update persists in database
        let retrieved_invitation = repository.find_by_id_and_user_id(invitation.id, to_user.id).await;
        assert!(retrieved_invitation.is_ok());
        let persisted_invitation = retrieved_invitation.unwrap();
        assert_eq!(persisted_invitation.status, InvitationStatus::Accepted);
        assert!(persisted_invitation.responded_at.is_some());

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_reject_success() {
        // Arrange
        let (from_user, _) = create_test_user("update_from_user_reject").await;
        let (to_user, _) = create_test_user("update_to_user_reject").await;
        let group_chat = create_test_group_chat("update_group_reject", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let update_status = UpdateInvitationStatus {
            invitation_id: invitation.id,
            status: InvitationStatus::Rejected,
        };

        // Act
        let result = repository.update_status(invitation.id, update_status.clone()).await;

        // Assert
        assert!(result.is_ok(), "Update status should succeed");
        let updated_invitation = result.unwrap();
        assert_eq!(updated_invitation.id, invitation.id);
        assert_eq!(updated_invitation.status, InvitationStatus::Rejected);
        assert!(updated_invitation.responded_at.is_some());
        assert_eq!(updated_invitation.from_user_id, from_user.id);
        assert_eq!(updated_invitation.to_user_id, to_user.id);
        assert_eq!(updated_invitation.group_chat_id, group_chat.id);

        // Verify the update persists in database
        let retrieved_invitation = repository.find_by_id_and_user_id(invitation.id, to_user.id).await;
        assert!(retrieved_invitation.is_ok());
        let persisted_invitation = retrieved_invitation.unwrap();
        assert_eq!(persisted_invitation.status, InvitationStatus::Rejected);
        assert!(persisted_invitation.responded_at.is_some());

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_nonexistent_invitation() {
        // Arrange
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let update_status = UpdateInvitationStatus {
            invitation_id: -1,
            status: InvitationStatus::Accepted,
        };

        // Act
        let result = repository.update_status(-1, update_status).await;

        // Assert
        assert!(result.is_err(), "Should fail for nonexistent invitation");
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {
                // Success - expected error
            },
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_already_responded_invitation() {
        // Arrange
        let (from_user, _) = create_test_user("update_from_user_already").await;
        let (to_user, _) = create_test_user("update_to_user_already").await;
        let group_chat = create_test_group_chat("update_group_already", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // First update - accept the invitation
        let first_update = UpdateInvitationStatus {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };
        let first_result = repository.update_status(invitation.id, first_update).await;
        assert!(first_result.is_ok(), "First update should succeed");

        // Second update - try to reject the already accepted invitation
        let second_update = UpdateInvitationStatus {
            invitation_id: invitation.id,
            status: InvitationStatus::Rejected,
        };

        // Act
        let result = repository.update_status(invitation.id, second_update).await;

        // Assert - This should still succeed as we're just updating the database record
        assert!(result.is_ok(), "Second update should also succeed");
        let updated_invitation = result.unwrap();
        assert_eq!(updated_invitation.status, InvitationStatus::Rejected);
        assert!(updated_invitation.responded_at.is_some());

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_preserves_other_fields() {
        // Arrange
        let (from_user, _) = create_test_user("update_from_user_preserve").await;
        let (to_user, _) = create_test_user("update_to_user_preserve").await;
        let group_chat = create_test_group_chat("update_group_preserve", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        
        // Store original values for comparison
        let original_sent_at = invitation.sent_at;
        let original_id = invitation.id;
        let original_from_user_id = invitation.from_user_id;
        let original_to_user_id = invitation.to_user_id;
        let original_group_chat_id = invitation.group_chat_id;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let update_status = UpdateInvitationStatus {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };

        // Act
        let result = repository.update_status(invitation.id, update_status).await;

        // Assert
        assert!(result.is_ok(), "Update should succeed");
        let updated_invitation = result.unwrap();
        
        // Verify only status and responded_at changed
        assert_eq!(updated_invitation.id, original_id);
        assert_eq!(updated_invitation.from_user_id, original_from_user_id);
        assert_eq!(updated_invitation.to_user_id, original_to_user_id);
        assert_eq!(updated_invitation.group_chat_id, original_group_chat_id);
        assert_eq!(updated_invitation.sent_at, original_sent_at);
        
        // Verify the fields that should have changed
        assert_eq!(updated_invitation.status, InvitationStatus::Accepted);
        assert!(updated_invitation.responded_at.is_some());

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }
}
