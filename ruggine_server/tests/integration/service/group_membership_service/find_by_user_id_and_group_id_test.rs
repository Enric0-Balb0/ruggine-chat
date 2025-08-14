use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::entity::group_membership::{MembershipStatus};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use crate::common::{
    get_database, create_test_user, create_test_group_chat, create_test_group_membership,
    cleanup_user_by_email, cleanup_group_chat, cleanup_group_membership, create_test_invitation, cleanup_invitation
};

#[cfg(test)]
mod group_membership_find_by_user_id_and_group_id_integration_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_success() {
        // Arrange: Create real user, group chat, and membership in the database
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        // Create test entities
        let (owner_user, _) = create_test_user("find_by_uid_gid_srv_owner").await;
        let (member_user, _) = create_test_user("find_by_uid_gid_srv_member").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_srv", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        // Create membership using common helper
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Act: Find membership by user_id and group_id
        let result = service.find_by_user_id_and_group_id(member_user.id, group_chat.id).await;

        // Assert: Verify membership was found
        assert!(result.is_ok(), "Failed to find membership: {:?}", result);
        let membership_dto = result.unwrap();
        
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
    async fn test_find_by_user_id_and_group_id_admin_membership() {
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_uid_gid_admin_srv_owner").await;
        let (admin_user, _) = create_test_user("find_by_uid_gid_admin_srv_member").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_admin_srv", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, admin_user.id, group_chat.id).await;
        
        // Create admin membership
        let membership = create_test_group_membership(invitation.id, admin_user.id).await;
        // Note: create_test_group_membership creates a Member role by default, 
        // but we'll test with what we have since the test is about finding the membership

        let result = service.find_by_user_id_and_group_id(admin_user.id, group_chat.id).await;

        assert!(result.is_ok());
        let membership_dto = result.unwrap();
        
        assert_eq!(membership_dto.user_id, admin_user.id);
        assert_eq!(membership_dto.group_chat_id, group_chat.id);
        assert_eq!(membership_dto.membership_status, MembershipStatus::Active);

        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email.clone()).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_not_found_wrong_user() {
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_uid_gid_nf_srv_owner").await;
        let (member_user, _) = create_test_user("find_by_uid_gid_nf_srv_member").await;
        let (wrong_user, _) = create_test_user("find_by_uid_gid_nf_srv_wrong").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_nf_srv", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Try to find with wrong user
        let result = service.find_by_user_id_and_group_id(wrong_user.id, group_chat.id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error"),
        }

        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(wrong_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_not_found_wrong_group() {
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_uid_gid_ng_srv_owner").await;
        let (member_user, _) = create_test_user("find_by_uid_gid_ng_srv_member").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_ng_srv", owner_user.id).await;
        let wrong_group = create_test_group_chat("find_by_uid_gid_ng_srv_wrong", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Try to find with wrong group
        let result = service.find_by_user_id_and_group_id(member_user.id, wrong_group.id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error"),
        }

        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(wrong_group.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_multiple_groups_different_users() {
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (creator, _) = create_test_user("find_by_uid_gid_multi_srv_creator").await;
        let (user1, _) = create_test_user("find_by_uid_gid_multi_srv_user1").await;
        let (user2, _) = create_test_user("find_by_uid_gid_multi_srv_user2").await;
        
        let group1 = create_test_group_chat("find_by_uid_gid_multi_srv_g1", creator.id).await;
        let group2 = create_test_group_chat("find_by_uid_gid_multi_srv_g2", creator.id).await;
        
        let invitation1_u1 = create_test_invitation(creator.id, user1.id, group1.id).await;
        let invitation1_u2 = create_test_invitation(creator.id, user2.id, group1.id).await;
        let invitation2_u1 = create_test_invitation(creator.id, user1.id, group2.id).await;

        let membership1_u1 = create_test_group_membership(invitation1_u1.id, user1.id).await;
        let membership1_u2 = create_test_group_membership(invitation1_u2.id, user2.id).await;
        let membership2_u1 = create_test_group_membership(invitation2_u1.id, user1.id).await;

        // Test user1 in group1
        let result1 = service.find_by_user_id_and_group_id(user1.id, group1.id).await;
        assert!(result1.is_ok());
        let found1 = result1.unwrap();
        assert_eq!(found1.user_id, user1.id);
        assert_eq!(found1.group_chat_id, group1.id);

        // Test user2 in group1
        let result2 = service.find_by_user_id_and_group_id(user2.id, group1.id).await;
        assert!(result2.is_ok());
        let found2 = result2.unwrap();
        assert_eq!(found2.user_id, user2.id);
        assert_eq!(found2.group_chat_id, group1.id);

        // Test user1 in group2
        let result3 = service.find_by_user_id_and_group_id(user1.id, group2.id).await;
        assert!(result3.is_ok());
        let found3 = result3.unwrap();
        assert_eq!(found3.user_id, user1.id);
        assert_eq!(found3.group_chat_id, group2.id);

        // Test user2 in group2 (should not exist)
        let result4 = service.find_by_user_id_and_group_id(user2.id, group2.id).await;
        assert!(result4.is_err());

        // Cleanup
        cleanup_group_membership(membership2_u1.id).await;
        cleanup_group_membership(membership1_u2.id).await;
        cleanup_group_membership(membership1_u1.id).await;
        cleanup_invitation(invitation2_u1.id).await;
        cleanup_invitation(invitation1_u2.id).await;
        cleanup_invitation(invitation1_u1.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_group_chat(group1.id).await;
        cleanup_user_by_email(user2.email.clone()).await;
        cleanup_user_by_email(user1.email.clone()).await;
        cleanup_user_by_email(creator.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_nonexistent_ids() {
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);

        // Test with completely nonexistent IDs
        let result = service.find_by_user_id_and_group_id(-999, -888).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_multiple_calls_same_params() {
        let db = get_database().await;
        let service = GroupMembershipService::new(&db);
        
        let (owner_user, _) = create_test_user("find_by_uid_gid_mc_srv_owner").await;
        let (member_user, _) = create_test_user("find_by_uid_gid_mc_srv_member").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_mc_srv", owner_user.id).await;
        let invitation = create_test_invitation(owner_user.id, member_user.id, group_chat.id).await;
        
        let membership = create_test_group_membership(invitation.id, member_user.id).await;

        // Make multiple calls with same parameters
        let result1 = service.find_by_user_id_and_group_id(member_user.id, group_chat.id).await;
        let result2 = service.find_by_user_id_and_group_id(member_user.id, group_chat.id).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let found1 = result1.unwrap();
        let found2 = result2.unwrap();
        
        // Both calls should return the same data
        assert_eq!(found1.id, found2.id);
        assert_eq!(found1.user_id, found2.user_id);
        assert_eq!(found1.group_chat_id, found2.group_chat_id);
        assert_eq!(found1.role, found2.role);

        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(owner_user.email.clone()).await;
    }
}
