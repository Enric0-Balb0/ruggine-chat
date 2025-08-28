use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;

#[cfg(test)]
mod group_membership_repository_find_connected_users_integration_tests {
    use ruggine_server::config::database::DatabaseTrait;
    use crate::common;
    use super::*;
    use crate::common::{
        get_database, create_test_user, create_test_group_chat, cleanup_group_chat,
        cleanup_user_by_email, cleanup_group_membership, create_test_invitation, cleanup_invitation
    };

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_success_single_shared_group() {
        // Arrange: Create users sharing one group
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_owner_single").await;
        let (target_user, _) = create_test_user("find_connected_target_single").await;
        let (connected_user, _) = create_test_user("find_connected_connected_single").await;
        
        let group_chat = create_test_group_chat("find_connected_single", owner_user.id).await;
        
        // Create invitations for both target_user and connected_user to the same group
        let invitation_target = create_test_invitation(owner_user.id, target_user.id, group_chat.id).await;
        let invitation_connected = create_test_invitation(owner_user.id, connected_user.id, group_chat.id).await;
        
        // Create active memberships for both users in the same group
        let new_membership_target = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target.id);
        let new_membership_connected = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected.id);
        
        let membership_id_target = repository.insert(new_membership_target).await.unwrap();
        let membership_id_connected = repository.insert(new_membership_connected).await.unwrap();

        // Act: Find connected users for target_user
        let result = repository.find_connected_users(target_user.id).await;

        // Assert: Should find connected_user as they share the same group
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 1);
        assert!(connected_user_ids.contains(&connected_user.id));
        assert!(!connected_user_ids.contains(&target_user.id)); // Should not include self

        // Cleanup
        cleanup_group_membership(membership_id_target).await;
        cleanup_group_membership(membership_id_connected).await;
        cleanup_invitation(invitation_target.id).await;
        cleanup_invitation(invitation_connected.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(target_user.email.clone()).await;
        cleanup_user_by_email(connected_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_multiple_shared_groups() {
        // Arrange: Create users sharing multiple groups
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_owner_multi").await;
        let (target_user, _) = create_test_user("find_connected_target_multi").await;
        let (connected_user1, _) = create_test_user("find_connected_connected1_multi").await;
        let (connected_user2, _) = create_test_user("find_connected_connected2_multi").await;
        
        // Create multiple group chats
        let group_chat1 = create_test_group_chat("find_connected_multi1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_connected_multi2", owner_user.id).await;
        
        // Create invitations - target_user shares group1 with connected_user1 and group2 with connected_user2
        let invitation_target_group1 = create_test_invitation(owner_user.id, target_user.id, group_chat1.id).await;
        let invitation_connected1_group1 = create_test_invitation(owner_user.id, connected_user1.id, group_chat1.id).await;
        let invitation_target_group2 = create_test_invitation(owner_user.id, target_user.id, group_chat2.id).await;
        let invitation_connected2_group2 = create_test_invitation(owner_user.id, connected_user2.id, group_chat2.id).await;
        
        // Create active memberships
        let membership_target_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target_group1.id);
        let membership_connected1_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected1_group1.id);
        let membership_target_2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target_group2.id);
        let membership_connected2_2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected2_group2.id);
        
        let membership_id_target_1 = repository.insert(membership_target_1).await.unwrap();
        let membership_id_connected1_1 = repository.insert(membership_connected1_1).await.unwrap();
        let membership_id_target_2 = repository.insert(membership_target_2).await.unwrap();
        let membership_id_connected2_2 = repository.insert(membership_connected2_2).await.unwrap();

        // Act: Find connected users for target_user
        let result = repository.find_connected_users(target_user.id).await;

        // Assert: Should find both connected users
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 2);
        assert!(connected_user_ids.contains(&connected_user1.id));
        assert!(connected_user_ids.contains(&connected_user2.id));
        assert!(!connected_user_ids.contains(&target_user.id)); // Should not include self

        // Cleanup
        cleanup_group_membership(membership_id_target_1).await;
        cleanup_group_membership(membership_id_connected1_1).await;
        cleanup_group_membership(membership_id_target_2).await;
        cleanup_group_membership(membership_id_connected2_2).await;
        cleanup_invitation(invitation_target_group1.id).await;
        cleanup_invitation(invitation_connected1_group1.id).await;
        cleanup_invitation(invitation_target_group2.id).await;
        cleanup_invitation(invitation_connected2_group2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(target_user.email.clone()).await;
        cleanup_user_by_email(connected_user1.email.clone()).await;
        cleanup_user_by_email(connected_user2.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_same_user_multiple_groups_with_same_connected_user() {
        // Arrange: Target user shares multiple groups with the same connected user
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_owner_same").await;
        let (target_user, _) = create_test_user("find_connected_target_same").await;
        let (connected_user, _) = create_test_user("find_connected_connected_same").await;
        
        // Create multiple group chats
        let group_chat1 = create_test_group_chat("find_connected_same1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_connected_same2", owner_user.id).await;
        
        // Create invitations - both target_user and connected_user are in both groups
        let invitation_target_group1 = create_test_invitation(owner_user.id, target_user.id, group_chat1.id).await;
        let invitation_connected_group1 = create_test_invitation(owner_user.id, connected_user.id, group_chat1.id).await;
        let invitation_target_group2 = create_test_invitation(owner_user.id, target_user.id, group_chat2.id).await;
        let invitation_connected_group2 = create_test_invitation(owner_user.id, connected_user.id, group_chat2.id).await;
        
        // Create active memberships
        let membership_target_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target_group1.id);
        let membership_connected_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected_group1.id);
        let membership_target_2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target_group2.id);
        let membership_connected_2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected_group2.id);
        
        let membership_id_target_1 = repository.insert(membership_target_1).await.unwrap();
        let membership_id_connected_1 = repository.insert(membership_connected_1).await.unwrap();
        let membership_id_target_2 = repository.insert(membership_target_2).await.unwrap();
        let membership_id_connected_2 = repository.insert(membership_connected_2).await.unwrap();

        // Act: Find connected users for target_user
        let result = repository.find_connected_users(target_user.id).await;

        // Assert: Should find connected_user only once (DISTINCT should prevent duplicates)
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 1);
        assert!(connected_user_ids.contains(&connected_user.id));

        // Cleanup
        cleanup_group_membership(membership_id_target_1).await;
        cleanup_group_membership(membership_id_connected_1).await;
        cleanup_group_membership(membership_id_target_2).await;
        cleanup_group_membership(membership_id_connected_2).await;
        cleanup_invitation(invitation_target_group1.id).await;
        cleanup_invitation(invitation_connected_group1.id).await;
        cleanup_invitation(invitation_target_group2.id).await;
        cleanup_invitation(invitation_connected_group2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(target_user.email.clone()).await;
        cleanup_user_by_email(connected_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_empty_result_user_not_in_groups() {
        // Arrange: User with no group memberships
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (isolated_user, _) = create_test_user("find_connected_isolated").await;

        // Act: Find connected users for user with no memberships
        let result = repository.find_connected_users(isolated_user.id).await;

        // Assert: Should return empty vector
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        assert_eq!(connected_user_ids.len(), 0);

        // Cleanup
        cleanup_user_by_email(isolated_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_empty_result_user_in_group_alone() {
        // Arrange: User in a group with no other active members
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_owner_alone").await;
        let (lonely_user, _) = create_test_user("find_connected_lonely").await;
        
        let group_chat = create_test_group_chat("find_connected_alone", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, lonely_user.id, group_chat.id).await;
        
        // Create membership for lonely_user only
        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership).await.unwrap();

        // Act: Find connected users for lonely_user
        let result = repository.find_connected_users(lonely_user.id).await;

        // Assert: Should return empty vector (no other active members)
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        assert_eq!(connected_user_ids.len(), 0);

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(lonely_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_nonexistent_user() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Act: Find connected users for nonexistent user
        let result = repository.find_connected_users(-1).await;

        // Assert: Should return empty vector (not error)
        assert!(result.is_ok(), "Should handle nonexistent user gracefully");
        let connected_user_ids = result.unwrap();
        assert_eq!(connected_user_ids.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_excludes_inactive_memberships() {
        // Arrange: Create users with both active and inactive memberships
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_owner_inactive").await;
        let (target_user, _) = create_test_user("find_connected_target_inactive").await;
        let (connected_user, _) = create_test_user("find_connected_connected_inactive").await;
        let (inactive_user, _) = create_test_user("find_connected_inactive_inactive").await;
        
        let group_chat = create_test_group_chat("find_connected_inactive", owner_user.id).await;
        
        // Create invitations for all users
        let invitation_target = create_test_invitation(owner_user.id, target_user.id, group_chat.id).await;
        let invitation_connected = create_test_invitation(owner_user.id, connected_user.id, group_chat.id).await;
        let invitation_inactive = create_test_invitation(owner_user.id, inactive_user.id, group_chat.id).await;
        
        // Create active memberships for target and connected users
        let membership_target = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target.id);
        let membership_connected = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected.id);
        
        let membership_id_target = repository.insert(membership_target).await.unwrap();
        let membership_id_connected = repository.insert(membership_connected).await.unwrap();
        
        // Create membership for inactive user and then manually set it to left status
        let membership_inactive = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_inactive.id);
        let membership_id_inactive = repository.insert(membership_inactive).await.unwrap();
        
        // Update the inactive membership to 'left' status using raw SQL since there might not be a helper for this
        common::test_user_leave_from_a_group(inactive_user.id, group_chat.id).await;

        // Act: Find connected users for target_user
        let result = repository.find_connected_users(target_user.id).await;

        // Assert: Should only find connected_user, not inactive_user
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 1);
        assert!(connected_user_ids.contains(&connected_user.id));
        assert!(!connected_user_ids.contains(&inactive_user.id)); // Should not include left member

        // Cleanup
        cleanup_group_membership(membership_id_target).await;
        cleanup_group_membership(membership_id_connected).await;
        cleanup_group_membership(membership_id_inactive).await;
        cleanup_invitation(invitation_target.id).await;
        cleanup_invitation(invitation_connected.id).await;
        cleanup_invitation(invitation_inactive.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(target_user.email.clone()).await;
        cleanup_user_by_email(connected_user.email.clone()).await;
        cleanup_user_by_email(inactive_user.email.clone()).await;
    }
}
