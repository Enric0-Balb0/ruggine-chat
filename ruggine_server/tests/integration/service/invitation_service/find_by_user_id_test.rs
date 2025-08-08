use ruggine_server::service::invitation_service::{InvitationService, InvitationServiceTrait};
use ruggine_server::entity::invitation::{InvitationStatus, NewInvitation};
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, create_test_group_chat, cleanup_invitation};

#[cfg(test)]
mod invitation_service_find_by_user_id_integration_tests {
    use ruggine_server::entity::group_membership::MemberRole;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_success_only_received_invitations() {
        // Arrange
        let (from_user, _) = create_test_user("svc_find_by_user_id_from").await;
        let (to_user, _) = create_test_user("svc_find_by_user_id_to").await;
        let (other_user, _) = create_test_user("svc_find_by_user_id_other").await;
        
        let group_chat1 = create_test_group_chat("svc_find_by_user_id_group1", from_user.id).await;
        let group_chat2 = create_test_group_chat("svc_find_by_user_id_group2", other_user.id).await;
        
        let db = get_database().await;
        let service = InvitationService::new(&db);
        
        // Create invitations: some sent by to_user, some received by to_user
        let invitation_id_1 = service.invitation_repo().insert(NewInvitation {
            from_user_id: to_user.id, // sent by to_user (should NOT be returned by service)
            to_user_id: other_user.id,
            group_chat_id: group_chat2.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();
        
        let invitation_id_2 = service.invitation_repo().insert(NewInvitation {
            from_user_id: from_user.id,
            to_user_id: to_user.id, // received by to_user (SHOULD be returned by service)
            group_chat_id: group_chat1.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Act - Service should return only received invitations by default
        let result = service.find_by_user_id(to_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find invitations for the user successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 1, "Should find only 1 received invitation");
        
        // Verify only the received invitation is returned
        let invitation_ids: Vec<i32> = found_invitations.iter().map(|inv| inv.id).collect();
        assert!(!invitation_ids.contains(&invitation_id_1)); // Should NOT include sent invitation
        assert!(invitation_ids.contains(&invitation_id_2)); // SHOULD include received invitation
        
        // All returned should be received by user (to_user_id == user_id)
        assert!(found_invitations.iter().all(|inv| inv.to_user_id == to_user.id));
        
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
        let (user_with_no_invitations, _) = create_test_user("svc_find_by_user_id_empty").await;
        
        let db = get_database().await;
        let service = InvitationService::new(&db);

        // Act
        let result = service.find_by_user_id(user_with_no_invitations.id).await;

        // Assert
        assert!(result.is_ok(), "Should return empty result successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0, "Should find no invitations");

        // Cleanup
        cleanup_user(user_with_no_invitations.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_different_statuses() {
        // Arrange
        let (user, _) = create_test_user("svc_find_by_user_id_statuses").await;
        let (other_user, _) = create_test_user("svc_find_by_user_id_other_statuses").await;
        
        let group_chat = create_test_group_chat("svc_find_by_user_id_status_group", other_user.id).await;
        
        let db = get_database().await;
        let service = InvitationService::new(&db);
        
        // Create invitation and update its status
        let invitation_id = service.invitation_repo().insert(NewInvitation {
            from_user_id: other_user.id,
            to_user_id: user.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Act - Find invitations before status update
        let result_before = service.find_by_user_id(user.id).await;

        // Assert - Should find the pending invitation
        assert!(result_before.is_ok());
        let invitations_before = result_before.unwrap();
        assert_eq!(invitations_before.len(), 1);
        assert_eq!(invitations_before[0].status, InvitationStatus::Pending);

        // Update status to accepted
        use ruggine_server::entity::invitation::UpdateInvitationStatus;
        service.invitation_repo().update_status(invitation_id, UpdateInvitationStatus {
            invitation_id: invitations_before[0].id,
            status: InvitationStatus::Accepted,
        }).await.unwrap();

        // Act - Find invitations after status update
        let result_after = service.find_by_user_id(user.id).await;

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
        let service = InvitationService::new(&db);
        let nonexistent_user_id = -999;

        // Act
        let result = service.find_by_user_id(nonexistent_user_id).await;

        // Assert
        assert!(result.is_ok(), "Should return empty result for nonexistent user");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0, "Should find no invitations for nonexistent user");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_ordering() {
        // Arrange
        let (user1, _) = create_test_user("svc_find_by_user_id_order_user1").await;
        let (user2, _) = create_test_user("svc_find_by_user_id_order_user2").await;
        let (user3, _) = create_test_user("svc_find_by_user_id_order_user3").await;
        
        let group_chat1 = create_test_group_chat("svc_find_by_user_id_order_group", user2.id).await;
        let group_chat2 = create_test_group_chat("svc_find_by_user_id_order_group", user3.id).await;
        
        let db = get_database().await;
        let service = InvitationService::new(&db);
        
        // Create invitations with some delay to ensure different sent_at times
        let invitation_id_1 = service.invitation_repo().insert(NewInvitation {
            from_user_id: user2.id,
            to_user_id: user1.id,
            group_chat_id: group_chat1.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let invitation_id_2 = service.invitation_repo().insert(NewInvitation {
            from_user_id: user3.id,
            to_user_id: user1.id,
            group_chat_id: group_chat2.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Act
        let result = service.find_by_user_id(user1.id).await;

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
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
        cleanup_user(user3.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_service_layer_error_handling() {
        // Arrange
        let (user, _) = create_test_user("svc_find_by_user_id_error").await;
        let (other_user, _) = create_test_user("svc_find_by_user_id_error_other").await;
        let group_chat = create_test_group_chat("svc_find_by_user_id_error_group", user.id).await;
        
        let db = get_database().await;
        let service = InvitationService::new(&db);
        
        // Create a valid invitation first
        let invitation_id = service.invitation_repo().insert(NewInvitation {
            from_user_id: user.id,
            to_user_id: other_user.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Act - Test with valid user (should work)
        let result = service.find_by_user_id(user.id).await;

        // Assert - Should work properly through the service layer
        assert!(result.is_ok(), "Service should handle valid requests successfully");
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0, "Should not find the invitation");

        // Cleanup
        cleanup_invitation(invitation_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.email).await;
        cleanup_user(other_user.email).await;
    }
}
