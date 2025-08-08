use ruggine_server::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use ruggine_server::entity::invitation::{InvitationStatus, NewInvitation, UserInvitationFilter};
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, create_test_group_chat, cleanup_invitation};

#[cfg(test)]
mod invitation_repository_find_by_user_id_integration_tests {
    use ruggine_server::entity::group_membership::MemberRole;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_success_mixed_invitations() {
        // Arrange
        let (from_user, _) = create_test_user("find_by_user_id_from").await;
        let (to_user, _) = create_test_user("find_by_user_id_to").await;
        let (other_user, _) = create_test_user("find_by_user_id_other").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_group1", from_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_group2", to_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create invitations: some sent by to_user, some received by to_user
        let invitation_id_1 = repository.insert(NewInvitation {
            from_user_id: to_user.id, // sent by to_user
            to_user_id: other_user.id,
            group_chat_id: group_chat2.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();
        
        let invitation_id_2 = repository.insert(NewInvitation {
            from_user_id: from_user.id,
            to_user_id: to_user.id, // received by to_user
            group_chat_id: group_chat1.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Act
        let result = repository.find_by_user_id(to_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find invitations for the user successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2, "Should find 2 invitations");
        
        // Verify both invitations involve the target user
        let invitation_ids: Vec<i32> = found_invitations.iter().map(|inv| inv.id).collect();
        assert!(invitation_ids.contains(&invitation_id_1));
        assert!(invitation_ids.contains(&invitation_id_2));
        
        // Verify one is sent by user and one is received by user
        let sent_count = found_invitations.iter().filter(|inv| inv.from_user_id == to_user.id).count();
        let received_count = found_invitations.iter().filter(|inv| inv.to_user_id == to_user.id).count();
        assert_eq!(sent_count, 1, "Should have 1 sent invitation");
        assert_eq!(received_count, 1, "Should have 1 received invitation");
        
        // All should be pending
        assert!(found_invitations.iter().all(|inv| inv.status == InvitationStatus::Pending));

        // Cleanup
        cleanup_invitation(invitation_id_1).await;
        cleanup_invitation(invitation_id_2).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
        cleanup_user(other_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_no_invitations() {
        // Arrange
        let (user_with_no_invitations, _) = create_test_user("find_by_user_id_empty").await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);

        // Act
        let result = repository.find_by_user_id(user_with_no_invitations.id).await;

        // Assert
        assert!(result.is_ok(), "Should return empty result successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0, "Should find no invitations");

        // Cleanup
        cleanup_user(user_with_no_invitations.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_only_sent_invitations() {
        // Arrange
        let (sender_user, _) = create_test_user("find_by_user_id_sender").await;
        let (recipient1, _) = create_test_user("find_by_user_id_recipient1").await;
        let (recipient2, _) = create_test_user("find_by_user_id_recipient2").await;
        
        let group_chat = create_test_group_chat("find_by_user_id_sender_group", sender_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create multiple invitations sent by sender_user
        let invitation_id_1 = repository.insert(NewInvitation {
            from_user_id: sender_user.id,
            to_user_id: recipient1.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();
        
        let invitation_id_2 = repository.insert(NewInvitation {
            from_user_id: sender_user.id,
            to_user_id: recipient2.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Admin
        }).await.unwrap();

        // Act
        let result = repository.find_by_user_id(sender_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find sent invitations successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2, "Should find 2 sent invitations");
        
        // All should be sent by the user
        assert!(found_invitations.iter().all(|inv| inv.from_user_id == sender_user.id));
        
        // Verify recipients
        let recipient_ids: Vec<i32> = found_invitations.iter().map(|inv| inv.to_user_id).collect();
        assert!(recipient_ids.contains(&recipient1.id));
        assert!(recipient_ids.contains(&recipient2.id));

        // Cleanup
        cleanup_invitation(invitation_id_1).await;
        cleanup_invitation(invitation_id_2).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(sender_user.email).await;
        cleanup_user(recipient1.email).await;
        cleanup_user(recipient2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_only_received_invitations() {
        // Arrange
        let (recipient_user, _) = create_test_user("find_by_user_id_recipient").await;
        let (sender1, _) = create_test_user("find_by_user_id_sender1").await;
        let (sender2, _) = create_test_user("find_by_user_id_sender2").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_recv_group1", sender1.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_recv_group2", sender2.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create multiple invitations received by recipient_user
        let invitation_id_1 = repository.insert(NewInvitation {
            from_user_id: sender1.id,
            to_user_id: recipient_user.id,
            group_chat_id: group_chat1.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();
        
        let invitation_id_2 = repository.insert(NewInvitation {
            from_user_id: sender2.id,
            to_user_id: recipient_user.id,
            group_chat_id: group_chat2.id,
            role_at_join: MemberRole::Admin
        }).await.unwrap();

        // Act
        let result = repository.find_by_user_id(recipient_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find received invitations successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2, "Should find 2 received invitations");
        
        // All should be received by the user
        assert!(found_invitations.iter().all(|inv| inv.to_user_id == recipient_user.id));
        
        // Verify senders
        let sender_ids: Vec<i32> = found_invitations.iter().map(|inv| inv.from_user_id).collect();
        assert!(sender_ids.contains(&sender1.id));
        assert!(sender_ids.contains(&sender2.id));

        // Cleanup
        cleanup_invitation(invitation_id_1).await;
        cleanup_invitation(invitation_id_2).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender1.email).await;
        cleanup_user(sender2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_different_statuses() {
        // Arrange
        let (user, _) = create_test_user("find_by_user_id_statuses").await;
        let (other_user, _) = create_test_user("find_by_user_id_other_statuses").await;
        
        let group_chat = create_test_group_chat("find_by_user_id_status_group", user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create invitation and update its status
        let invitation_id = repository.insert(NewInvitation {
            from_user_id: user.id,
            to_user_id: other_user.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Act - Find invitations before status update
        let result_before = repository.find_by_user_id(user.id).await;

        // Assert - Should find the pending invitation
        assert!(result_before.is_ok());
        let invitations_before = result_before.unwrap();
        assert_eq!(invitations_before.len(), 1);
        assert_eq!(invitations_before[0].status, InvitationStatus::Pending);

        // Update status to accepted
        use ruggine_server::entity::invitation::UpdateInvitationStatus;
        repository.update_status(invitation_id, UpdateInvitationStatus {
            invitation_id: invitations_before[0].id,
            status: InvitationStatus::Accepted,
        }).await.unwrap();

        // Act - Find invitations after status update
        let result_after = repository.find_by_user_id(user.id).await;

        // Assert - Should still find the invitation (now accepted)
        assert!(result_after.is_ok());
        let invitations_after = result_after.unwrap();
        assert_eq!(invitations_after.len(), 1);
        assert_eq!(invitations_after[0].status, InvitationStatus::Accepted);

        // Cleanup
        cleanup_invitation(invitation_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.email).await;
        cleanup_user(other_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_nonexistent_user() {
        // Arrange
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let nonexistent_user_id = -999;

        // Act
        let result = repository.find_by_user_id(nonexistent_user_id).await;

        // Assert
        assert!(result.is_ok(), "Should return empty result for nonexistent user");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0, "Should find no invitations for nonexistent user");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_ordering() {
        // Arrange
        let (user1, _) = create_test_user("find_by_user_id_order_user1").await;
        let (user2, _) = create_test_user("find_by_user_id_order_user2").await;
        let (user3, _) = create_test_user("find_by_user_id_order_user3").await;
        
        let group_chat = create_test_group_chat("find_by_user_id_order_group", user1.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create invitations with some delay to ensure different sent_at times
        let invitation_id_1 = repository.insert(NewInvitation {
            from_user_id: user1.id,
            to_user_id: user2.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let invitation_id_2 = repository.insert(NewInvitation {
            from_user_id: user1.id,
            to_user_id: user3.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Act
        let result = repository.find_by_user_id(user1.id).await;

        // Assert
        assert!(result.is_ok(), "Should find invitations successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2, "Should find 2 invitations");
        
        // Should be ordered by sent_at DESC (most recent first)
        assert!(found_invitations[0].sent_at >= found_invitations[1].sent_at, 
                "Invitations should be ordered by sent_at DESC");

        // Cleanup
        cleanup_invitation(invitation_id_1).await;
        cleanup_invitation(invitation_id_2).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
        cleanup_user(user3.email).await;
    }
}
