use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto;
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use ruggine_server::entity::group_membership::MembershipStatus;
use crate::common::{get_database, create_test_user, cleanup_user_by_email, cleanup_group_chat, cleanup_invitation, create_test_invitation, create_test_group_membership, cleanup_group_membership};

#[cfg(test)]
mod group_membership_service_leave_group_integration_tests {
    use crate::cleanup_test_user_from_a_group_chat;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_success() {
        // Arrange: Create real users, group, invitation, and membership in database
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        // Create users and group
        let (admin_user, _) = create_test_user("leave_group_admin").await;
        let (member_user, _) = create_test_user("leave_group_member").await;
        
        // Create group chat with admin as creator and member
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_group_test", admin_user.id).await;
        
        // Add member user to the group by creating invitation and membership
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Create leave DTO
        let leave_dto = LeaveGroupMembershipDto {
            id: membership.id,
        };

        // Act: Leave the group
        let result = group_membership_service.leave_group(leave_dto, member_user.id).await;

        // Assert: Verify the user successfully left the group
        assert!(result.is_ok(), "Failed to leave group: {:?}", result);
        let membership_dto = result.unwrap();

        assert_eq!(membership_dto.id, membership.id);
        assert_eq!(membership_dto.user_id, member_user.id);
        assert_eq!(membership_dto.group_chat_id, group_chat.id);
        assert_eq!(membership_dto.membership_status, MembershipStatus::Left);
        assert!(membership_dto.left_at.is_some());

        // Cleanup
        crate::common::cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_membership_not_found() {
        // Arrange: Create user but no membership
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (user, _) = create_test_user("leave_group_no_membership").await;
        
        let nonexistent_membership_id = 99999;
        let leave_dto = LeaveGroupMembershipDto {
            id: nonexistent_membership_id,
        };

        // Act: Try to leave non-existent membership
        let result = group_membership_service.leave_group(leave_dto, user.id).await;

        // Assert: Should fail with GroupMembershipNotFound
        assert!(result.is_err(), "Should fail when membership doesn't exist");
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            e => panic!("Expected GroupMembershipNotFound error, got: {:?}", e),
        }

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_unauthorized_user() {
        // Arrange: Create membership for one user, try to leave with different user
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (admin_user, _) = create_test_user("leave_group_admin_unauth").await;
        let (member_user, _) = create_test_user("leave_group_member_unauth").await;
        let (unauthorized_user, _) = create_test_user("leave_group_unauth_user").await;
        
        // Create group and membership for member_user
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_group_unauthorized", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        let leave_dto = LeaveGroupMembershipDto {
            id: membership.id,
        };

        // Act: Try to leave with unauthorized user
        let result = group_membership_service.leave_group(leave_dto, unauthorized_user.id).await;

        // Assert: Should fail with GroupMembershipNotFound (because membership doesn't belong to this user)
        assert!(result.is_err(), "Should fail when user doesn't own the membership");
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            e => panic!("Expected GroupMembershipNotFound error, got: {:?}", e),
        }

