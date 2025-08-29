use std::sync::Arc;
use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::service::group_chat_service::GroupChatService;
use ruggine_server::service::invitation_service::InvitationService;
use ruggine_server::service::user_service::UserService;
use ruggine_server::repository::invitation_repository::InvitationRepository;
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use crate::common::{
    get_database, create_test_user, cleanup_user_by_email, cleanup_group_chat, 
    cleanup_invitation, create_test_invitation, cleanup_group_membership, create_test_group_chat
};

#[cfg(test)]
mod group_membership_service_promote_admin_if_none_integration_tests {
    use chrono::Utc;
    use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
    use super::*;

    async fn create_service() -> Arc<GroupMembershipService> {
        let db = get_database().await;
        let service = Arc::new(GroupMembershipService::new(&db));
        let invitation_service = InvitationService::with(
            Arc::new(InvitationRepository::new(&db)),
            Arc::new(GroupChatService::new(&db)),
            Arc::new(UserService::new(&db)),
            service.clone(),
        );
        service.set_invitation_service(Arc::new(invitation_service));
        service
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_promote_admin_if_none_success_single_member() {
        // Arrange: Create a group with only one active member (no admin)
        let service = create_service().await;
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        // Create users and group
        let (owner_user, _) = create_test_user("service_promote_owner").await;
        let (member_user, _) = create_test_user("service_promote_member").await;
        
        // Create group and membership for regular member
        let group_chat = create_test_group_chat("service_promote_group", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create regular member membership (not admin)
        let membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(membership).await.unwrap();

        // Act: Promote admin if none exists
        let result = service.promote_admin_if_none_internal(group_chat.id).await;

        // Assert: Should successfully promote the member to admin
        assert!(result.is_ok(), "Failed to promote admin: {:?}", result);
        
        let promoted_membership = result.unwrap();
        assert!(promoted_membership.is_some(), "Should have promoted a member to admin");
        
        let promoted = promoted_membership.unwrap();
        assert_eq!(promoted.id, membership_id);
        assert_eq!(promoted.role, MemberRole::Admin);
        assert_eq!(promoted.membership_status, MembershipStatus::Active);
        assert_eq!(promoted.user_id, member_user.id);
        assert_eq!(promoted.group_chat_id, group_chat.id);

        // Verify the promotion was persisted in database
        let updated_membership = repository.find_by_id_and_user_id(membership_id, member_user.id).await;
        assert!(updated_membership.is_ok(), "Failed to retrieve updated membership");
        
        let updated = updated_membership.unwrap();
        assert_eq!(updated.role, MemberRole::Admin);

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_promote_admin_if_none_no_promotion_when_admin_exists() {
        // Arrange: Create a group with an existing admin
        let service = create_service().await;
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        // Create users and group
        let (owner_user, _) = create_test_user("service_no_promote_owner").await;
        let (admin_user, _) = create_test_user("service_no_promote_admin").await;
        let (member_user, _) = create_test_user("service_no_promote_member").await;
        
        // Create group and memberships
        let group_chat = create_test_group_chat("service_no_promote_group", owner_user.id).await;
        let admin_invitation = create_test_invitation(owner_user.id, admin_user.id, group_chat.id).await;
        let member_invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create admin membership
        let admin_membership = GroupMembershipFactory::fake_new_admin_group_membership_with_id(admin_invitation.id);
        let admin_membership_id = repository.insert(admin_membership).await.unwrap();
        
        // Create regular member membership
        let member_membership = GroupMembershipFactory::fake_new_group_membership_with_id(member_invitation.id);
        let member_membership_id = repository.insert(member_membership).await.unwrap();

        // Act: Try to promote admin when one already exists
        let result = service.promote_admin_if_none_internal(group_chat.id).await;

        // Assert: Should not promote anyone since admin already exists
        assert!(result.is_ok(), "Function should succeed even when no promotion needed");
        
        let promoted_membership = result.unwrap();
        assert!(promoted_membership.is_none(), "Should not promote anyone when admin already exists");

        // Verify existing roles remain unchanged
        let admin_check = repository.find_by_id_and_user_id(admin_membership_id, admin_user.id).await.unwrap();
        let member_check = repository.find_by_id_and_user_id(member_membership_id, member_user.id).await.unwrap();
        
        assert_eq!(admin_check.role, MemberRole::Admin);
        assert_eq!(member_check.role, MemberRole::Member);

        // Cleanup
        cleanup_group_membership(admin_membership_id).await;
        cleanup_group_membership(member_membership_id).await;
        cleanup_invitation(admin_invitation.id).await;
        cleanup_invitation(member_invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_promote_admin_if_none_multiple_members_promotes_first() {
        // Arrange: Create a group with multiple active members (no admin)
        let service = create_service().await;
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        // Create users and group
        let (owner_user, _) = create_test_user("service_multi_promote_owner").await;
        let (member1_user, _) = create_test_user("service_multi_promote_member1").await;
        let (member2_user, _) = create_test_user("service_multi_promote_member2").await;
        let (member3_user, _) = create_test_user("service_multi_promote_member3").await;
        
        // Create group and invitations
        let group_chat = create_test_group_chat("service_multi_promote_group", owner_user.id).await;
        let invitation1 = create_test_invitation(owner_user.id, member1_user.id, group_chat.id).await;
        let invitation2 = create_test_invitation(owner_user.id, member2_user.id, group_chat.id).await;
        let invitation3 = create_test_invitation(owner_user.id, member3_user.id, group_chat.id).await;
        
        // Create memberships (all as regular members)
        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let membership2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation2.id);
        let membership3 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation3.id);
        
        let membership_id1 = repository.insert(membership1).await.unwrap();
        let membership_id2 = repository.insert(membership2).await.unwrap();
        let membership_id3 = repository.insert(membership3).await.unwrap();

        // Act: Promote admin if none exists
        let result = service.promote_admin_if_none_internal(group_chat.id).await;

        // Assert: Should promote exactly one member to admin
        assert!(result.is_ok(), "Failed to promote admin: {:?}", result);
        
        let promoted_membership = result.unwrap();
        assert!(promoted_membership.is_some(), "Should have promoted one member to admin");
        
        let promoted = promoted_membership.unwrap();
        assert_eq!(promoted.role, MemberRole::Admin);
        assert_eq!(promoted.membership_status, MembershipStatus::Active);
        assert_eq!(promoted.group_chat_id, group_chat.id);

        // Verify exactly one member is promoted and others remain members
        let all_membership_ids = vec![membership_id1, membership_id2, membership_id3];
        let all_user_ids = vec![member1_user.id, member2_user.id, member3_user.id];
        
        let mut admin_count = 0;
        let mut member_count = 0;
        
        for (i, &membership_id) in all_membership_ids.iter().enumerate() {
            let membership = repository.find_by_id_and_user_id(membership_id, all_user_ids[i]).await.unwrap();
            match membership.role {
                MemberRole::Admin => admin_count += 1,
                MemberRole::Member => member_count += 1,
            }
        }
        
        assert_eq!(admin_count, 1, "Exactly one member should be promoted to admin");
        assert_eq!(member_count, 2, "Two members should remain as regular members");

        // Cleanup
        cleanup_group_membership(membership_id1).await;
        cleanup_group_membership(membership_id2).await;
        cleanup_group_membership(membership_id3).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_invitation(invitation3.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email).await;
        cleanup_user_by_email(member1_user.email).await;
        cleanup_user_by_email(member2_user.email).await;
        cleanup_user_by_email(member3_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_promote_admin_if_none_no_active_members() {
        // Arrange: Create a group with no active members
        let service = create_service().await;
        
        // Create users and group
        let (owner_user, _) = create_test_user("service_no_active_owner").await;
        let group_chat = create_test_group_chat("service_no_active_group", owner_user.id).await;

        // Act: Try to promote admin when no active members exist
        let result = service.promote_admin_if_none_internal(group_chat.id).await;

        // Assert: Should succeed but not promote anyone
        assert!(result.is_ok(), "Function should succeed even when no active members");
        
        let promoted_membership = result.unwrap();
        assert!(promoted_membership.is_none(), "Should not promote anyone when no active members exist");

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_promote_admin_if_none_only_inactive_members() {
        // Arrange: Create a group with only inactive members (left the group)
        let service = create_service().await;
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        // Create users and group
        let (owner_user, _) = create_test_user("service_inactive_owner").await;
        let (member_user, _) = create_test_user("service_inactive_member").await;
        
        // Create group and membership
        let group_chat = create_test_group_chat("service_inactive_group", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create membership and then mark as left
        let membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(membership).await.unwrap();
        
        // Update membership to left status
        let update_membership = ruggine_server::entity::group_membership::UpdateGroupMembership {
            id: membership_id,
            role: None,
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(chrono::Utc::now()),
            current_action: None,
        };
        repository.update(update_membership).await.unwrap();

        // Act: Try to promote admin when only inactive members exist
        let result = service.promote_admin_if_none_internal(group_chat.id).await;

        // Assert: Should succeed but not promote anyone
        assert!(result.is_ok(), "Function should succeed even when only inactive members");
        
        let promoted_membership = result.unwrap();
        assert!(promoted_membership.is_none(), "Should not promote anyone when only inactive members exist");

        // Verify the left member remains a member with left status
        let member_check = repository.find_by_id_and_user_id(membership_id, member_user.id).await.unwrap();
        assert_eq!(member_check.role, MemberRole::Member);
        assert_eq!(member_check.membership_status, MembershipStatus::Left);

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_promote_admin_if_none_group_not_found() {
        // Arrange: Use a non-existent group chat ID
        let service = create_service().await;
        
        let nonexistent_group_id = 999999;

        // Act: Try to promote admin for non-existent group
        let result = service.promote_admin_if_none_internal(nonexistent_group_id).await;

        // Assert: Should fail with GroupNotFound error
        assert!(result.is_err(), "Should fail for non-existent group");
        
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupNotFound) => {
                // Expected error
            }
            e => panic!("Expected GroupNotFound error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_promote_admin_if_none_ignores_left_members() {
        // Arrange: Create a group with active members and left members
        let service = create_service().await;
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        // Create users and group
        let (owner_user, _) = create_test_user("service_left_owner").await;
        let (active_user, _) = create_test_user("service_left_active").await;
        let (left_user, _) = create_test_user("service_left_left").await;
        
        // Create group and invitations
        let group_chat = create_test_group_chat("service_left_group", owner_user.id).await;
        let active_invitation = create_test_invitation(owner_user.id, active_user.id, group_chat.id).await;
        let left_invitation = create_test_invitation(owner_user.id, left_user.id, group_chat.id).await;
        
        // Create active membership
        let active_membership = GroupMembershipFactory::fake_new_group_membership_with_id(active_invitation.id);
        let active_membership_id = repository.insert(active_membership).await.unwrap();
        
        // Create left membership
        let left_membership = GroupMembershipFactory::fake_new_group_membership_with_id(left_invitation.id);
        let left_membership_id = repository.insert(left_membership).await.unwrap();
        
        // Update left membership to left status
        let update_left = ruggine_server::entity::group_membership::UpdateGroupMembership {
            id: left_membership_id,
            role: None,
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(Utc::now()),
            current_action: None,
        };
        repository.update(update_left).await.unwrap();

        // Act: Promote admin if none exists
        let result = service.promote_admin_if_none_internal(group_chat.id).await;

        // Assert: Should promote the active member, not the left one
        assert!(result.is_ok(), "Failed to promote admin: {:?}", result);
        
        let promoted_membership = result.unwrap();
        assert!(promoted_membership.is_some(), "Should have promoted the active member to admin");
        
        let promoted = promoted_membership.unwrap();
        assert_eq!(promoted.id, active_membership_id);
        assert_eq!(promoted.role, MemberRole::Admin);
        assert_eq!(promoted.user_id, active_user.id);
        assert_eq!(promoted.group_chat_id, group_chat.id);

        // Verify left member remains left with member role
        let left_check = repository.find_by_id_and_user_id(left_membership_id, left_user.id).await.unwrap();
        assert_eq!(left_check.role, MemberRole::Member);
        assert_eq!(left_check.membership_status, MembershipStatus::Left);

        // Cleanup
        cleanup_group_membership(active_membership_id).await;
        cleanup_group_membership(left_membership_id).await;
        cleanup_invitation(active_invitation.id).await;
        cleanup_invitation(left_invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email).await;
        cleanup_user_by_email(active_user.email).await;
        cleanup_user_by_email(left_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_promote_admin_if_none_returns_dto_format() {
        // Arrange: Create a group with one member to test DTO format
        let service = create_service().await;
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        
        // Create users and group
        let (owner_user, _) = create_test_user("service_dto_owner").await;
        let (member_user, _) = create_test_user("service_dto_member").await;
        
        // Create group and membership for regular member
        let group_chat = create_test_group_chat("service_dto_group", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create regular member membership (not admin)
        let membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(membership).await.unwrap();

        // Act: Promote admin if none exists
        let result = service.promote_admin_if_none_internal(group_chat.id).await;

        // Assert: Verify DTO structure and content
        assert!(result.is_ok(), "Failed to promote admin: {:?}", result);
        
        let promoted_membership = result.unwrap();
        assert!(promoted_membership.is_some(), "Should have promoted a member to admin");
        
        let promoted_dto = promoted_membership.unwrap();
        
        // Check DTO fields are properly populated
        assert_eq!(promoted_dto.id, membership_id);
        assert_eq!(promoted_dto.user_id, member_user.id);
        assert_eq!(promoted_dto.group_chat_id, group_chat.id);
        assert_eq!(promoted_dto.role, MemberRole::Admin);
        assert_eq!(promoted_dto.membership_status, MembershipStatus::Active);
        assert!(promoted_dto.left_at.is_none());

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(owner_user.email).await;
        cleanup_user_by_email(member_user.email).await;
    }
}
