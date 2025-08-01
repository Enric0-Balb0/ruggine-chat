use std::sync::Arc;
use ruggine_server::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use ruggine_server::factory::invitation_factory::InvitationFactory;
use ruggine_server::entity::invitation::{InvitationStatus, NewInvitation};
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, create_test_group_chat, create_test_invitation, cleanup_invitation};

#[cfg(test)]
mod invitation_repository_find_by_id_integration_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange
        let (from_user, _) = create_test_user("find_invitation_from").await;
        let (to_user, _) = create_test_user("find_invitation_to").await;
        let group_chat = create_test_group_chat("find_invitation_group", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // First create an invitation to find
        let invitation_id = repository.insert(NewInvitation {
            from_user_id: from_user.id,
            to_user_id: to_user.id,
            group_chat_id: group_chat.id,
        }).await.unwrap();

        // Act
        let mut result = repository.find_by_id(invitation_id, to_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find the invitation successfully");
        let mut found_invitation = result.unwrap();
        assert_eq!(found_invitation.id, invitation_id);
        assert_eq!(found_invitation.from_user_id, from_user.id);
        assert_eq!(found_invitation.to_user_id, to_user.id);
        assert_eq!(found_invitation.group_chat_id, group_chat.id);
        assert_eq!(found_invitation.status, InvitationStatus::Pending);
        assert!(found_invitation.responded_at.is_none());
        assert!(found_invitation.sent_at <= chrono::Utc::now());

        // Now try with to_user_id
        // Act
        result = repository.find_by_id(invitation_id, from_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find the invitation successfully");
        found_invitation = result.unwrap();
        assert_eq!(found_invitation.id, invitation_id);
        assert_eq!(found_invitation.from_user_id, from_user.id);
        assert_eq!(found_invitation.to_user_id, to_user.id);
        assert_eq!(found_invitation.group_chat_id, group_chat.id);
        assert_eq!(found_invitation.status, InvitationStatus::Pending);
        assert!(found_invitation.responded_at.is_none());
        assert!(found_invitation.sent_at <= chrono::Utc::now());

        // Cleanup
        cleanup_invitation(invitation_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let non_existent_id = -1;
        let user_id = -1;

        // Act
        let result = repository.find_by_id(non_existent_id, user_id).await;

        // Assert
        assert!(result.is_err(), "Should not find non-existent invitation");
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {}, // Expected
            other => panic!("Expected RowNotFound error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_zero_id() {
        // Arrange
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let zero_id = 0;

        // Act
        let result = repository.find_by_id(zero_id, zero_id).await;

        // Assert
        assert!(result.is_err(), "Should not find invitation with zero ID");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_multiple_invitations() {
        // Arrange
        let (from_user, _) = create_test_user("multi_find_from").await;
        let (to_user1, _) = create_test_user("multi_find_to1").await;
        let (to_user2, _) = create_test_user("multi_find_to2").await;
        let group_chat = create_test_group_chat("multi_find_group", from_user.id).await;
        
        let invitation1 = create_test_invitation(from_user.id, to_user1.id, group_chat.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user2.id, group_chat.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);

        // Act
        let result1 = repository.find_by_id(invitation1.id, from_user.id).await;
        let result2 = repository.find_by_id(invitation2.id, from_user.id).await;
        let result3 = repository.find_by_id(invitation1.id, to_user1.id).await;
        let result4 = repository.find_by_id(invitation2.id, to_user2.id).await;

        // Assert
        assert!(result1.is_ok(), "Should find first invitation");
        assert!(result2.is_ok(), "Should find second invitation");
        assert!(result3.is_ok(), "Should find first invitation for to_user1");
        assert!(result4.is_ok(), "Should find second invitation for to_user2");

        let found1 = result1.unwrap();
        let found2 = result2.unwrap();
        let found3 = result3.unwrap();
        let found4 = result4.unwrap();

        assert_eq!(found1.id, invitation1.id);
        assert_eq!(found1.to_user_id, to_user1.id);
        assert_eq!(found2.id, invitation2.id);
        assert_eq!(found2.to_user_id, to_user2.id);

        // Both should have same from_user and group_chat
        assert_eq!(found1.from_user_id, from_user.id);
        assert_eq!(found2.from_user_id, from_user.id);
        assert_eq!(found1.group_chat_id, group_chat.id);
        assert_eq!(found2.group_chat_id, group_chat.id);

        // result3 and result4 should match found1 and found2 respectively
        assert_eq!(found3.id, invitation1.id);
        assert_eq!(found3.from_user_id, from_user.id);
        assert_eq!(found4.id, invitation2.id);
        assert_eq!(found4.from_user_id, from_user.id);

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user1.email).await;
        cleanup_user(to_user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_concurrent_access() {
        // Arrange
        let (from_user, _) = create_test_user("concurrent_find_from").await;
        let (to_user1, _) = create_test_user("concurrent_find_to1").await;
        let (to_user2, _) = create_test_user("concurrent_find_to2").await;
        let group_chat = create_test_group_chat("concurrent_find_group", from_user.id).await;
        
        let invitation1 = create_test_invitation(from_user.id, to_user1.id, group_chat.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user2.id, group_chat.id).await;
        
        let db = get_database().await;
        let repository = Arc::new(InvitationRepository::new(&db));
        let repo1 = repository.clone();
        let repo2 = repository.clone();

        // Act - Concurrent find operations
        let (result1, result2) = tokio::join!(
            repo1.find_by_id(invitation1.id, to_user1.id),
            repo2.find_by_id(invitation2.id, to_user2.id)
        );

        // Assert
        assert!(result1.is_ok(), "First concurrent find should succeed");
        assert!(result2.is_ok(), "Second concurrent find should succeed");

        let found1 = result1.unwrap();
        let found2 = result2.unwrap();

        assert_eq!(found1.id, invitation1.id);
        assert_eq!(found2.id, invitation2.id);
        assert_ne!(found1.id, found2.id);

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user1.email).await;
        cleanup_user(to_user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_timestamp_validation() {
        // Arrange
        let (from_user, _) = create_test_user("timestamp_find_from").await;
        let (to_user, _) = create_test_user("timestamp_find_to").await;
        let group_chat = create_test_group_chat("timestamp_find_group", from_user.id).await;
        
        let before_creation = chrono::Utc::now();
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        let after_creation = chrono::Utc::now();
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);

        // Act
        let result = repository.find_by_id(invitation.id, from_user.id).await;

        // Assert
        assert!(result.is_ok(), "Should find invitation");
        let found_invitation = result.unwrap();
        
        // Validate timestamps
        assert!(found_invitation.sent_at >= before_creation, "sent_at should be after test start");
        assert!(found_invitation.sent_at <= after_creation, "sent_at should be before test end");
        assert!(found_invitation.responded_at.is_none(), "responded_at should be None for pending invitation");

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_database_consistency() {
        // Arrange
        let (from_user, _) = create_test_user("consistency_find_from").await;
        let (to_user, _) = create_test_user("consistency_find_to").await;
        let group_chat = create_test_group_chat("consistency_find_group", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create invitation directly using repository
        let new_invitation = ruggine_server::entity::invitation::NewInvitation {
            from_user_id: from_user.id,
            to_user_id: to_user.id,
            group_chat_id: group_chat.id,
        };
        let invitation_id = repository.insert(new_invitation.clone()).await.unwrap();

        // Act - Find the invitation multiple times
        let result1 = repository.find_by_id(invitation_id, from_user.id).await;
        let result2 = repository.find_by_id(invitation_id, to_user.id).await;
        let result3 = repository.find_by_id(invitation_id, from_user.id).await;

        // Assert
        assert!(result1.is_ok(), "First find should succeed");
        assert!(result2.is_ok(), "Second find should succeed");
        assert!(result3.is_ok(), "Third find should succeed");

        let found1 = result1.unwrap();
        let found2 = result2.unwrap();
        let found3 = result3.unwrap();

        // All results should be identical
        assert_eq!(found1.id, found2.id);
        assert_eq!(found2.id, found3.id);
        assert_eq!(found1.from_user_id, found2.from_user_id);
        assert_eq!(found2.from_user_id, found3.from_user_id);
        assert_eq!(found1.to_user_id, found2.to_user_id);
        assert_eq!(found2.to_user_id, found3.to_user_id);
        assert_eq!(found1.group_chat_id, found2.group_chat_id);
        assert_eq!(found2.group_chat_id, found3.group_chat_id);
        assert_eq!(found1.status, found2.status);
        assert_eq!(found2.status, found3.status);
        assert_eq!(found1.sent_at, found2.sent_at);
        assert_eq!(found2.sent_at, found3.sent_at);
        assert_eq!(found1.responded_at, found2.responded_at);
        assert_eq!(found2.responded_at, found3.responded_at);

        // Cleanup
        cleanup_invitation(invitation_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }
}
