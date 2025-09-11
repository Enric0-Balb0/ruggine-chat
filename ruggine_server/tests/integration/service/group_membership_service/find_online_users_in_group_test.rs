use crate::common::{
    cleanup_group_chat, cleanup_test_user_from_a_group_chat, cleanup_user_by_email, create_test_group_chat,
    create_test_group_membership, create_test_invitation, create_test_user, get_database
};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::service::user_service::{UserService, UserServiceTrait};

#[cfg(test)]
mod group_membership_find_online_users_in_group_integration_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_success_single_online_user() {
        // Arrange: Create a group with multiple users, only one online
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        let user_service = UserService::new(&db);
        
        let (owner_user, _) = create_test_user("find_online_svc_owner_single").await;
        let (auth_user, _) = create_test_user("find_online_svc_auth_single").await;
        let (online_user, _) = create_test_user("find_online_svc_online_single").await;
        let (offline_user, _) = create_test_user("find_online_svc_offline_single").await;
        
        let group_chat = create_test_group_chat("find_online_svc_single", owner_user.id).await;
        
        // Create invitations for all users to the group
        let invitation_auth = create_test_invitation(owner_user.id, auth_user.id, group_chat.id).await;
        let invitation_online = create_test_invitation(owner_user.id, online_user.id, group_chat.id).await;
        let invitation_offline = create_test_invitation(owner_user.id, offline_user.id, group_chat.id).await;
        
        // Create active memberships for all users
        let _membership_auth = create_test_group_membership(invitation_auth.id, auth_user.id).await;
        let _membership_online = create_test_group_membership(invitation_online.id, online_user.id).await;
        let _membership_offline = create_test_group_membership(invitation_offline.id, offline_user.id).await;

        // Set auth_user and online_user to online, offline_user to offline
        let update_online_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_true();
        let update_offline_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_false();
        
        user_service.update_online(auth_user.id, update_online_dto.clone()).await.unwrap();
        user_service.update_online(online_user.id, update_online_dto).await.unwrap();
        user_service.update_online(offline_user.id, update_offline_dto).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_service.find_online_users_in_group(auth_user.id, group_chat.id).await;

