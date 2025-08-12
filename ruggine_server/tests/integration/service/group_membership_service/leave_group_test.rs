use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto;
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use ruggine_server::entity::group_membership::MembershipStatus;
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, cleanup_invitation, create_test_invitation, create_test_group_membership, cleanup_group_membership};

#[cfg(test)]
mod group_membership_service_leave_group_integration_tests {
    use crate::clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id;
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
        crate::common::clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(member_user.email).await;
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
        cleanup_user(user.email).await;
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
        crate::common::clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(member_user.email).await;
        cleanup_user(unauthorized_user.email).await;
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
        crate::common::clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(member_user.email).await;
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
        let admin_membership = group_membership_service.find_by_user_id_and_group_id(admin_user.id, group_chat.id).await.unwrap();

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
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
    }
}
