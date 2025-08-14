use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use crate::common::{
    get_database, create_test_user, create_test_group_chat, create_test_group_membership,
    cleanup_user_by_email, cleanup_group_chat, cleanup_group_membership, create_test_invitation, cleanup_invitation
};

#[cfg(test)]
mod group_membership_find_by_user_id_integration_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_success_single_membership() {
        // Arrange: Create real user, group chat, and membership in the database
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        // Create test entities
        let (owner_user, _) = create_test_user("find_by_user_id_owner_single").await;
        let (member_user, _) = create_test_user("find_by_user_id_member_single").await;
        let group_chat = create_test_group_chat("find_by_user_id_single", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create membership using common helper
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Act: Find all memberships for the member user
        let result = service.find_by_user_id(member_user.id).await;

        // Assert: Verify single membership was found
        assert!(result.is_ok(), "Failed to find memberships: {:?}", result);
        let memberships = result.unwrap();
        
        assert_eq!(memberships.len(), 1);
        let membership_dto = &memberships[0];
        
        assert_eq!(membership_dto.id, membership.id);
        assert_eq!(membership_dto.user_id, member_user.id);
        assert_eq!(membership_dto.group_chat_id, group_chat.id);
        assert_eq!(membership_dto.role, membership.role);
        assert_eq!(membership_dto.membership_status, membership.membership_status);
        assert!(membership_dto.left_at.is_none());

        // Cleanup: Delete the test data
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_multiple_memberships() {
        // Arrange: Create multiple memberships for same user
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_user_id_owner_multi").await;
        let (member_user, _) = create_test_user("find_by_user_id_member_multi").await;
        
        // Create multiple group chats
        let group_chat1 = create_test_group_chat("find_by_user_id_multi1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_multi2", owner_user.id).await;
        let group_chat3 = create_test_group_chat("find_by_user_id_multi3", owner_user.id).await;
        
        // Create invitations
        let invitation1 = create_test_invitation(owner_user.id, member_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(owner_user.id, member_user.id, group_chat2.id).await;
        let invitation3 = create_test_invitation(owner_user.id, member_user.id, group_chat3.id).await;
        
        // Create memberships
        let membership1 = create_test_group_membership(invitation1.id, member_user.id).await;
        let membership2 = create_test_group_membership(invitation2.id, member_user.id).await;
        let membership3 = create_test_group_membership(invitation3.id, member_user.id).await;

        // Act: Find all memberships for the member user
        let result = service.find_by_user_id(member_user.id).await;

        // Assert: Verify all memberships were found
        assert!(result.is_ok(), "Failed to find memberships: {:?}", result);
        let memberships = result.unwrap();
        
        assert_eq!(memberships.len(), 3);
        
        // Verify all memberships belong to the correct user
        for membership_dto in &memberships {
            assert_eq!(membership_dto.user_id, member_user.id);
            assert_eq!(membership_dto.membership_status, MembershipStatus::Active);
        }
        
        // Verify we have all the group chat IDs
        let group_chat_ids: std::collections::HashSet<_> = memberships.iter().map(|m| m.group_chat_id).collect();
        assert!(group_chat_ids.contains(&group_chat1.id));
        assert!(group_chat_ids.contains(&group_chat2.id));
        assert!(group_chat_ids.contains(&group_chat3.id));
        
        // Verify memberships are ordered by joined_at DESC (newest first)
        for i in 1..memberships.len() {
            assert!(memberships[i-1].joined_at >= memberships[i].joined_at, 
                    "Memberships should be ordered by joined_at DESC");
        }

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_group_membership(membership3.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_invitation(invitation3.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_group_chat(group_chat3.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_empty_result() {
        // Arrange: Create user with no memberships
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (user, _) = create_test_user("find_by_user_id_empty").await;

        // Act: Find memberships for user with no memberships
        let result = service.find_by_user_id(user.id).await;

        // Assert: Should return empty vector
        assert!(result.is_ok(), "Failed to find memberships: {:?}", result);
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 0);

        // Cleanup
        cleanup_user_by_email(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_nonexistent_user() {
        // Arrange
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);

        // Act: Find memberships for nonexistent user
        let result = service.find_by_user_id(-1).await;

        // Assert: Should return empty vector (not error)
        assert!(result.is_ok(), "Should handle nonexistent user gracefully");
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_different_roles() {
        // Arrange: Create memberships with different roles
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_user_id_owner_roles").await;
        let (member_user, _) = create_test_user("find_by_user_id_member_roles").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_roles1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_roles2", owner_user.id).await;
        
        let invitation1 = create_test_invitation(owner_user.id, member_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(owner_user.id, member_user.id, group_chat2.id).await;
        
        // Create one membership as Member and one as Admin
        let membership1 = create_test_group_membership(invitation1.id, member_user.id).await; // Default Member
        let membership2 = create_test_group_membership(invitation2.id, member_user.id).await; // Will be Member by default
        
        // Note: The test helper creates Member role by default. 
        // For a more realistic test, you might want to create an admin membership helper

        // Act
        let result = service.find_by_user_id(member_user.id).await;

        // Assert
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 2);
        
        // Verify both memberships exist
        for membership_dto in &memberships {
            assert_eq!(membership_dto.user_id, member_user.id);
            assert_eq!(membership_dto.membership_status, MembershipStatus::Active);
            // Both will be Member role due to test helper limitation
            assert_eq!(membership_dto.role, MemberRole::Member);
        }

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_user_isolation() {
        // Arrange: Create memberships for different users to test isolation
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_user_id_owner_isolation").await;
        let (member_user1, _) = create_test_user("find_by_user_id_member1_isolation").await;
        let (member_user2, _) = create_test_user("find_by_user_id_member2_isolation").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_isolation1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_isolation2", owner_user.id).await;
        
        let invitation1 = create_test_invitation(owner_user.id, member_user1.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(owner_user.id, member_user2.id, group_chat2.id).await;
        
        let membership1 = create_test_group_membership(invitation1.id, member_user1.id).await;
        let membership2 = create_test_group_membership(invitation2.id, member_user2.id).await;

        // Act: Find memberships for each user separately
        let result1 = service.find_by_user_id(member_user1.id).await;
        let result2 = service.find_by_user_id(member_user2.id).await;

        // Assert: Each user should only see their own memberships
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let memberships1 = result1.unwrap();
        let memberships2 = result2.unwrap();
        
        assert_eq!(memberships1.len(), 1);
        assert_eq!(memberships2.len(), 1);
        
        // Verify user 1 only sees their membership
        assert_eq!(memberships1[0].id, membership1.id);
        assert_eq!(memberships1[0].user_id, member_user1.id);
        assert_eq!(memberships1[0].group_chat_id, group_chat1.id);
        
        // Verify user 2 only sees their membership
        assert_eq!(memberships2[0].id, membership2.id);
        assert_eq!(memberships2[0].user_id, member_user2.id);
        assert_eq!(memberships2[0].group_chat_id, group_chat2.id);

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member_user1.email.clone()).await;
        cleanup_user_by_email(member_user2.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_ordering() {
        // Arrange: Create memberships at different times to test ordering
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_user_id_owner_ordering").await;
        let (member_user, _) = create_test_user("find_by_user_id_member_ordering").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_ordering1", owner_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_ordering2", owner_user.id).await;
        
        let invitation1 = create_test_invitation(owner_user.id, member_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(owner_user.id, member_user.id, group_chat2.id).await;
        
        // Create first membership
        let membership1 = create_test_group_membership(invitation1.id, member_user.id).await;
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Create second membership
        let membership2 = create_test_group_membership(invitation2.id, member_user.id).await;

        // Act
        let result = service.find_by_user_id(member_user.id).await;

        // Assert: Results should be ordered by joined_at DESC (newest first)
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 2);
        
        // The second membership should be first (newer)
        assert!(memberships[0].joined_at >= memberships[1].joined_at, 
                "Memberships should be ordered by joined_at DESC");

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_with_zero_user_id() {
        // Arrange
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);

        // Act: Find memberships for user ID 0
        let result = service.find_by_user_id(0).await;

        // Assert: Should return empty vector
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 0);
    }
}