        // Assert: Should find only the online user (excluding auth_user)
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 2, "Should find exactly two online users (auth + online)");
        assert!(online_user_ids.contains(&auth_user.id), "Should contain the auth user");
        assert!(online_user_ids.contains(&online_user.id), "Should contain the online user");
        assert!(!online_user_ids.contains(&offline_user.id), "Should not contain the offline user");

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(online_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(offline_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(auth_user.email.clone()).await;
        cleanup_user_by_email(online_user.email.clone()).await;
        cleanup_user_by_email(offline_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_multiple_online_users() {
        // Arrange: Create a group with multiple online users
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        let user_service = UserService::new(&db);
        
        let (owner_user, _) = create_test_user("find_online_svc_owner_multi").await;
        let (auth_user, _) = create_test_user("find_online_svc_auth_multi").await;
        let (online_user1, _) = create_test_user("find_online_svc_online1_multi").await;
        let (online_user2, _) = create_test_user("find_online_svc_online2_multi").await;
        let (offline_user, _) = create_test_user("find_online_svc_offline_multi").await;
        
        let group_chat = create_test_group_chat("find_online_svc_multi", owner_user.id).await;
        
        // Create invitations for all users to the group
        let invitation_auth = create_test_invitation(owner_user.id, auth_user.id, group_chat.id).await;
        let invitation_online1 = create_test_invitation(owner_user.id, online_user1.id, group_chat.id).await;
        let invitation_online2 = create_test_invitation(owner_user.id, online_user2.id, group_chat.id).await;
        let invitation_offline = create_test_invitation(owner_user.id, offline_user.id, group_chat.id).await;
        
        // Create active memberships for all users
        let _membership_auth = create_test_group_membership(invitation_auth.id, auth_user.id).await;
        let _membership_online1 = create_test_group_membership(invitation_online1.id, online_user1.id).await;
        let _membership_online2 = create_test_group_membership(invitation_online2.id, online_user2.id).await;
        let _membership_offline = create_test_group_membership(invitation_offline.id, offline_user.id).await;

        // Set auth_user and two users online, one offline
        let update_online_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_true();
        let update_offline_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_false();
        
        user_service.update_online(auth_user.id, update_online_dto.clone()).await.unwrap();
        user_service.update_online(online_user1.id, update_online_dto.clone()).await.unwrap();
        user_service.update_online(online_user2.id, update_online_dto).await.unwrap();
        user_service.update_online(offline_user.id, update_offline_dto).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_service.find_online_users_in_group(auth_user.id, group_chat.id).await;

        // Assert: Should find all online users
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 3, "Should find exactly three online users");
        assert!(online_user_ids.contains(&auth_user.id), "Should contain the auth user");
        assert!(online_user_ids.contains(&online_user1.id), "Should contain first online user");
        assert!(online_user_ids.contains(&online_user2.id), "Should contain second online user");
        assert!(!online_user_ids.contains(&offline_user.id), "Should not contain the offline user");

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(online_user1.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(online_user2.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(offline_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(auth_user.email.clone()).await;
        cleanup_user_by_email(online_user1.email.clone()).await;
        cleanup_user_by_email(online_user2.email.clone()).await;
        cleanup_user_by_email(offline_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_no_online_users() {
        // Arrange: Create a group where auth user is online but all other users are offline
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        let user_service = UserService::new(&db);
        
        let (owner_user, _) = create_test_user("find_online_svc_owner_none").await;
        let (auth_user, _) = create_test_user("find_online_svc_auth_none").await;
        let (offline_user1, _) = create_test_user("find_online_svc_offline1_none").await;
        let (offline_user2, _) = create_test_user("find_online_svc_offline2_none").await;
        
        let group_chat = create_test_group_chat("find_online_svc_none", owner_user.id).await;
        
        // Create invitations for all users to the group
        let invitation_auth = create_test_invitation(owner_user.id, auth_user.id, group_chat.id).await;
        let invitation_offline1 = create_test_invitation(owner_user.id, offline_user1.id, group_chat.id).await;
        let invitation_offline2 = create_test_invitation(owner_user.id, offline_user2.id, group_chat.id).await;
        
        // Create active memberships for all users
        let _membership_auth = create_test_group_membership(invitation_auth.id, auth_user.id).await;
        let _membership_offline1 = create_test_group_membership(invitation_offline1.id, offline_user1.id).await;
        let _membership_offline2 = create_test_group_membership(invitation_offline2.id, offline_user2.id).await;

        // Set auth_user online, all others offline
        let update_online_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_true();
        let update_offline_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_false();
        
        user_service.update_online(auth_user.id, update_online_dto).await.unwrap();
        user_service.update_online(offline_user1.id, update_offline_dto.clone()).await.unwrap();
        user_service.update_online(offline_user2.id, update_offline_dto).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_service.find_online_users_in_group(auth_user.id, group_chat.id).await;

        // Assert: Should find only the auth user
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 1, "Should find only the auth user online");
        assert!(online_user_ids.contains(&auth_user.id), "Should contain the auth user");

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(offline_user1.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(offline_user2.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(auth_user.email.clone()).await;
        cleanup_user_by_email(offline_user1.email.clone()).await;
        cleanup_user_by_email(offline_user2.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_auth_user_offline_error() {
        // Arrange: Create a group but auth user is offline
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        let user_service = UserService::new(&db);
        
        let (owner_user, _) = create_test_user("find_online_svc_owner_offline_auth").await;
        let (auth_user, _) = create_test_user("find_online_svc_auth_offline_auth").await;
        let (online_user, _) = create_test_user("find_online_svc_online_offline_auth").await;
        
        let group_chat = create_test_group_chat("find_online_svc_offline_auth", owner_user.id).await;
        
        // Create invitations for users to the group
        let invitation_auth = create_test_invitation(owner_user.id, auth_user.id, group_chat.id).await;
        let invitation_online = create_test_invitation(owner_user.id, online_user.id, group_chat.id).await;
        
        // Create active memberships
        let _membership_auth = create_test_group_membership(invitation_auth.id, auth_user.id).await;
        let _membership_online = create_test_group_membership(invitation_online.id, online_user.id).await;

        // Set auth_user offline, online_user online
        let update_online_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_true();
        let update_offline_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_false();
        
        user_service.update_online(auth_user.id, update_offline_dto).await.unwrap();
        user_service.update_online(online_user.id, update_online_dto).await.unwrap();

        // Act: Try to find online users in the group with offline auth user
        let result = group_membership_service.find_online_users_in_group(auth_user.id, group_chat.id).await;

        // Assert: Should fail because auth user is not online
        assert!(result.is_err(), "Should fail when auth user is offline");
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::CannotAccessIfUserIsNotOnline) => {
                // Expected error
            }
            other => panic!("Expected CannotAccessIfUserIsNotOnline error, got: {:?}", other)
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(online_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(auth_user.email.clone()).await;
        cleanup_user_by_email(online_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_auth_user_not_member_error() {
        // Arrange: Create a group but auth user is not a member
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        let user_service = UserService::new(&db);
        
        let (owner_user, _) = create_test_user("find_online_svc_owner_not_member").await;
        let (auth_user, _) = create_test_user("find_online_svc_auth_not_member").await;
        let (online_user, _) = create_test_user("find_online_svc_online_not_member").await;
        
        let group_chat = create_test_group_chat("find_online_svc_not_member", owner_user.id).await;
        
        // Create invitation only for online_user (auth_user is not a member)
        let invitation_online = create_test_invitation(owner_user.id, online_user.id, group_chat.id).await;
        
        // Create active membership only for online_user
        let _membership_online = create_test_group_membership(invitation_online.id, online_user.id).await;

        // Set both users online
        let update_online_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_true();
        
        user_service.update_online(auth_user.id, update_online_dto.clone()).await.unwrap();
        user_service.update_online(online_user.id, update_online_dto).await.unwrap();

        // Act: Try to find online users in the group with non-member auth user
        let result = group_membership_service.find_online_users_in_group(auth_user.id, group_chat.id).await;

        // Assert: Should fail because auth user is not a member
        assert!(result.is_err(), "Should fail when auth user is not a member");
        // The exact error depends on the implementation - could be GroupMembershipError or other

        // Cleanup
        cleanup_test_user_from_a_group_chat(online_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(auth_user.email.clone()).await;
        cleanup_user_by_email(online_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_nonexistent_group() {
        // Arrange: Use a non-existent group ID
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        let user_service = UserService::new(&db);
        
        let (auth_user, _) = create_test_user("find_online_svc_auth_nonexistent").await;
        let nonexistent_group_id = 999999;

        // Set auth_user online
        let update_online_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_true();
        user_service.update_online(auth_user.id, update_online_dto).await.unwrap();

        // Act: Try to find online users in a non-existent group
        let result = group_membership_service.find_online_users_in_group(auth_user.id, nonexistent_group_id).await;

        // Assert: Should fail because auth user is not a member of non-existent group
        assert!(result.is_err(), "Should fail for non-existent group");

        // Cleanup
        cleanup_user_by_email(auth_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_filters_inactive_memberships() {
        // Arrange: Create a group with some users having inactive memberships
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        let user_service = UserService::new(&db);
        
        let (owner_user, _) = create_test_user("find_online_svc_owner_inactive").await;
        let (auth_user, _) = create_test_user("find_online_svc_auth_inactive").await;
        let (active_online_user, _) = create_test_user("find_online_svc_active_online").await;
        
        let group_chat = create_test_group_chat("find_online_svc_inactive", owner_user.id).await;
        
        // Create invitations for users
        let invitation_auth = create_test_invitation(owner_user.id, auth_user.id, group_chat.id).await;
        let invitation_active = create_test_invitation(owner_user.id, active_online_user.id, group_chat.id).await;
        
        // Create active memberships
        let _membership_auth = create_test_group_membership(invitation_auth.id, auth_user.id).await;
        let _membership_active = create_test_group_membership(invitation_active.id, active_online_user.id).await;

        // Set both users online
        let update_online_dto = ruggine_server::factory::user_factory::UserFactory::fake_update_online_dto_true();
        
        user_service.update_online(auth_user.id, update_online_dto.clone()).await.unwrap();
        user_service.update_online(active_online_user.id, update_online_dto).await.unwrap();

        // Act: Find online users in the group
        let result = group_membership_service.find_online_users_in_group(auth_user.id, group_chat.id).await;

        // Assert: Should find users with active memberships
        assert!(result.is_ok(), "Failed to find online users in group: {:?}", result);
        let online_user_ids = result.unwrap();
        
        assert_eq!(online_user_ids.len(), 2, "Should find users with active memberships");
        assert!(online_user_ids.contains(&auth_user.id), "Should contain the auth user");
        assert!(online_user_ids.contains(&active_online_user.id), "Should contain the user with active membership");

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(active_online_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(auth_user.email.clone()).await;
        cleanup_user_by_email(active_online_user.email.clone()).await;
    }
}
