use ruggine_server::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use ruggine_server::entity::invitation::InvitationStatus;
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, create_test_group_chat, create_test_invitation, cleanup_invitation};

#[cfg(test)]
mod invitation_repository_find_pending_between_users_integration_tests {
    use ruggine_server::config::database::DatabaseTrait;
    use ruggine_server::entity::group_membership::MemberRole;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_found() {
        // Arrange
        let (from_user, _) = create_test_user("find_between_from").await;
        let (to_user, _) = create_test_user("find_between_to").await;
        let group_chat = create_test_group_chat("find_between_group", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create a pending invitation from user1 to user2
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        // Act
        let result = repository.find_pending_invitation_between_users(from_user.id, to_user.id, group_chat.id).await;

        // Assert
        assert!(result.is_ok(), "Should find the pending invitation successfully");
        let found_invitation = result.unwrap();
        assert!(found_invitation.is_some(), "Should find exactly one invitation");
        
        let invitation_data = found_invitation.unwrap();
        assert_eq!(invitation_data.id, invitation.id);
        assert_eq!(invitation_data.from_user_id, from_user.id);
        assert_eq!(invitation_data.to_user_id, to_user.id);
        assert_eq!(invitation_data.group_chat_id, group_chat.id);
        assert_eq!(invitation_data.status, InvitationStatus::Pending);
        assert!(invitation_data.responded_at.is_none());

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_not_found() {
        // Arrange
        let (user1, _) = create_test_user("find_between_user1").await;
        let (user2, _) = create_test_user("find_between_user2").await;
        let group_chat = create_test_group_chat("find_between_group", user1.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);

        // Act - search for invitation between users with no pending invitations
        let result = repository.find_pending_invitation_between_users(user1.id, user2.id, group_chat.id).await;

        // Assert
        assert!(result.is_ok(), "Should return None successfully");
        let found_invitation = result.unwrap();
        assert!(found_invitation.is_none(), "Should find no invitation between users");

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_ignores_accepted_rejected() {
        // Arrange
        let (from_user, _) = create_test_user("find_between_mixed_from").await;
        let (to_user, _) = create_test_user("find_between_mixed_to").await;
        let group_chat1 = create_test_group_chat("find_between_mixed_group1", from_user.id).await;

        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create an accepted invitation
        use ruggine_server::entity::invitation::NewInvitation;
        let accepted_id = repository.insert(NewInvitation {
            from_user_id: from_user.id,
            to_user_id: to_user.id,
            group_chat_id: group_chat1.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Update status to accepted
        sqlx::query!(
            "UPDATE \"invitation\" SET status = 'accepted', responded_at = NOW() WHERE id = $1",
            accepted_id
        )
        .execute(db.get_pool())
        .await
        .unwrap();

        // Create a rejected invitation
        let rejected_id = repository.insert(NewInvitation {
            from_user_id: from_user.id,
            to_user_id: to_user.id,
            group_chat_id: group_chat1.id,
            role_at_join: MemberRole::Member
        }).await.unwrap();

        // Update status to rejected
        sqlx::query!(
            "UPDATE \"invitation\" SET status = 'rejected', responded_at = NOW() WHERE id = $1",
            rejected_id
        )
        .execute(db.get_pool())
        .await
        .unwrap();

        // Act
        let result = repository.find_pending_invitation_between_users(from_user.id, to_user.id, group_chat1.id).await;

        // Assert
        assert!(result.is_ok(), "Should return None for non-pending invitations");
        let found_invitation = result.unwrap();
        assert!(found_invitation.is_none(), "Should not find accepted/rejected invitations");

        // Cleanup
        cleanup_invitation(accepted_id).await;
        cleanup_invitation(rejected_id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_specific_direction() {
        // Arrange
        let (user1, _) = create_test_user("find_between_dir_user1").await;
        let (user2, _) = create_test_user("find_between_dir_user2").await;
        let group_chat = create_test_group_chat("find_between_dir_group", user1.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create invitation from user1 to user2
        let invitation = create_test_invitation(user1.id, user2.id, group_chat.id).await;

        // Act - search in the same direction (should find)
        let result1 = repository.find_pending_invitation_between_users(user1.id, user2.id, group_chat.id).await;
        
        // Act - search in reverse direction (should not find with current implementation)
        let result2 = repository.find_pending_invitation_between_users(user2.id, user1.id, group_chat.id).await;

        // Assert
        assert!(result1.is_ok(), "Should find invitation in correct direction");
        let found_invitation1 = result1.unwrap();
        assert!(found_invitation1.is_some(), "Should find invitation from user1 to user2");
        
        assert!(result2.is_ok(), "Should not error in reverse direction");
        let found_invitation2 = result2.unwrap();
        assert!(found_invitation2.is_none(), "Should not find invitation from user2 to user1");

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_multiple_group_chats() {
        // Arrange
        let (from_user, _) = create_test_user("find_between_multi_from").await;
        let (to_user, _) = create_test_user("find_between_multi_to").await;
        let group_chat1 = create_test_group_chat("find_between_multi_group1", from_user.id).await;
        let group_chat2 = create_test_group_chat("find_between_multi_group2", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Create multiple pending invitations between same users for different groups
        let invitation1 = create_test_invitation(from_user.id, to_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user.id, group_chat2.id).await;

        // Act
        let result1 = repository.find_pending_invitation_between_users(from_user.id, to_user.id, group_chat1.id).await;
        let result2 = repository.find_pending_invitation_between_users(from_user.id, to_user.id, group_chat2.id).await;

        // Assert
        match result1 {
            Ok(Some(found1)) => {
                assert_eq!(found1.id, invitation1.id, "Expected invitation1 but got a different one");
            }
            Ok(None) => panic!("Expected an invitation for group_chat1, but got None"),
            Err(e) => panic!("Error retrieving invitation for group_chat1: {:?}", e),
        }

        match result2 {
            Ok(Some(found2)) => {
                assert_eq!(found2.id, invitation2.id, "Expected invitation2 but got a different one");
            }
            Ok(None) => panic!("Expected an invitation for group_chat2, but got None"),
            Err(e) => panic!("Error retrieving invitation for group_chat2: {:?}", e),
        }




        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_nonexistent_users() {
        // Arrange
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let nonexistent_user1_id = -1;
        let nonexistent_user2_id = -2;
        let nonexistent_group_id = -3;

        // Act
        let result = repository.find_pending_invitation_between_users(nonexistent_user1_id, nonexistent_user2_id, nonexistent_group_id).await;

        // Assert
        assert!(result.is_ok(), "Should return None for nonexistent users");
        let found_invitation = result.unwrap();
        assert!(found_invitation.is_none(), "Should find no invitation for nonexistent users");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_same_user() {
        // Arrange
        let (user, _) = create_test_user("find_between_same_user").await;
        let group_chat = create_test_group_chat("find_between_same_user", user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);

        // Act - search for invitation from user to themselves
        let result = repository.find_pending_invitation_between_users(user.id, user.id, group_chat.id).await;

        // Assert
        assert!(result.is_ok(), "Should return None for same user");
        let found_invitation = result.unwrap();
        assert!(found_invitation.is_none(), "Should find no invitation from user to themselves");

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.email).await;
    }
}
