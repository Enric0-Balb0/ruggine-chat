use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;

#[cfg(test)]
mod group_membership_repository_find_online_users_in_group_integration_tests {
    use ruggine_server::config::database::DatabaseTrait;
    use crate::common;
    use super::*;
    use crate::common::{
        get_database, create_test_user, create_test_group_chat, cleanup_group_chat,
        cleanup_user_by_email, cleanup_group_membership, create_test_invitation, cleanup_invitation
    };

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_success_single_online_user() {
        // Arrange: Create a group with multiple users, only one online
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_online_group_owner_single").await;
        let (online_user, _) = create_test_user("find_online_group_online_single").await;
        let (offline_user, _) = create_test_user("find_online_group_offline_single").await;
        
        let group_chat = create_test_group_chat("find_online_group_single", owner_user.id).await;
        
        // Create invitations for both users to the group
        let invitation_online = create_test_invitation(owner_user.id, online_user.id, group_chat.id).await;
        let invitation_offline = create_test_invitation(owner_user.id, offline_user.id, group_chat.id).await;
        
        // Create active memberships for both users
        let new_membership_online = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_online.id);
        let new_membership_offline = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_offline.id);
        
        let membership_id_online = group_membership_repository.insert(new_membership_online).await.unwrap();
        let membership_id_offline = group_membership_repository.insert(new_membership_offline).await.unwrap();

        // Set online_user to online and offline_user to offline
        user_repository.update_online(online_user.id, true).await.unwrap();
        user_repository.update_online(offline_user.id, false).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_repository.find_online_users_in_group(group_chat.id).await;

        // Assert: Should find only the online user
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 1, "Should find exactly one online user");
        assert!(online_user_ids.contains(&online_user.id), "Should contain the online user");
        assert!(!online_user_ids.contains(&offline_user.id), "Should not contain the offline user");

        // Cleanup
        cleanup_group_membership(membership_id_online).await;
        cleanup_group_membership(membership_id_offline).await;
        cleanup_invitation(invitation_online.id).await;
        cleanup_invitation(invitation_offline.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(online_user.email.clone()).await;
        cleanup_user_by_email(offline_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_multiple_online_users() {
        // Arrange: Create a group with multiple online users
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_online_group_owner_multi").await;
        let (online_user1, _) = create_test_user("find_online_group_online1_multi").await;
        let (online_user2, _) = create_test_user("find_online_group_online2_multi").await;
        let (offline_user, _) = create_test_user("find_online_group_offline_multi").await;
        
        let group_chat = create_test_group_chat("find_online_group_multi", owner_user.id).await;
        
        // Create invitations for all users to the group
        let invitation_online1 = create_test_invitation(owner_user.id, online_user1.id, group_chat.id).await;
        let invitation_online2 = create_test_invitation(owner_user.id, online_user2.id, group_chat.id).await;
        let invitation_offline = create_test_invitation(owner_user.id, offline_user.id, group_chat.id).await;
        
        // Create active memberships for all users
        let new_membership_online1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_online1.id);
        let new_membership_online2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_online2.id);
        let new_membership_offline = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_offline.id);
        
        let membership_id_online1 = group_membership_repository.insert(new_membership_online1).await.unwrap();
        let membership_id_online2 = group_membership_repository.insert(new_membership_online2).await.unwrap();
        let membership_id_offline = group_membership_repository.insert(new_membership_offline).await.unwrap();

        // Set two users online and one offline
        user_repository.update_online(online_user1.id, true).await.unwrap();
        user_repository.update_online(online_user2.id, true).await.unwrap();
        user_repository.update_online(offline_user.id, false).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_repository.find_online_users_in_group(group_chat.id).await;

        // Assert: Should find both online users
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 2, "Should find exactly two online users");
        assert!(online_user_ids.contains(&online_user1.id), "Should contain first online user");
        assert!(online_user_ids.contains(&online_user2.id), "Should contain second online user");
        assert!(!online_user_ids.contains(&offline_user.id), "Should not contain the offline user");

        // Cleanup
        cleanup_group_membership(membership_id_online1).await;
        cleanup_group_membership(membership_id_online2).await;
        cleanup_group_membership(membership_id_offline).await;
        cleanup_invitation(invitation_online1.id).await;
        cleanup_invitation(invitation_online2.id).await;
        cleanup_invitation(invitation_offline.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(online_user1.email.clone()).await;
        cleanup_user_by_email(online_user2.email.clone()).await;
        cleanup_user_by_email(offline_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_no_online_users() {
        // Arrange: Create a group where all users are offline
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_online_group_owner_none").await;
        let (offline_user1, _) = create_test_user("find_online_group_offline1_none").await;
        let (offline_user2, _) = create_test_user("find_online_group_offline2_none").await;
        
        let group_chat = create_test_group_chat("find_online_group_none", owner_user.id).await;
        
        // Create invitations for all users to the group
        let invitation_offline1 = create_test_invitation(owner_user.id, offline_user1.id, group_chat.id).await;
        let invitation_offline2 = create_test_invitation(owner_user.id, offline_user2.id, group_chat.id).await;
        
        // Create active memberships for all users
        let new_membership_offline1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_offline1.id);
        let new_membership_offline2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_offline2.id);
        
        let membership_id_offline1 = group_membership_repository.insert(new_membership_offline1).await.unwrap();
        let membership_id_offline2 = group_membership_repository.insert(new_membership_offline2).await.unwrap();

        // Ensure all users are offline
        user_repository.update_online(offline_user1.id, false).await.unwrap();
        user_repository.update_online(offline_user2.id, false).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_repository.find_online_users_in_group(group_chat.id).await;

        // Assert: Should find no online users
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 0, "Should find no online users");

        // Cleanup
        cleanup_group_membership(membership_id_offline1).await;
        cleanup_group_membership(membership_id_offline2).await;
        cleanup_invitation(invitation_offline1.id).await;
        cleanup_invitation(invitation_offline2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(offline_user1.email.clone()).await;
        cleanup_user_by_email(offline_user2.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_empty_group() {
        // Arrange: Create a group with no members
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);

        let (owner_user, _) = create_test_user("find_online_group_owner_empty").await;
        let group_chat = create_test_group_chat("find_online_group_empty", owner_user.id).await;

        // Act: Find online users in the empty group
        let result = group_membership_repository.find_online_users_in_group(group_chat.id).await;

        // Assert: Should find no users
        assert!(result.is_ok(), "Failed to find online users in empty group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 0, "Should find no users in empty group");

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_nonexistent_group() {
        // Arrange: Use a non-existent group ID
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let nonexistent_group_id = 999999;

        // Act: Find online users in the non-existent group
        let result = group_membership_repository.find_online_users_in_group(nonexistent_group_id).await;

        // Assert: Should succeed but return empty list
        assert!(result.is_ok(), "Failed to handle non-existent group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 0, "Should find no users for non-existent group");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_filters_inactive_memberships() {
        // Arrange: Create a group with some users having inactive memberships
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_online_group_owner_inactive").await;
        let (active_online_user, _) = create_test_user("find_online_group_active_online").await;
        let (inactive_online_user, _) = create_test_user("find_online_group_inactive_online").await;
        
        let group_chat = create_test_group_chat("find_online_group_inactive", owner_user.id).await;
        
        // Create invitations for both users
        let invitation_active = create_test_invitation(owner_user.id, active_online_user.id, group_chat.id).await;
        let invitation_inactive = create_test_invitation(owner_user.id, inactive_online_user.id, group_chat.id).await;
        
        // Create one active and one inactive membership
        let active_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_active.id);
        let mut inactive_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_inactive.id);
        
        let membership_id_active = group_membership_repository.insert(active_membership).await.unwrap();
        let membership_id_inactive = group_membership_repository.insert(inactive_membership).await.unwrap();
        common::test_user_leave_from_a_group(inactive_online_user.id, group_chat.id).await;

        // Set both users online
        user_repository.update_online(active_online_user.id, true).await.unwrap();
        user_repository.update_online(inactive_online_user.id, true).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_repository.find_online_users_in_group(group_chat.id).await;

        // Assert: Should find only the user with active membership
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 1, "Should find only one user with active membership");
        assert!(online_user_ids.contains(&active_online_user.id), "Should contain the user with active membership");
        assert!(!online_user_ids.contains(&inactive_online_user.id), "Should not contain user with inactive membership");

        // Cleanup
        cleanup_group_membership(membership_id_active).await;
        cleanup_group_membership(membership_id_inactive).await;
        cleanup_invitation(invitation_active.id).await;
        cleanup_invitation(invitation_inactive.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(active_online_user.email.clone()).await;
        cleanup_user_by_email(inactive_online_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_filters_inactive_users() {
        // Arrange: Create a group where some users have inactive status
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_online_group_owner_user_inactive").await;
        let (active_user, _) = create_test_user("find_online_group_active_user").await;
        let (inactive_status_user, _) = create_test_user("find_online_group_inactive_status_user").await;
        
        let group_chat = create_test_group_chat("find_online_group_user_inactive", owner_user.id).await;
        
        // Create invitations for both users
        let invitation_active = create_test_invitation(owner_user.id, active_user.id, group_chat.id).await;
        let invitation_inactive_status = create_test_invitation(owner_user.id, inactive_status_user.id, group_chat.id).await;
        
        // Create active memberships for both users
        let active_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_active.id);
        let inactive_status_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation_inactive_status.id);
        
        let membership_id_active = group_membership_repository.insert(active_membership).await.unwrap();
        let membership_id_inactive_status = group_membership_repository.insert(inactive_status_membership).await.unwrap();

        // Set both users online
        user_repository.update_online(active_user.id, true).await.unwrap();
        user_repository.update_online(inactive_status_user.id, true).await.unwrap();
        
        // Set one user status to inactive using direct SQL
        let result = sqlx::query(
            r#"UPDATE "user" SET user_status = 'banned' WHERE id = $1"#
        )
            .bind(inactive_status_user.id)
            .execute(db.get_pool())
            .await;
        assert!(result.is_ok(), "Failed to set user status to banned");

        // Act: Find online users in the group
        let result = group_membership_repository.find_online_users_in_group(group_chat.id).await;

        // Assert: Should find only the user with active status
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 1, "Should find only one user with active status");
        assert!(online_user_ids.contains(&active_user.id), "Should contain the user with active status");
        assert!(!online_user_ids.contains(&inactive_status_user.id), "Should not contain user with inactive status");

        // Cleanup
        cleanup_group_membership(membership_id_active).await;
        cleanup_group_membership(membership_id_inactive_status).await;
        cleanup_invitation(invitation_active.id).await;
        cleanup_invitation(invitation_inactive_status.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(active_user.email.clone()).await;
        cleanup_user_by_email(inactive_status_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_distinct_users() {
        // Arrange: Test that the function returns distinct user IDs
        // This is important if there are multiple memberships for the same user in the group
        let db = get_database().await;
        let group_membership_repository = GroupMembershipRepository::new(&db);
        let user_repository = UserRepository::new(&db);

        let (owner_user, _) = create_test_user("find_online_group_owner_distinct").await;
        let (online_user, _) = create_test_user("find_online_group_online_distinct").await;
        
        let group_chat = create_test_group_chat("find_online_group_distinct", owner_user.id).await;
        
        // Create invitation for the user
        let invitation = create_test_invitation(owner_user.id, online_user.id, group_chat.id).await;
        
        // Create active membership
        let membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = group_membership_repository.insert(membership).await.unwrap();

        // Set user online
        user_repository.update_online(online_user.id, true).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_repository.find_online_users_in_group(group_chat.id).await;

        // Assert: Should find the user exactly once despite the DISTINCT clause in SQL
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 1, "Should find exactly one instance of the user");
        assert!(online_user_ids.contains(&online_user.id), "Should contain the online user");
        
        // Count occurrences to ensure no duplicates
        let user_count = online_user_ids.iter().filter(|&&id| id == online_user.id).count();
        assert_eq!(user_count, 1, "User should appear exactly once in results");

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(online_user.email.clone()).await;
    }
}
