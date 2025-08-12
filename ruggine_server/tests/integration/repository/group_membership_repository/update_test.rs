use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus, UpdateGroupMembership};
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat, cleanup_invitation, create_test_invitation, create_test_group_membership, cleanup_group_membership};
use chrono::Utc;

#[cfg(test)]
mod group_membership_repository_update_integration_tests {
    use crate::create_test_group_chat;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_leave_group_success() {
        // Arrange: Create real membership in database
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        // Create users and group
        let (admin_user, _) = create_test_user("update_repo_admin").await;
        let (member_user, _) = create_test_user("update_repo_member").await;
        
        // Create group and membership
        let group_chat = create_test_group_chat("update_repo_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Create update to leave group
        let update_membership = UpdateGroupMembership {
            id: membership.id,
            role: None,
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(Utc::now()),
        };

        // Act: Update the membership
        let result = repository.update(update_membership).await;

        // Assert: Verify update was successful
        assert!(result.is_ok(), "Failed to update membership: {:?}", result);

        // Verify the membership was actually updated
        let updated_membership = repository.find_by_id_and_user_id(membership.id, member_user.id).await;
        assert!(updated_membership.is_ok(), "Failed to retrieve updated membership");
        
        let updated = updated_membership.unwrap();
        assert_eq!(updated.membership_status, MembershipStatus::Left);
        assert!(updated.left_at.is_some());

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_role_change_success() {
        // Arrange: Create real membership in database
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        // Create users and group
        let (admin_user, _) = create_test_user("update_role_admin").await;
        let (member_user, _) = create_test_user("update_role_member").await;
        
        // Create group and membership
        let group_chat = create_test_group_chat("update_role_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Create update to change role to admin
        let update_membership = UpdateGroupMembership {
            id: membership.id,
            role: Some(MemberRole::Admin),
            membership_status: None,
            left_at: None,
        };

        // Act: Update the membership
        let result = repository.update(update_membership).await;

        // Assert: Verify update was successful
        assert!(result.is_ok(), "Failed to update membership role: {:?}", result);

        // Verify the role was actually updated
        let updated_membership = repository.find_by_id_and_user_id(membership.id, member_user.id).await;
        assert!(updated_membership.is_ok(), "Failed to retrieve updated membership");
        
        let updated = updated_membership.unwrap();
        assert_eq!(updated.role, MemberRole::Admin);
        assert_eq!(updated.membership_status, MembershipStatus::Active); // Should remain active

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_membership_not_found() {
        // Arrange: Try to update non-existent membership
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        let nonexistent_membership_id = 999999;
        let update_membership = UpdateGroupMembership {
            id: nonexistent_membership_id,
            role: Some(MemberRole::Admin),
            membership_status: None,
            left_at: None,
        };

        // Act: Try to update non-existent membership
        let result = repository.update(update_membership).await;

        // Assert: Should fail with RowNotFound
        assert!(result.is_err(), "Should fail when membership doesn't exist");
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {
                // Expected error
            }
            e => panic!("Expected RowNotFound error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_no_changes() {
        // Arrange: Create real membership in database
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        let (admin_user, _) = create_test_user("update_no_change_admin").await;
        let (member_user, _) = create_test_user("update_no_change_member").await;
        
        // Create group and membership
        let group_chat = create_test_group_chat("update_no_change_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Create update with no actual changes
        let update_membership = UpdateGroupMembership {
            id: membership.id,
            role: None,
            membership_status: None,
            left_at: None,
        };

        // Act: Update with no changes
        let result = repository.update(update_membership).await;

        // Assert: Should succeed (no-op)
        assert!(result.is_ok(), "Should succeed even with no changes: {:?}", result);

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_multiple_fields() {
        // Arrange: Create real membership in database
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        let (admin_user, _) = create_test_user("update_multi_admin").await;
        let (member_user, _) = create_test_user("update_multi_member").await;
        
        // Create group and membership
        let group_chat = create_test_group_chat("update_multi_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Create update that changes multiple fields
        let left_time = Utc::now();
        let update_membership = UpdateGroupMembership {
            id: membership.id,
            role: Some(MemberRole::Admin),
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(left_time),
        };

        // Act: Update multiple fields
        let result = repository.update(update_membership).await;

        // Assert: Verify update was successful
        assert!(result.is_ok(), "Failed to update multiple fields: {:?}", result);

        // Verify all fields were updated
        let updated_membership = repository.find_by_id_and_user_id(membership.id, member_user.id).await;
        assert!(updated_membership.is_ok(), "Failed to retrieve updated membership");
        
        let updated = updated_membership.unwrap();
        assert_eq!(updated.role, MemberRole::Admin);
        assert_eq!(updated.membership_status, MembershipStatus::Left);
        assert!(updated.left_at.is_some());

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(member_user.email).await;
    }
}
