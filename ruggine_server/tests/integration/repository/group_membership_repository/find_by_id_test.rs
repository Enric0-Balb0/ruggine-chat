use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::MemberRole;

#[cfg(test)]
mod group_membership_repository_integration_tests {
    use ruggine_server::entity::group_membership::MembershipStatus;
    use crate::{get_database, create_test_user, create_test_group_chat, cleanup_group_chat, cleanup_user, cleanup_group_membership};

    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Create test user and group chat for the membership
        let (user, _password) = create_test_user("find_by_id").await;
        let group_chat = create_test_group_chat("find_by_id", user.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_ids(user.id, group_chat.id);

        // Insert membership first
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        // Act
        let result = repository.find_by_id(membership_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to find membership by id");
        let found_membership = result.unwrap();
        
        assert_eq!(found_membership.id, membership_id);
        assert_eq!(found_membership.user_id, new_membership.user_id);
        assert_eq!(found_membership.group_chat_id, new_membership.group_chat_id);
        assert_eq!(found_membership.role, new_membership.role);
        assert!(found_membership.joined_at.timestamp() > 0, "joined_at should be set");
        assert!(found_membership.left_at.is_none(), "left_at should be None for active membership");
        assert_eq!(found_membership.membership_status, MembershipStatus::Active, "New membership should have active status");

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_admin_membership() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Create test user and group chat for the membership
        let (user, _password) = create_test_user("find_admin_by_id").await;
        let group_chat = create_test_group_chat("find_admin_by_id", user.id).await;

        let new_membership = GroupMembershipFactory::fake_new_admin_group_membership_with_ids(user.id, group_chat.id);

        // Insert admin membership first
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        // Act
        let result = repository.find_by_id(membership_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to find admin membership by id");
        let found_membership = result.unwrap();
        
        assert_eq!(found_membership.id, membership_id);
        assert_eq!(found_membership.user_id, new_membership.user_id);
        assert_eq!(found_membership.group_chat_id, new_membership.group_chat_id);
        assert_eq!(found_membership.role, MemberRole::Admin);
        assert!(found_membership.joined_at.timestamp() > 0, "joined_at should be set");
        assert!(found_membership.left_at.is_none(), "left_at should be None for active membership");
        assert_eq!(found_membership.membership_status, MembershipStatus::Active, "New membership should have active status");

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Act - Try to find a membership with an ID that doesn't exist
        let result = repository.find_by_id(99999).await;

        // Assert
        assert!(result.is_err(), "Should not find membership with non-existent ID");
        
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {
                // This is expected
            }
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_multiple_memberships() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Create test users and group chats
        let (user1, _password1) = create_test_user("multi_find1").await;
        let (user2, _password2) = create_test_user("multi_find2").await;
        let group_chat1 = create_test_group_chat("multi_find_group1", user1.id).await;
        let group_chat2 = create_test_group_chat("multi_find_group2", user1.id).await;

        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_ids(user1.id, group_chat1.id);
        let membership2 = GroupMembershipFactory::fake_new_admin_group_membership_with_ids(user2.id, group_chat2.id);

        // Insert memberships
        let membership_id1 = repository.insert(membership1.clone()).await.unwrap();
        let membership_id2 = repository.insert(membership2.clone()).await.unwrap();

        // Act - Find each membership by its ID
        let result1 = repository.find_by_id(membership_id1).await;
        let result2 = repository.find_by_id(membership_id2).await;

        // Assert
        assert!(result1.is_ok(), "Failed to find first membership");
        assert!(result2.is_ok(), "Failed to find second membership");

        let found_membership1 = result1.unwrap();
        let found_membership2 = result2.unwrap();

        // Verify first membership
        assert_eq!(found_membership1.id, membership_id1);
        assert_eq!(found_membership1.user_id, user1.id);
        assert_eq!(found_membership1.group_chat_id, group_chat1.id);
        assert_eq!(found_membership1.role, MemberRole::Member);

        // Verify second membership
        assert_eq!(found_membership2.id, membership_id2);
        assert_eq!(found_membership2.user_id, user2.id);
        assert_eq!(found_membership2.group_chat_id, group_chat2.id);
        assert_eq!(found_membership2.role, MemberRole::Admin);

        // Cleanup
        cleanup_group_membership(membership_id1).await;
        cleanup_group_membership(membership_id2).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(user1.email.clone()).await;
        cleanup_user(user2.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_with_negative_id() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Act - Try to find a membership with a negative ID
        let result = repository.find_by_id(-1).await;

        // Assert
        assert!(result.is_err(), "Should not find membership with negative ID");
        
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {
                // This is expected
            }
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_with_zero_id() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Act - Try to find a membership with ID zero
        let result = repository.find_by_id(0).await;

        // Assert
        assert!(result.is_err(), "Should not find membership with zero ID");
        
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {
                // This is expected
            }
            _ => panic!("Expected RowNotFound error"),
        }
    }
}
