use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use crate::common::{
    get_database, create_test_user, create_test_group_chat, create_test_group_membership,
    cleanup_user_by_email, cleanup_group_chat, create_test_invitation, cleanup_test_user_from_a_group_chat
};

#[cfg(test)]
mod group_membership_find_connected_users_integration_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_success_single_shared_group() {
        // Arrange: Create users sharing one group
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_connected_owner_single").await;
        let (target_user, _) = create_test_user("find_connected_target_single").await;
        let (connected_user, _) = create_test_user("find_connected_connected_single").await;
        
        let group_chat = create_test_group_chat("find_connected_single", owner_user.id).await;
        
        // Create invitations for both target_user and connected_user to the same group
        let invitation_target = create_test_invitation(owner_user.id, target_user.id, group_chat.id).await;
        let invitation_connected = create_test_invitation(owner_user.id, connected_user.id, group_chat.id).await;
        
        // Create active memberships for both users in the same group
        let _membership_target = create_test_group_membership(invitation_target.id, target_user.id).await;
        let _membership_connected = create_test_group_membership(invitation_connected.id, connected_user.id).await;

        // Act: Find connected users for target_user
        let result = service.find_connected_users(target_user.id).await;

        // Assert: Should find connected_user as they share the same group
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 1);
        assert!(connected_user_ids.contains(&connected_user.id));
        assert!(!connected_user_ids.contains(&target_user.id)); // Should not include self

        // Cleanup: Use the specific cleanup method for users in groups
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(target_user.email.clone()).await;
        cleanup_user_by_email(connected_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_multiple_shared_groups() {
        // Arrange: Create users sharing multiple groups
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
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
        let _membership_target_1 = create_test_group_membership(invitation_target_group1.id, target_user.id).await;
        let _membership_connected1_1 = create_test_group_membership(invitation_connected1_group1.id, connected_user1.id).await;
        let _membership_target_2 = create_test_group_membership(invitation_target_group2.id, target_user.id).await;
        let _membership_connected2_2 = create_test_group_membership(invitation_connected2_group2.id, connected_user2.id).await;

        // Act: Find connected users for target_user
        let result = service.find_connected_users(target_user.id).await;

        // Assert: Should find both connected users
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 2);
        assert!(connected_user_ids.contains(&connected_user1.id));
        assert!(connected_user_ids.contains(&connected_user2.id));
        assert!(!connected_user_ids.contains(&target_user.id)); // Should not include self

        // Cleanup: Clean up each user from their respective groups
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(connected_user1.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(connected_user2.id, group_chat2.id).await;
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
        let service = GroupMembershipService::new(&db);
        
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
        let _membership_target_1 = create_test_group_membership(invitation_target_group1.id, target_user.id).await;
        let _membership_connected_1 = create_test_group_membership(invitation_connected_group1.id, connected_user.id).await;
        let _membership_target_2 = create_test_group_membership(invitation_target_group2.id, target_user.id).await;
        let _membership_connected_2 = create_test_group_membership(invitation_connected_group2.id, connected_user.id).await;

        // Act: Find connected users for target_user
        let result = service.find_connected_users(target_user.id).await;

        // Assert: Should find connected_user only once (DISTINCT should prevent duplicates)
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 1);
        assert!(connected_user_ids.contains(&connected_user.id));

        // Cleanup: Clean up users from both groups
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(connected_user.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(connected_user.id, group_chat2.id).await;
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
        let service = GroupMembershipService::new(&db);
        
        let (isolated_user, _) = create_test_user("find_connected_isolated").await;

        // Act: Find connected users for user with no memberships
        let result = service.find_connected_users(isolated_user.id).await;

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
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_connected_owner_alone").await;
        let (lonely_user, _) = create_test_user("find_connected_lonely").await;
        
        let group_chat = create_test_group_chat("find_connected_alone", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, lonely_user.id, group_chat.id).await;
        
        // Create membership for lonely_user only
        let _membership = create_test_group_membership(invitation.id, lonely_user.id).await;

        // Act: Find connected users for lonely_user
        let result = service.find_connected_users(lonely_user.id).await;

        // Assert: Should return empty vector (no other active members)
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        assert_eq!(connected_user_ids.len(), 0);

        // Cleanup
        cleanup_test_user_from_a_group_chat(lonely_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(lonely_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_nonexistent_user() {
        // Arrange
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);

        // Act: Find connected users for nonexistent user
        let result = service.find_connected_users(-1).await;

        // Assert: Should return empty vector (not error)
        assert!(result.is_ok(), "Should handle nonexistent user gracefully");
        let connected_user_ids = result.unwrap();
        assert_eq!(connected_user_ids.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_complex_scenario_with_multiple_groups_and_users() {
        // Arrange: Complex scenario with multiple users and groups
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_connected_owner_complex").await;
        let (target_user, _) = create_test_user("find_connected_target_complex").await;
        let (connected_user1, _) = create_test_user("find_connected_connected1_complex").await;
        let (connected_user2, _) = create_test_user("find_connected_connected2_complex").await;
        let (connected_user3, _) = create_test_user("find_connected_connected3_complex").await;
        let (unrelated_user, _) = create_test_user("find_connected_unrelated_complex").await;
        
        // Create multiple group chats
        let group_chat1 = create_test_group_chat("find_connected_complex1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_connected_complex2", owner_user.id).await;
        let group_chat3 = create_test_group_chat("find_connected_complex3", owner_user.id).await;
        
        // Target user is in group1 and group2
        let invitation_target_group1 = create_test_invitation(owner_user.id, target_user.id, group_chat1.id).await;
        let invitation_target_group2 = create_test_invitation(owner_user.id, target_user.id, group_chat2.id).await;
        
        // Connected user1 is in group1 with target
        let invitation_connected1_group1 = create_test_invitation(owner_user.id, connected_user1.id, group_chat1.id).await;
        
        // Connected user2 is in both group1 and group2 with target
        let invitation_connected2_group1 = create_test_invitation(owner_user.id, connected_user2.id, group_chat1.id).await;
        let invitation_connected2_group2 = create_test_invitation(owner_user.id, connected_user2.id, group_chat2.id).await;
        
        // Connected user3 is in group2 with target
        let invitation_connected3_group2 = create_test_invitation(owner_user.id, connected_user3.id, group_chat2.id).await;
        
        // Unrelated user is only in group3 (not connected to target)
        let invitation_unrelated_group3 = create_test_invitation(owner_user.id, unrelated_user.id, group_chat3.id).await;
        
        // Create all memberships
        let _membership_target_1 = create_test_group_membership(invitation_target_group1.id, target_user.id).await;
        let _membership_target_2 = create_test_group_membership(invitation_target_group2.id, target_user.id).await;
        let _membership_connected1_1 = create_test_group_membership(invitation_connected1_group1.id, connected_user1.id).await;
        let _membership_connected2_1 = create_test_group_membership(invitation_connected2_group1.id, connected_user2.id).await;
        let _membership_connected2_2 = create_test_group_membership(invitation_connected2_group2.id, connected_user2.id).await;
        let _membership_connected3_2 = create_test_group_membership(invitation_connected3_group2.id, connected_user3.id).await;
        let _membership_unrelated_3 = create_test_group_membership(invitation_unrelated_group3.id, unrelated_user.id).await;

        // Act: Find connected users for target_user
        let result = service.find_connected_users(target_user.id).await;

        // Assert: Should find connected_user1, connected_user2, and connected_user3, but not unrelated_user
        assert!(result.is_ok(), "Failed to find connected users: {:?}", result);
        let connected_user_ids = result.unwrap();
        
        assert_eq!(connected_user_ids.len(), 3);
        assert!(connected_user_ids.contains(&connected_user1.id));
        assert!(connected_user_ids.contains(&connected_user2.id));
        assert!(connected_user_ids.contains(&connected_user3.id));
        assert!(!connected_user_ids.contains(&unrelated_user.id)); // Should not include user from different group
        assert!(!connected_user_ids.contains(&target_user.id)); // Should not include self

        // Cleanup: Clean up all users from their respective groups
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(connected_user1.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(connected_user2.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(connected_user2.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(connected_user3.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(unrelated_user.id, group_chat3.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_group_chat(group_chat3.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(target_user.email.clone()).await;
        cleanup_user_by_email(connected_user1.email.clone()).await;
        cleanup_user_by_email(connected_user2.email.clone()).await;
        cleanup_user_by_email(connected_user3.email.clone()).await;
        cleanup_user_by_email(unrelated_user.email.clone()).await;
    }
}