        // Cleanup
        crate::common::cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member_user.email).await;
        cleanup_user_by_email(unauthorized_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_user_already_left() {
        // Arrange: Create membership and leave it first, then try to leave again
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (admin_user, _) = create_test_user("leave_group_admin_already_left").await;
        let (member_user, _) = create_test_user("leave_group_member_already_left").await;
        
        // Create group and membership
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_group_already_left", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        let leave_dto = LeaveGroupMembershipDto {
            id: membership.id,
        };

        // Act: Leave the group first time (should succeed)
        let first_leave_result = group_membership_service.leave_group(leave_dto.clone(), member_user.id).await;
        assert!(first_leave_result.is_ok(), "First leave should succeed");

        // Act: Try to leave again
        let second_leave_result = group_membership_service.leave_group(leave_dto, member_user.id).await;

        // Assert: Should fail with UserAlreadyLeftGroup
        assert!(second_leave_result.is_err(), "Should fail when user already left");
        match second_leave_result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyLeftGroup) => {
                // Expected error
            }
            e => panic!("Expected UserAlreadyLeftGroup error, got: {:?}", e),
        }

        // Cleanup
        crate::common::cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_admin_can_leave() {
        // Arrange: Create admin user and have them leave their own group
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (admin_user, _) = create_test_user("leave_group_admin_can_leave").await;
        
        // Create group chat with admin as creator (this creates membership)
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_group_admin_leaves", admin_user.id).await;
        
        // Get the admin's membership
        let admin_membership = group_membership_service.find_active_by_user_id_and_group_id(admin_user.id, group_chat.id).await.unwrap();

        let leave_dto = LeaveGroupMembershipDto {
            id: admin_membership.id,
        };

        // Act: Admin leaves their own group
        let result = group_membership_service.leave_group(leave_dto, admin_user.id).await;

        // Assert: Admin should be able to leave
        assert!(result.is_ok(), "Admin should be able to leave their own group: {:?}", result);
        let membership_dto = result.unwrap();

        assert_eq!(membership_dto.id, admin_membership.id);
        assert_eq!(membership_dto.user_id, admin_user.id);
        assert_eq!(membership_dto.membership_status, MembershipStatus::Left);
        assert!(membership_dto.left_at.is_some());

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_admin_promotes_member_to_admin() {
        // Arrange: Create a group with admin and member, admin leaves, member should be promoted
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (admin_user, _) = create_test_user("leave_promote_admin").await;
        let (member_user, _) = create_test_user("leave_promote_member").await;
        
        // Create group with admin
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_promote_group", admin_user.id).await;
        
        // Add member user to the group
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let member_membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        // Get admin membership
        let admin_membership = group_membership_service.find_active_by_user_id_and_group_id(admin_user.id, group_chat.id).await.unwrap();

        let leave_dto = LeaveGroupMembershipDto {
            id: admin_membership.id,
        };

        // Act: Admin leaves the group
        let result = group_membership_service.leave_group(leave_dto, admin_user.id).await;

        // Assert: Admin should successfully leave
        assert!(result.is_ok(), "Admin should be able to leave: {:?}", result);
        
        // Verify admin left
        let left_membership = result.unwrap();
        assert_eq!(left_membership.membership_status, MembershipStatus::Left);
        
        // Verify member was promoted to admin
        let updated_member = group_membership_service.find_by_id_and_user_id(member_membership.id, member_user.id).await.unwrap();
        assert_eq!(updated_member.role, ruggine_server::entity::group_membership::MemberRole::Admin, "Member should be promoted to admin");
        assert_eq!(updated_member.membership_status, MembershipStatus::Active);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_membership(member_membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_admin_promotes_first_available_member() {
        // Arrange: Create a group with admin and multiple members, admin leaves, first member should be promoted
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (admin_user, _) = create_test_user("leave_promote_multi_admin").await;
        let (member1_user, _) = create_test_user("leave_promote_multi_member1").await;
        let (member2_user, _) = create_test_user("leave_promote_multi_member2").await;
        let (member3_user, _) = create_test_user("leave_promote_multi_member3").await;
        
        // Create group with admin
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_promote_multi_group", admin_user.id).await;
        
        // Add multiple members to the group
        let invitation1 = create_test_invitation(admin_user.id, member1_user.id, group_chat.id).await;
        let invitation2 = create_test_invitation(admin_user.id, member2_user.id, group_chat.id).await;
        let invitation3 = create_test_invitation(admin_user.id, member3_user.id, group_chat.id).await;
        
        let member1_membership = create_test_group_membership(invitation1.id, member1_user.id).await;
        let member2_membership = create_test_group_membership(invitation2.id, member2_user.id).await;
        let member3_membership = create_test_group_membership(invitation3.id, member3_user.id).await;
        
        // Get admin membership
        let admin_membership = group_membership_service.find_active_by_user_id_and_group_id(admin_user.id, group_chat.id).await.unwrap();

        let leave_dto = LeaveGroupMembershipDto {
            id: admin_membership.id,
        };

        // Act: Admin leaves the group
        let result = group_membership_service.leave_group(leave_dto, admin_user.id).await;

        // Assert: Admin should successfully leave
        assert!(result.is_ok(), "Admin should be able to leave: {:?}", result);
        
        // Verify admin left
        let left_membership = result.unwrap();
        assert_eq!(left_membership.membership_status, MembershipStatus::Left);
        
        // Verify exactly one member was promoted to admin
        let member1 = group_membership_service.find_by_id_and_user_id(member1_membership.id, member1_user.id).await.unwrap();
        let member2 = group_membership_service.find_by_id_and_user_id(member2_membership.id, member2_user.id).await.unwrap();
        let member3 = group_membership_service.find_by_id_and_user_id(member3_membership.id, member3_user.id).await.unwrap();
        
        let admin_count = [&member1, &member2, &member3]
            .iter()
            .filter(|m| m.role == ruggine_server::entity::group_membership::MemberRole::Admin)
            .count();
        
        assert_eq!(admin_count, 1, "Exactly one member should be promoted to admin");
        
        let member_count = [&member1, &member2, &member3]
            .iter()
            .filter(|m| m.role == ruggine_server::entity::group_membership::MemberRole::Member)
            .count();
        
        assert_eq!(member_count, 2, "Two members should remain as regular members");

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_membership(member1_membership.id).await;
        cleanup_group_membership(member2_membership.id).await;
        cleanup_group_membership(member3_membership.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_invitation(invitation3.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member1_user.email).await;
        cleanup_user_by_email(member2_user.email).await;
        cleanup_user_by_email(member3_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_member_leaves_no_promotion_needed() {
        // Arrange: Create a group with admin and member, member leaves, no promotion should occur
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (admin_user, _) = create_test_user("leave_no_promote_admin").await;
        let (member_user, _) = create_test_user("leave_no_promote_member").await;
        
        // Create group with admin
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_no_promote_group", admin_user.id).await;
        
        // Add member user to the group
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let member_membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        // Get admin membership before member leaves
        let admin_membership_before = group_membership_service.find_active_by_user_id_and_group_id(admin_user.id, group_chat.id).await.unwrap();

        let leave_dto = LeaveGroupMembershipDto {
            id: member_membership.id,
        };

        // Act: Member leaves the group
        let result = group_membership_service.leave_group(leave_dto, member_user.id).await;

        // Assert: Member should successfully leave
        assert!(result.is_ok(), "Member should be able to leave: {:?}", result);
        
        // Verify member left
        let left_membership = result.unwrap();
        assert_eq!(left_membership.membership_status, MembershipStatus::Left);
        assert_eq!(left_membership.user_id, member_user.id);
        
        // Verify admin remains admin (no change)
        let admin_membership_after = group_membership_service.find_active_by_user_id_and_group_id(admin_user.id, group_chat.id).await.unwrap();
        assert_eq!(admin_membership_after.role, ruggine_server::entity::group_membership::MemberRole::Admin);
        assert_eq!(admin_membership_after.id, admin_membership_before.id);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_membership(member_membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_admin_leaves_no_members_left() {
        // Arrange: Create a group with only admin, admin leaves, no promotion should occur
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (admin_user, _) = create_test_user("leave_no_members_admin").await;
        
        // Create group with only admin
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_no_members_group", admin_user.id).await;
        
        // Get admin membership
        let admin_membership = group_membership_service.find_active_by_user_id_and_group_id(admin_user.id, group_chat.id).await.unwrap();

        let leave_dto = LeaveGroupMembershipDto {
            id: admin_membership.id,
        };

        // Act: Admin leaves the group (only member)
        let result = group_membership_service.leave_group(leave_dto, admin_user.id).await;

        // Assert: Admin should successfully leave even though no one is left
        assert!(result.is_ok(), "Admin should be able to leave even with no other members: {:?}", result);
        
        // Verify admin left
        let left_membership = result.unwrap();
        assert_eq!(left_membership.membership_status, MembershipStatus::Left);
        assert_eq!(left_membership.user_id, admin_user.id);
        
        // Verify no active members remain in the group
        let remaining_members = group_membership_service.find_by_group_chat_id(group_chat.id).await.unwrap();
        let active_members: Vec<_> = remaining_members
            .iter()
            .filter(|m| m.membership_status == MembershipStatus::Active)
            .collect();
        
        assert_eq!(active_members.len(), 0, "No active members should remain in the group");

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_admin_leaves_only_left_members_remain() {
        // Arrange: Create a group with admin and member who already left, admin leaves, no promotion should occur
        let db = get_database().await;
        let group_membership_service = GroupMembershipService::new(&db);
        
        let (admin_user, _) = create_test_user("leave_only_left_admin").await;
        let (member_user, _) = create_test_user("leave_only_left_member").await;
        
        // Create group with admin
        let group_chat = crate::common::create_test_group_chat_with_invitation_and_membership("leave_only_left_group", admin_user.id).await;
        
        // Add member user to the group and make them leave first
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let member_membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        // Member leaves first
        let member_leave_dto = LeaveGroupMembershipDto {
            id: member_membership.id,
        };
        let member_leave_result = group_membership_service.leave_group(member_leave_dto, member_user.id).await;
        assert!(member_leave_result.is_ok(), "Member should be able to leave first");
        
        // Get admin membership
        let admin_membership = group_membership_service.find_active_by_user_id_and_group_id(admin_user.id, group_chat.id).await.unwrap();

        let admin_leave_dto = LeaveGroupMembershipDto {
            id: admin_membership.id,
        };

        // Act: Admin leaves the group (only active member, other member already left)
        let result = group_membership_service.leave_group(admin_leave_dto, admin_user.id).await;

        // Assert: Admin should successfully leave
        assert!(result.is_ok(), "Admin should be able to leave: {:?}", result);
        
        // Verify admin left
        let left_membership = result.unwrap();
        assert_eq!(left_membership.membership_status, MembershipStatus::Left);
        assert_eq!(left_membership.user_id, admin_user.id);
        
        // Verify no active members remain, but left member is still Left (not promoted)
        let all_members = group_membership_service.find_by_group_chat_id(group_chat.id).await.unwrap();
        
        assert_eq!(all_members.len(), 0, "No active members should remain");
        
        // Verify the previously left member was not promoted
        let final_member_state = group_membership_service.find_by_id_and_user_id(member_membership.id, member_user.id).await.unwrap();
        assert_eq!(final_member_state.role, ruggine_server::entity::group_membership::MemberRole::Member, "Previously left member should remain a member (not promoted)");
        assert_eq!(final_member_state.membership_status, MembershipStatus::Left);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_membership(member_membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }
}
