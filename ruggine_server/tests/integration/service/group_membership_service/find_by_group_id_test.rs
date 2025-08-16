use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_chat_error::GroupChatError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use crate::common::{
    get_database, create_test_user, create_test_group_chat, create_test_group_membership,
    cleanup_user_by_email, cleanup_group_chat, cleanup_group_membership, create_test_invitation, cleanup_invitation
};

#[cfg(test)]
mod group_membership_find_by_group_id_integration_tests {
    use crate::{cleanup_test_user_from_a_group_chat, create_test_group_chat_with_invitation_and_membership};

    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_success_single_membership() {
        // Arrange: Create real user, group chat, and membership in the database
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        // Create test entities
        let (owner_user, _) = create_test_user("find_by_group_id_owner_single").await;
        let (member_user, _) = create_test_user("find_by_group_id_member_single").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_by_group_id_single", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create membership using common helper
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Act: Find all memberships for the group (authenticated as member)
        let result = service.find_by_group_id(group_chat.id, member_user.id).await;

        // Assert: Verify single membership was found
        assert!(result.is_ok(), "Failed to find group memberships: {:?}", result);
        let memberships = result.unwrap();
        
        assert_eq!(memberships.len(), 2);
        // Verify all user IDs are present
        let found_user_ids: std::collections::HashSet<i32> = memberships.iter().map(|m| m.user_id).collect();
        let expected_user_ids: std::collections::HashSet<i32> = [member_user.id, owner_user.id].iter().cloned().collect();
        assert_eq!(found_user_ids, expected_user_ids);

        // Cleanup: Delete the test data
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_success_multiple_memberships() {
        // Arrange: Create multiple users and memberships in a group
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_group_id_owner_multi").await;
        let (member1_user, _) = create_test_user("find_by_group_id_member1_multi").await;
        let (member2_user, _) = create_test_user("find_by_group_id_member2_multi").await;
        let (member3_user, _) = create_test_user("find_by_group_id_member3_multi").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_by_group_id_multi", owner_user.id).await;

        // Create invitations and memberships
        let invitation1 = create_test_invitation(owner_user.id, member1_user.id, group_chat.id).await;
        let invitation2 = create_test_invitation(owner_user.id, member2_user.id, group_chat.id).await;
        let invitation3 = create_test_invitation(owner_user.id, member3_user.id, group_chat.id).await;
        
        let membership1 = create_test_group_membership(invitation1.id, member1_user.id).await;
        let membership2 = create_test_group_membership(invitation2.id, member2_user.id).await;
        let membership3 = create_test_group_membership(invitation3.id, member3_user.id).await;

        // Act: Find all memberships for the group (authenticated as first member)
        let result = service.find_by_group_id(group_chat.id, member1_user.id).await;

        // Assert: Verify all memberships were found
        assert!(result.is_ok(), "Failed to find group memberships: {:?}", result);
        let memberships = result.unwrap();
        
        assert_eq!(memberships.len(), 4);
        
        // Verify all user IDs are present
        let found_user_ids: std::collections::HashSet<i32> = memberships.iter().map(|m| m.user_id).collect();
        let expected_user_ids: std::collections::HashSet<i32> = [member1_user.id, member2_user.id, member3_user.id, owner_user.id].iter().cloned().collect();
        assert_eq!(found_user_ids, expected_user_ids);

        // Verify all have same group ID and active status
        for membership_dto in &memberships {
            assert_eq!(membership_dto.group_chat_id, group_chat.id);
            assert_eq!(membership_dto.membership_status, MembershipStatus::Active);
            assert!(membership_dto.left_at.is_none());
        }

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_group_membership(membership3.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_invitation(invitation3.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member1_user.email.clone()).await;
        cleanup_user_by_email(member2_user.email.clone()).await;
        cleanup_user_by_email(member3_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_group_not_found() {
        // Arrange: Create user but use non-existent group
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (user, _) = create_test_user("find_by_group_id_no_group").await;
        let non_existent_group_id = 999999;

        // Act: Try to find memberships for non-existent group
        let result = service.find_by_group_id(non_existent_group_id, user.id).await;

        // Assert: Should return group not found error
        assert!(result.is_err());
        if let Err(ApiError::GroupMembershipError(GroupMembershipError::GroupNotFound)) = result {
            // Expected error
        } else {
            panic!("Expected GroupMembershipError::GroupNotFound, got: {:?}", result);
        }

        // Cleanup
        cleanup_user_by_email(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_user_not_in_group() {
        // Arrange: Create group and user, but no membership
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_group_id_owner_no_member").await;
        let (unauthorized_user, _) = create_test_user("find_by_group_id_unauthorized").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_by_group_id_no_member", owner_user.id).await;

        // Act: Try to access group memberships as non-member
        let result = service.find_by_group_id(group_chat.id, unauthorized_user.id).await;

        // Assert: Should return user not in group error
        assert!(result.is_err());
        if let Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) = result {
            // Expected error
        } else {
            panic!("Expected GroupMembershipError::GroupMembershipNotFound, got: {:?}", result);
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(unauthorized_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_empty_group() {
        // Arrange: Create group with owner but no other members
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_group_id_owner_empty").await;
        let (member_user, _) = create_test_user("find_by_group_id_member_empty").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_by_group_id_empty", owner_user.id).await;
        
        // Create a membership so the auth user can access the group, but it's the only one
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Act: Find memberships in group with only one member
        let result = service.find_by_group_id(group_chat.id, member_user.id).await;

        // Assert: Should return one membership (the auth user)
        assert!(result.is_ok(), "Failed to find group memberships: {:?}", result);
        let memberships = result.unwrap();
        
        assert_eq!(memberships.len(), 2);
        // Verify all user IDs are present
        let found_user_ids: std::collections::HashSet<i32> = memberships.iter().map(|m| m.user_id).collect();
        let expected_user_ids: std::collections::HashSet<i32> = [member_user.id, owner_user.id].iter().cloned().collect();
        assert_eq!(found_user_ids, expected_user_ids);

        // Cleanup
        cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_only_active_memberships() {
        // Arrange: Create group with both active and left memberships
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_group_id_owner_active").await;
        let (active_user, _) = create_test_user("find_by_group_id_active").await;
        let (left_user, _) = create_test_user("find_by_group_id_left").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_by_group_id_active", owner_user.id).await;
        
        // Create invitations and memberships
        let invitation1 = create_test_invitation(owner_user.id, active_user.id, group_chat.id).await;
        let invitation2 = create_test_invitation(owner_user.id, left_user.id, group_chat.id).await;
        
        let active_membership = create_test_group_membership(invitation1.id, active_user.id).await;
        let left_membership = create_test_group_membership(invitation2.id, left_user.id).await;

        // Make the second user leave the group
        let leave_dto = ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto {
            id: left_membership.id,
        };
        let _leave_result = service.leave_group(leave_dto, left_user.id).await;

        // Act: Find memberships as active user
        let result = service.find_by_group_id(group_chat.id, active_user.id).await;

        // Assert: Should only return active memberships
        assert!(result.is_ok(), "Failed to find group memberships: {:?}", result);
        let memberships = result.unwrap();
        
        // Should only find the active membership and the owner (left users are not returned by repository)
        assert_eq!(memberships.len(), 2);
        
        // Verify all user IDs are present
        let found_user_ids: std::collections::HashSet<i32> = memberships.iter().map(|m| m.user_id).collect();
        let expected_user_ids: std::collections::HashSet<i32> = [active_user.id, owner_user.id].iter().cloned().collect();
        assert_eq!(found_user_ids, expected_user_ids);

        // Cleanup
        cleanup_test_user_from_a_group_chat(active_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(left_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(active_user.email.clone()).await;
        cleanup_user_by_email(left_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_user_left_group_cannot_access() {
        // Arrange: Create group with user who then leaves
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_group_id_owner_left_access").await;
        let (member_user, _) = create_test_user("find_by_group_id_member_left_access").await;
        let (other_user, _) = create_test_user("find_by_group_id_other_left_access").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_by_group_id_left_access", owner_user.id).await;
        
        // Create invitations and memberships
        let invitation1 = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        let invitation2 = create_test_invitation(owner_user.id, other_user.id, group_chat.id).await;
        
        let member_membership = create_test_group_membership(invitation1.id, member_user.id).await;
        let other_membership = create_test_group_membership(invitation2.id, other_user.id).await;

        // Make the member user leave the group
        let leave_dto = ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto {
            id: member_membership.id,
        };
        let _leave_result = service.leave_group(leave_dto, member_user.id).await;

        // Act: Try to access group memberships as user who left
        let result = service.find_by_group_id(group_chat.id, member_user.id).await;

        // Assert: Should return error because user has left the group
        assert!(result.is_err());
        if let Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) = result {
            // Expected error
        } else {
            panic!("Expected GroupMembershipError::GroupMembershipNotFound, got: {:?}", result);
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(other_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(other_user.email.clone()).await;
    }
}
