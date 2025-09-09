use ruggine_server::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use ruggine_server::entity::invitation::InvitationStatus;
use crate::common::{get_database, create_test_user, cleanup_user_by_email, cleanup_group_chat, create_test_group_chat, create_test_invitation, cleanup_invitation};

#[cfg(test)]
mod invitation_repository_find_pending_for_user_integration_tests {
    use ruggine_server::config::database::DatabaseTrait;
    use ruggine_server::entity::group_membership::MemberRole;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitations_for_user_success() {
        // Arrange
        let (from_user, _) = create_test_user("find_pending_from").await;
        let (to_user, _) = create_test_user("find_pending_to").await;
        let group_chat1 = create_test_group_chat("find_pending_group1", from_user.id).await;
        let group_chat2 = create_test_group_chat("find_pending_group2", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create multiple pending invitations for the same user
        let invitation1 = create_test_invitation(from_user.id, to_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user.id, group_chat2.id).await;

        // Act
        let result = repository.find_pending_invitations_for_user(to_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find pending invitations successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2, "Should find exactly 2 pending invitations");
        
        for invitation in &found_invitations {
            assert_eq!(invitation.to_user_id, to_user.id);
            assert_eq!(invitation.from_user_id, from_user.id);
            assert_eq!(invitation.status, InvitationStatus::Pending);
            assert!(invitation.responded_at.is_none());
        }

        // Check that invitations are ordered by sent_at DESC (most recent first)
        if found_invitations.len() > 1 {
            assert!(found_invitations[0].sent_at >= found_invitations[1].sent_at);
        }

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(from_user.email).await;
        cleanup_user_by_email(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitations_for_user_empty_result() {
        // Arrange
        let (user, _) = create_test_user("find_pending_empty").await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);

        // Act - search for a user with no pending invitations
        let result = repository.find_pending_invitations_for_user(user.id).await;

        // Assert
        assert!(result.is_ok(), "Should return empty result successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0, "Should find no pending invitations");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitations_for_user_ignores_accepted_rejected() {
        // Arrange
        let (from_user, _) = create_test_user("find_pending_mixed_from").await;
        let (to_user, _) = create_test_user("find_pending_mixed_to").await;
        let group_chat1 = create_test_group_chat("find_pending_mixed_group1", from_user.id).await;
        let group_chat2 = create_test_group_chat("find_pending_mixed_group2", from_user.id).await;
        let group_chat3 = create_test_group_chat("find_pending_mixed_group3", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create one pending invitation
        let pending_invitation = create_test_invitation(from_user.id, to_user.id, group_chat1.id).await;
        
        // Create accepted and rejected invitations (we'll simulate this by inserting directly)
        use ruggine_server::entity::invitation::NewInvitation;
        let accepted_id = repository.insert(NewInvitation {
            from_user_id: from_user.id,
            to_user_id: to_user.id,
            group_chat_id: group_chat2.id,
            role_at_join: MemberRole::Member,
        }).await.unwrap();
        
        let rejected_id = repository.insert(NewInvitation {
            from_user_id: from_user.id,
            to_user_id: to_user.id,
            group_chat_id: group_chat3.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Manually update status to simulate accepted/rejected invitations
        // (In a real scenario, these would be updated through a different method)
        sqlx::query(
            "UPDATE invitation
            SET status = $1::invitation_status, responded_at = NOW()
            WHERE id = $2"
        )
        .bind("accepted")
        .bind(accepted_id)
        .execute(db.get_pool())
        .await
        .unwrap();

        sqlx::query(
            "UPDATE invitation
            SET status = $1::invitation_status, responded_at = NOW()
            WHERE id = $2"
        )
        .bind("rejected")
        .bind(rejected_id)
        .execute(db.get_pool())
        .await
        .unwrap();

        // Act
        let result = repository.find_pending_invitations_for_user(to_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find pending invitations successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 1, "Should find only the pending invitation");
        
        let invitation = &found_invitations[0];
        assert_eq!(invitation.id, pending_invitation.id);
        assert_eq!(invitation.status, InvitationStatus::Pending);

        // Cleanup
        cleanup_invitation(pending_invitation.id).await;
        cleanup_invitation(accepted_id).await;
        cleanup_invitation(rejected_id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_group_chat(group_chat3.id).await;
        cleanup_user_by_email(from_user.email).await;
        cleanup_user_by_email(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitations_for_user_multiple_senders() {
        // Arrange
        let (from_user1, _) = create_test_user("find_pending_sender1").await;
        let (from_user2, _) = create_test_user("find_pending_sender2").await;
        let (to_user, _) = create_test_user("find_pending_receiver").await;
        let group_chat1 = create_test_group_chat("find_pending_multi_group1", from_user1.id).await;
        let group_chat2 = create_test_group_chat("find_pending_multi_group2", from_user2.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create pending invitations from different users to the same recipient
        let invitation1 = create_test_invitation(from_user1.id, to_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(from_user2.id, to_user.id, group_chat2.id).await;

        // Act
        let result = repository.find_pending_invitations_for_user(to_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find pending invitations successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2, "Should find invitations from both senders");
        
        let invitation_ids: Vec<i32> = found_invitations.iter().map(|inv| inv.id).collect();
        assert!(invitation_ids.contains(&invitation1.id));
        assert!(invitation_ids.contains(&invitation2.id));
        
        for invitation in &found_invitations {
            assert_eq!(invitation.to_user_id, to_user.id);
            assert_eq!(invitation.status, InvitationStatus::Pending);
        }

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(from_user1.email).await;
        cleanup_user_by_email(from_user2.email).await;
        cleanup_user_by_email(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitations_for_user_nonexistent_user() {
        // Arrange
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let nonexistent_user_id = -1;

        // Act
        let result = repository.find_pending_invitations_for_user(nonexistent_user_id).await;

        // Assert
        assert!(result.is_ok(), "Should return empty result for nonexistent user");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0, "Should find no invitations for nonexistent user");
    }
}
