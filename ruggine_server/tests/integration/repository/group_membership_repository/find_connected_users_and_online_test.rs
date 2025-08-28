use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;

#[cfg(test)]
mod group_membership_repository_find_connected_users_and_online_integration_tests {
    use ruggine_server::config::database::DatabaseTrait;
    use crate::common;
    use super::*;
    use crate::common::{
        get_database, create_test_user, create_test_group_chat, cleanup_group_chat,
        cleanup_user_by_email, cleanup_group_membership, create_test_invitation, cleanup_invitation,
        cleanup_test_user_from_a_group_chat
    };

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_success_single_online_user() {
        // Arrange: Create users sharing one group, with one online user
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_online_owner_single").await;
        let (target_user, _) = create_test_user("find_connected_online_target_single").await;
        let (connected_user, _) = create_test_user("find_connected_online_connected_single").await;
        
        let group_chat = create_test_group_chat("find_connected_online_single", owner_user.id).await;
        
        // Create invitations for both target_user and connected_user to the same group
        let invitation_target = create_test_invitation(owner_user.id, target_user.id, group_chat.id).await;
        let invitation_connected = create_test_invitation(owner_user.id, connected_user.id, group_chat.id).await;
        
        // Create active memberships for both users in the same group
        let new_membership_target = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target.id);
        let new_membership_connected = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected.id);
        
        let membership_id_target = group_membership_repository.insert(new_membership_target).await.unwrap();
        let membership_id_connected = group_membership_repository.insert(new_membership_connected).await.unwrap();

        // Set connected_user to online (target_user remains offline by default)
        user_repository.update_online(connected_user.id, true).await.unwrap();

        // Act: Find connected users that are online for target_user
        let result = group_membership_repository.find_connected_users_and_online(target_user.id).await;

        // Assert: Should find connected_user since they share the same group and connected_user is online
        assert!(result.is_ok(), "Failed to find connected users and online: {:?}", result);
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
    async fn test_find_connected_users_and_online_filters_offline_users() {
        // Arrange: Create users sharing one group, but connected user is offline
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_offline_owner").await;
        let (target_user, _) = create_test_user("find_connected_offline_target").await;
        let (connected_user, _) = create_test_user("find_connected_offline_connected").await;
        
        let group_chat = create_test_group_chat("find_connected_offline", owner_user.id).await;
        
        // Create invitations for both users to the same group
        let invitation_target = create_test_invitation(owner_user.id, target_user.id, group_chat.id).await;
        let invitation_connected = create_test_invitation(owner_user.id, connected_user.id, group_chat.id).await;
        
        // Create active memberships for both users
        let new_membership_target = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target.id);
        let new_membership_connected = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected.id);
        
        let membership_id_target = group_membership_repository.insert(new_membership_target).await.unwrap();
        let membership_id_connected = group_membership_repository.insert(new_membership_connected).await.unwrap();

        // Ensure connected_user is offline (default state)
        user_repository.update_online(connected_user.id, false).await.unwrap();

        // Act: Find connected users that are online for target_user
        let result = group_membership_repository.find_connected_users_and_online(target_user.id).await;

        // Assert: Should find no users since connected_user is offline
        assert!(result.is_ok(), "Failed to find connected users and online: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 0, "Should not find offline users");

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
    async fn test_find_connected_users_and_online_multiple_groups_mixed_status() {
        // Arrange: Create users in multiple groups with mixed online/offline status
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_multi_owner").await;
        let (target_user, _) = create_test_user("find_connected_multi_target").await;
        let (online_user1, _) = create_test_user("find_connected_multi_online1").await;
        let (online_user2, _) = create_test_user("find_connected_multi_online2").await;
        let (offline_user, _) = create_test_user("find_connected_multi_offline").await;
        
        // Create multiple group chats
        let group_chat1 = create_test_group_chat("find_connected_multi1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_connected_multi2", owner_user.id).await;
        
        // Create invitations - target_user shares:
        // - group1 with online_user1 and offline_user
        // - group2 with online_user2
        let invitation_target_group1 = create_test_invitation(owner_user.id, target_user.id, group_chat1.id).await;
        let invitation_online1_group1 = create_test_invitation(owner_user.id, online_user1.id, group_chat1.id).await;
        let invitation_offline_group1 = create_test_invitation(owner_user.id, offline_user.id, group_chat1.id).await;
        let invitation_target_group2 = create_test_invitation(owner_user.id, target_user.id, group_chat2.id).await;
        let invitation_online2_group2 = create_test_invitation(owner_user.id, online_user2.id, group_chat2.id).await;
        
        // Create active memberships
        let membership_target_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target_group1.id);
        let membership_online1_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_online1_group1.id);
        let membership_offline_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_offline_group1.id);
        let membership_target_2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target_group2.id);
        let membership_online2_2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_online2_group2.id);
        
        let membership_id_target_1 = group_membership_repository.insert(membership_target_1).await.unwrap();
        let membership_id_online1_1 = group_membership_repository.insert(membership_online1_1).await.unwrap();
        let membership_id_offline_1 = group_membership_repository.insert(membership_offline_1).await.unwrap();
        let membership_id_target_2 = group_membership_repository.insert(membership_target_2).await.unwrap();
        let membership_id_online2_2 = group_membership_repository.insert(membership_online2_2).await.unwrap();

        // Set online status for users
        user_repository.update_online(online_user1.id, true).await.unwrap();
        user_repository.update_online(online_user2.id, true).await.unwrap();
        user_repository.update_online(offline_user.id, false).await.unwrap();

        // Act: Find connected users that are online for target_user
        let result = group_membership_repository.find_connected_users_and_online(target_user.id).await;

        // Assert: Should find only the two online users
        assert!(result.is_ok(), "Failed to find connected users and online: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 2);
        assert!(connected_user_ids.contains(&online_user1.id));
        assert!(connected_user_ids.contains(&online_user2.id));
        assert!(!connected_user_ids.contains(&offline_user.id), "Should not include offline user");
        assert!(!connected_user_ids.contains(&target_user.id), "Should not include self");

        // Cleanup
        cleanup_group_membership(membership_id_target_1).await;
        cleanup_group_membership(membership_id_online1_1).await;
        cleanup_group_membership(membership_id_offline_1).await;
        cleanup_group_membership(membership_id_target_2).await;
        cleanup_group_membership(membership_id_online2_2).await;
        cleanup_invitation(invitation_target_group1.id).await;
        cleanup_invitation(invitation_online1_group1.id).await;
        cleanup_invitation(invitation_offline_group1.id).await;
        cleanup_invitation(invitation_target_group2.id).await;
        cleanup_invitation(invitation_online2_group2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(target_user.email.clone()).await;
        cleanup_user_by_email(online_user1.email.clone()).await;
        cleanup_user_by_email(online_user2.email.clone()).await;
        cleanup_user_by_email(offline_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_no_shared_groups() {
        // Arrange: Create users in different groups (no shared groups)
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user1, _) = create_test_user("find_connected_no_shared_owner1").await;
        let (owner_user2, _) = create_test_user("find_connected_no_shared_owner2").await;
        let (target_user, _) = create_test_user("find_connected_no_shared_target").await;
        let (other_user, _) = create_test_user("find_connected_no_shared_other").await;
        
        // Create separate group chats
        let group_chat1 = create_test_group_chat("find_connected_no_shared1", owner_user1.id).await;
        let group_chat2 = create_test_group_chat("find_connected_no_shared2", owner_user2.id).await;
        
        // target_user is in group1, other_user is in group2 (no shared groups)
        let invitation_target = create_test_invitation(owner_user1.id, target_user.id, group_chat1.id).await;
        let invitation_other = create_test_invitation(owner_user2.id, other_user.id, group_chat2.id).await;
        
        // Create active memberships
        let membership_target = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target.id);
        let membership_other = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_other.id);
        
        let membership_id_target = group_membership_repository.insert(membership_target).await.unwrap();
        let membership_id_other = group_membership_repository.insert(membership_other).await.unwrap();

        // Set other_user to online
        user_repository.update_online(other_user.id, true).await.unwrap();

        // Act: Find connected users that are online for target_user
        let result = group_membership_repository.find_connected_users_and_online(target_user.id).await;

        // Assert: Should find no users since they don't share any groups
        assert!(result.is_ok(), "Failed to find connected users and online: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 0, "Should not find users with no shared groups");

        // Cleanup
        cleanup_group_membership(membership_id_target).await;
        cleanup_group_membership(membership_id_other).await;
        cleanup_invitation(invitation_target.id).await;
        cleanup_invitation(invitation_other.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(owner_user1.email.clone()).await;
        cleanup_user_by_email(owner_user2.email.clone()).await;
        cleanup_user_by_email(target_user.email.clone()).await;
        cleanup_user_by_email(other_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_nonexistent_user() {
        // Arrange: Use a non-existent user ID
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let nonexistent_user_id = -999;

        // Act: Try to find connected users for non-existent user
        let result = group_membership_repository.find_connected_users_and_online(nonexistent_user_id).await;

        // Assert: Should return empty result for non-existent user
        assert!(result.is_ok(), "Should handle non-existent user gracefully");
        let connected_user_ids = result.unwrap();
        assert_eq!(connected_user_ids.len(), 0, "Should return empty result for non-existent user");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_distinct_results() {
        // Arrange: Create scenario where user shares multiple groups with same connected user
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_distinct_owner").await;
        let (target_user, _) = create_test_user("find_connected_distinct_target").await;
        let (connected_user, _) = create_test_user("find_connected_distinct_connected").await;
        
        // Create multiple group chats where target_user and connected_user are both members
        let group_chat1 = create_test_group_chat("find_connected_distinct1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_connected_distinct2", owner_user.id).await;
        
        // Create invitations for both users in both groups
        let invitation_target_group1 = create_test_invitation(owner_user.id, target_user.id, group_chat1.id).await;
        let invitation_connected_group1 = create_test_invitation(owner_user.id, connected_user.id, group_chat1.id).await;
        let invitation_target_group2 = create_test_invitation(owner_user.id, target_user.id, group_chat2.id).await;
        let invitation_connected_group2 = create_test_invitation(owner_user.id, connected_user.id, group_chat2.id).await;
        
        // Create active memberships for both users in both groups
        let membership_target_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target_group1.id);
        let membership_connected_1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected_group1.id);
        let membership_target_2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target_group2.id);
        let membership_connected_2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected_group2.id);
        
        let membership_id_target_1 = group_membership_repository.insert(membership_target_1).await.unwrap();
        let membership_id_connected_1 = group_membership_repository.insert(membership_connected_1).await.unwrap();
        let membership_id_target_2 = group_membership_repository.insert(membership_target_2).await.unwrap();
        let membership_id_connected_2 = group_membership_repository.insert(membership_connected_2).await.unwrap();

        // Set connected_user to online
        user_repository.update_online(connected_user.id, true).await.unwrap();

        // Act: Find connected users that are online for target_user
        let result = group_membership_repository.find_connected_users_and_online(target_user.id).await;

        // Assert: Should find connected_user only once despite sharing multiple groups (DISTINCT clause)
        assert!(result.is_ok(), "Failed to find connected users and online: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 1, "Should return distinct results even with multiple shared groups");
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
    async fn test_find_connected_users_and_online_user_changes_status() {
        // Arrange: Create users and test changing online status affects results
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_connected_status_owner").await;
        let (target_user, _) = create_test_user("find_connected_status_target").await;
        let (connected_user, _) = create_test_user("find_connected_status_connected").await;
        
        let group_chat = create_test_group_chat("find_connected_status", owner_user.id).await;
        
        // Create invitations and memberships
        let invitation_target = create_test_invitation(owner_user.id, target_user.id, group_chat.id).await;
        let invitation_connected = create_test_invitation(owner_user.id, connected_user.id, group_chat.id).await;
        
        let membership_target = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_target.id);
        let membership_connected = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_connected.id);
        
        let membership_id_target = group_membership_repository.insert(membership_target).await.unwrap();
        let membership_id_connected = group_membership_repository.insert(membership_connected).await.unwrap();

        // Test 1: User starts offline
        user_repository.update_online(connected_user.id, false).await.unwrap();
        let result_offline = group_membership_repository.find_connected_users_and_online(target_user.id).await.unwrap();
        assert_eq!(result_offline.len(), 0, "Should find no users when connected user is offline");

        // Test 2: User goes online
        user_repository.update_online(connected_user.id, true).await.unwrap();
        let result_online = group_membership_repository.find_connected_users_and_online(target_user.id).await.unwrap();
        assert_eq!(result_online.len(), 1, "Should find user when connected user is online");
        assert!(result_online.contains(&connected_user.id));

        // Test 3: User goes offline again
        user_repository.update_online(connected_user.id, false).await.unwrap();
        let result_offline_again = group_membership_repository.find_connected_users_and_online(target_user.id).await.unwrap();
        assert_eq!(result_offline_again.len(), 0, "Should find no users when connected user goes offline again");

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
}
