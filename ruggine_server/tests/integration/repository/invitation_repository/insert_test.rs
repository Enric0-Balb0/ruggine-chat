use std::sync::Arc;
use ruggine_server::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use ruggine_server::factory::invitation_factory::InvitationFactory;
use ruggine_server::repository::user_repository::{UserRepositoryTrait};
use ruggine_server::repository::group_chat_repository::{GroupChatRepositoryTrait};
use crate::common::{cleanup_user, create_test_user, cleanup_group, create_test_group_chat, cleanup_invitation};

#[cfg(test)]
mod invitation_repository_integration_tests {
    use ruggine_server::config::database::DatabaseTrait;
    use ruggine_server::entity::invitation::NewInvitation;
    use crate::get_database;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_invitation_success() {
        // Arrange
        let (from_user, _) = create_test_user("invitation_from_user").await;
        let (to_user, _) = create_test_user("invitation_to_user").await;
        let group_chat = create_test_group_chat("invitation_group", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let new_invitation = NewInvitation { from_user_id: from_user.id, to_user_id: to_user.id, group_chat_id: group_chat.id };

        // Act
        let result = repository.insert(new_invitation.clone()).await;

        // Assert
        assert!(result.is_ok(), "Invitation insert should succeed");
        let invitation_id = result.unwrap();
        assert!(invitation_id > 0, "Invitation ID should be positive");

        // Verify invitation was created correctly
        let retrieved_invitation = repository.find_by_id(invitation_id).await;
        assert!(retrieved_invitation.is_ok(), "Should be able to retrieve created invitation");
        let invitation = retrieved_invitation.unwrap();
        assert_eq!(invitation.from_user_id, from_user.id);
        assert_eq!(invitation.to_user_id, to_user.id);
        assert_eq!(invitation.group_chat_id, group_chat.id);
        assert_eq!(invitation.status, ruggine_server::entity::invitation::InvitationStatus::Pending);
        assert!(invitation.responded_at.is_none());

        // Cleanup
        
        cleanup_invitation(invitation_id).await;
        cleanup_group(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_invitation_nonexistent_from_user() {
        // Arrange
        let (to_user, _) = create_test_user("invitation_to_user_invalid").await;
        let group_chat = create_test_group_chat("invitation_group_invalid", to_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let new_invitation = NewInvitation {from_user_id: -1, to_user_id: to_user.id, group_chat_id: group_chat.id };

        // Act
        let result = repository.insert(new_invitation).await;

        // Assert
        assert!(result.is_err(), "Should fail with foreign key constraint error");

        // Cleanup
        
        cleanup_group(group_chat.id).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_invitation_nonexistent_to_user() {
        // Arrange
        let (from_user, _) = create_test_user("invitation_from_user_invalid").await;
        let group_chat = create_test_group_chat("invitation_group_invalid2", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let new_invitation = NewInvitation {from_user_id: from_user.id, to_user_id: -1, group_chat_id: group_chat.id };

        // Act
        let result = repository.insert(new_invitation).await;

        // Assert
        assert!(result.is_err(), "Should fail with foreign key constraint error");

        // Cleanup
        
        cleanup_group(group_chat.id).await;
        cleanup_user(from_user.email).await;

    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_invitation_nonexistent_group_chat() {
        // Arrange
        let (from_user, _) = create_test_user("invitation_from_user_invalid_group").await;
        let (to_user, _) = create_test_user("invitation_to_user_invalid_group").await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        let new_invitation = NewInvitation {from_user_id: from_user.id, to_user_id: to_user.id, group_chat_id: -1 };

        // Act
        let result = repository.insert(new_invitation).await;

        // Assert
        assert!(result.is_err(), "Should fail with foreign key constraint error");

        // Cleanup
        
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_multiple_invitations_same_group() {
        // Arrange
        let (from_user, _) = create_test_user("multi_invitation_from").await;
        let (to_user1, _) = create_test_user("multi_invitation_to1").await;
        let (to_user2, _) = create_test_user("multi_invitation_to2").await;
        let (to_user3, _) = create_test_user("multi_invitation_to3").await;
        let group_chat = create_test_group_chat("multi_invitation_group", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        let invitation1 = NewInvitation {from_user_id: from_user.id, to_user_id: to_user1.id, group_chat_id: group_chat.id };
        
        let invitation2 =  NewInvitation {from_user_id: from_user.id, to_user_id: to_user2.id, group_chat_id: group_chat.id };
        
        let invitation3 =  NewInvitation {from_user_id: from_user.id, to_user_id: to_user3.id, group_chat_id: group_chat.id };

        // Act
        let result1 = repository.insert(invitation1).await;
        let result2 = repository.insert(invitation2).await;
        let result3 = repository.insert(invitation3).await;

        // Assert
        assert!(result1.is_ok(), "First invitation should succeed");
        assert!(result2.is_ok(), "Second invitation should succeed");
        assert!(result3.is_ok(), "Third invitation should succeed");

        let invitation_id1 = result1.unwrap();
        let invitation_id2 = result2.unwrap();
        let invitation_id3 = result3.unwrap();

        // Verify all invitations are different
        assert_ne!(invitation_id1, invitation_id2);
        assert_ne!(invitation_id2, invitation_id3);
        assert_ne!(invitation_id1, invitation_id3);

        // Cleanup
        
        cleanup_invitation(invitation_id1).await;
        cleanup_invitation(invitation_id2).await;
        cleanup_invitation(invitation_id3).await;
        cleanup_group(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user1.email).await;
        cleanup_user(to_user2.email).await;
        cleanup_user(to_user3.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_invitation_with_factory_utilities() {
        // Arrange
        let (from_user, _) = create_test_user("factory_invitation_from").await;
        let (to_user, _) = create_test_user("factory_invitation_to").await;
        let group_chat = create_test_group_chat("factory_invitation_group", from_user.id).await;
        
        let db = get_database().await;
        let repository = InvitationRepository::new(&db);
        
        // Use factory utility methods to build invitation
        let base_invitation = InvitationFactory::fake_new_invitation();
        let custom_invitation = InvitationFactory::with_group_chat_id(
            InvitationFactory::with_to_user_id(
                InvitationFactory::with_from_user_id(base_invitation, from_user.id),
                to_user.id
            ),
            group_chat.id
        );

        // Act
        let result = repository.insert(custom_invitation).await;

        // Assert
        assert!(result.is_ok(), "Invitation with factory utilities should succeed");
        let invitation_id = result.unwrap();
        assert!(invitation_id > 0, "Invitation ID should be positive");

        // Verify the invitation
        let retrieved_invitation = repository.find_by_id(invitation_id).await;
        assert!(retrieved_invitation.is_ok());
        let invitation = retrieved_invitation.unwrap();
        assert_eq!(invitation.from_user_id, from_user.id);
        assert_eq!(invitation.to_user_id, to_user.id);
        assert_eq!(invitation.group_chat_id, group_chat.id);

        // Cleanup
        
        cleanup_invitation(invitation_id).await;
        cleanup_group(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_invitation_concurrent() {
        // Arrange
        let (from_user, _) = create_test_user("concurrent_invitation_from").await;
        let (to_user1, _) = create_test_user("concurrent_invitation_to1").await;
        let (to_user2, _) = create_test_user("concurrent_invitation_to2").await;
        let group_chat = create_test_group_chat("concurrent_invitation_group", from_user.id).await;
        
        let db = get_database().await;
        let repository = Arc::new(InvitationRepository::new(&db));
        
        let invitation1 =  NewInvitation {from_user_id: from_user.id, to_user_id: to_user1.id, group_chat_id: group_chat.id };
        
        let invitation2 =  NewInvitation {from_user_id: from_user.id, to_user_id: to_user2.id, group_chat_id: group_chat.id };

        let repo1 = repository.clone();
        let repo2 = repository.clone();

        // Act - Execute invitations concurrently
        let (result1, result2) = tokio::join!(
            repo1.insert(invitation1),
            repo2.insert(invitation2)
        );

        // Assert
        assert!(result1.is_ok(), "First concurrent invitation should succeed");
        assert!(result2.is_ok(), "Second concurrent invitation should succeed");

        let invitation_id1 = result1.unwrap();
        let invitation_id2 = result2.unwrap();
        assert_ne!(invitation_id1, invitation_id2, "Concurrent invitation IDs should be different");

        // Cleanup
        
        cleanup_invitation(invitation_id1).await;
        cleanup_invitation(invitation_id2).await;
        cleanup_group(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user1.email).await;
        cleanup_user(to_user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_invitation_duplicate_pending_fails() {
        // Arrange
        let (from_user, _) = create_test_user("dup_pending_from").await;
        let (to_user, _) = create_test_user("dup_pending_to").await;
        let group_chat = create_test_group_chat("dup_pending_group", from_user.id).await;

        let db = get_database().await;
        let repository = InvitationRepository::new(&db);

        let invitation1 = NewInvitation { from_user_id: from_user.id, to_user_id: to_user.id, group_chat_id: group_chat.id };
        let invitation2 = NewInvitation { from_user_id: from_user.id, to_user_id: to_user.id, group_chat_id: group_chat.id };

        // Act
        let result1 = repository.insert(invitation1).await;
        let result2 = repository.insert(invitation2).await;

        // Assert
        assert!(result1.is_ok(), "First invitation should succeed");

        // The second insert should fail for unique constraint violation
        assert!(result2.is_err(), "Second invitation should fail due to unique pending constraint");

        // Check if it is unique violation error
        if let Err(sqlx::Error::Database(db_err)) = &result2 {
            assert_eq!(db_err.code().as_deref(), Some("23505"), "Error code should be 23505 for unique violation");
        } else {
            panic!("Expected a database unique violation error");
        }

        // Cleanup
        if let Ok(id) = result1 {
            cleanup_invitation(id).await;
        }

        
        cleanup_group(group_chat.id).await;
        cleanup_user(from_user.email).await;
        cleanup_user(to_user.email).await;
    }


}
