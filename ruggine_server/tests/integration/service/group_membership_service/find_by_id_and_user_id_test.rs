use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use crate::common::{get_database, create_test_user, create_test_group_chat, create_test_group_membership, cleanup_user, cleanup_group_chat, cleanup_group_membership};

#[cfg(test)]
mod group_membership_find_by_id_and_user_id_integration_tests {
    use crate::{cleanup_invitation, create_test_invitation};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_success() {
        // Arrange: Create real user, group chat, and membership in the database
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        // Create a unique user, group and invitation
        let (owner_user, _password) = create_test_user("find_owner").await;
        let group_chat = create_test_group_chat("find_test", owner_user.id).await;
        let (member_user, _member_password) = create_test_user("find_member").await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create membership using common helper
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Act: Find the membership by ID and user ID
        let result = service.find_by_id_and_user_id(membership.id, member_user.id).await;

        // Assert: Verify membership was found successfully
        assert!(result.is_ok(), "Failed to find group membership: {:?}", result);
        let membership_dto = result.unwrap();
        
        assert_eq!(membership_dto.id, membership.id);
        assert_eq!(membership_dto.user_id, member_user.id);
        assert_eq!(membership_dto.group_chat_id, group_chat.id);
        assert_eq!(membership_dto.role, membership.role);
        assert_eq!(membership_dto.membership_status, membership.membership_status);
        assert_eq!(membership_dto.joined_at, membership.joined_at);
        assert_eq!(membership_dto.left_at, membership.left_at);

        // Cleanup: Delete the test data
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(owner_user.email.clone()).await;
        cleanup_user(member_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_not_found() {
        // Arrange: Set up service with non-existent membership ID
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        // Create a real user for the test
        let (user, _password) = create_test_user("find_not_found").await;
        
        // Use a non-existent membership ID
        let nonexistent_membership_id = -1;

        // Act: Try to find non-existent membership
        let result = service.find_by_id_and_user_id(nonexistent_membership_id, user.id).await;

        // Assert: Should fail with GroupMembershipNotFound error
        assert!(result.is_err(), "Should fail for non-existent membership");
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            other => panic!("Expected GroupMembershipNotFound error, got: {:?}", other),
        }

        // Cleanup: Delete the test user
        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_wrong_user() {
        // Arrange: Create membership and try to access with different user
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        // Create users, group and invitation
        let (owner_user, _password) = create_test_user("find_wrong_owner").await;
        let group_chat = create_test_group_chat("find_wrong_test", owner_user.id).await;
        let (member_user, _member_password) = create_test_user("find_wrong_member").await;
        let (other_user, _other_password) = create_test_user("find_wrong_other").await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create membership for member_user
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Act: Try to find membership with different user ID
        let result = service.find_by_id_and_user_id(membership.id, other_user.id).await;

        // Assert: Should fail with GroupMembershipNotFound error (user doesn't have access to this membership)
        assert!(result.is_err(), "Should fail when accessing membership with wrong user ID");
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error - the query filters by both ID and user_id
            }
            other => panic!("Expected GroupMembershipNotFound error, got: {:?}", other),
        }

        // Cleanup: Delete the test data
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(owner_user.email.clone()).await;
        cleanup_user(member_user.email.clone()).await;
        cleanup_user(other_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_multiple_memberships() {
        // Arrange: Create multiple memberships for the same user in different groups
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        // Create user multiple groups and invitations
        let (owner_user, _password) = create_test_user("find_multi_owner").await;
        let (member_user, _member_password) = create_test_user("find_multi_member").await;
        
        let group_chat1 = create_test_group_chat("find_multi_group1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_multi_group2", owner_user.id).await;

        let invitation1 = create_test_invitation(owner_user.id, member_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(owner_user.id, member_user.id, group_chat2.id).await;
        
        // Create memberships for the same user in different groups
        let membership1 = create_test_group_membership(invitation1.id, member_user.id).await;
        let membership2 = create_test_group_membership(invitation2.id, member_user.id).await;

        // Act: Find both memberships
        let result1 = service.find_by_id_and_user_id(membership1.id, member_user.id).await;
        let result2 = service.find_by_id_and_user_id(membership2.id, member_user.id).await;

        // Assert: Both should be found successfully
        assert!(result1.is_ok(), "Failed to find first membership: {:?}", result1);
        assert!(result2.is_ok(), "Failed to find second membership: {:?}", result2);

        let membership_dto1 = result1.unwrap();
        let membership_dto2 = result2.unwrap();

        // Verify they are different memberships but same user
        assert_eq!(membership_dto1.id, membership1.id);
        assert_eq!(membership_dto2.id, membership2.id);
        assert_ne!(membership_dto1.id, membership_dto2.id);
        
        assert_eq!(membership_dto1.user_id, member_user.id);
        assert_eq!(membership_dto2.user_id, member_user.id);
        
        assert_eq!(membership_dto1.group_chat_id, group_chat1.id);
        assert_eq!(membership_dto2.group_chat_id, group_chat2.id);

        // Cleanup: Delete all test data
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(owner_user.email.clone()).await;
        cleanup_user(member_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_admin_membership() {
        // Arrange: Create admin membership and verify it can be found
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        // Create user, group and invitation
        let (owner_user, _password) = create_test_user("find_admin_owner").await;
        let group_chat = create_test_group_chat("find_admin_test", owner_user.id).await;
        let (admin_user, _admin_password) = create_test_user("find_admin_member").await;
        let invitation = create_test_invitation(owner_user.id, admin_user.id, group_chat.id).await;
        
        // Create admin membership using common helper
        let membership = crate::common::create_test_admin_group_membership(invitation.id, admin_user.id).await;

        // Act: Find the admin membership by ID and user ID
        let result = service.find_by_id_and_user_id(membership.id, admin_user.id).await;

        // Assert: Verify admin membership was found successfully
        assert!(result.is_ok(), "Failed to find admin group membership: {:?}", result);
        let membership_dto = result.unwrap();
        
        assert_eq!(membership_dto.id, membership.id);
        assert_eq!(membership_dto.user_id, admin_user.id);
        assert_eq!(membership_dto.group_chat_id, group_chat.id);
        assert_eq!(membership_dto.role, ruggine_server::entity::group_membership::MemberRole::Admin);
        assert_eq!(membership_dto.membership_status, membership.membership_status);

        // Cleanup: Delete the test data
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(owner_user.email.clone()).await;
        cleanup_user(admin_user.email.clone()).await;
    }
}
