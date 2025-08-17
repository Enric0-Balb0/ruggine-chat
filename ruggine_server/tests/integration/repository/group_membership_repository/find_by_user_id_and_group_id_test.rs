use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use ruggine_server::model::group_membership_model::GroupMembershipWithInvitationRow;

#[cfg(test)]
mod group_membership_repository_find_by_user_id_and_group_id_integration_tests {
    use ruggine_server::entity::group_membership::all_membership_statuses;
    use super::*;
    use crate::{
        get_database, create_test_user, create_test_group_chat, cleanup_group_chat,
        cleanup_user_by_email, cleanup_group_membership, create_test_invitation, cleanup_invitation
    };

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_success() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_uid_gid_from").await;
        let (to_user, _) = create_test_user("find_by_uid_gid_to").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        let result = repository.find_by_user_id_and_group_id(to_user.id, group_chat.id, all_membership_statuses()).await;

        assert!(result.is_ok(), "Failed to find membership by user_id and group_id");
        let found_vec = result.unwrap();
        assert!(!found_vec.is_empty(), "Expected at least one membership");
        let found: &GroupMembershipWithInvitationRow = &found_vec[0];

        assert_eq!(found.id, membership_id);
        assert_eq!(found.user_id, to_user.id);
        assert_eq!(found.group_chat_id, group_chat.id);
        assert_eq!(found.role, new_membership.role);
        assert!(found.joined_at.timestamp() > 0);
        assert!(found.left_at.is_none());
        assert_eq!(found.invitation_id, invitation.id);
        assert_eq!(found.membership_status, MembershipStatus::Active);

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_admin_membership() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_uid_gid_admin_from").await;
        let (to_user, _) = create_test_user("find_by_uid_gid_admin_to").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_admin_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        let result = repository.find_by_user_id_and_group_id(to_user.id, group_chat.id, all_membership_statuses()).await;

        assert!(result.is_ok());
        let found_vec = result.unwrap();
        assert!(!found_vec.is_empty());
        let found = &found_vec[0];

        assert_eq!(found.id, membership_id);
        assert_eq!(found.user_id, to_user.id);
        assert_eq!(found.group_chat_id, group_chat.id);
        assert_eq!(found.role, MemberRole::Admin);
        assert_eq!(found.membership_status, MembershipStatus::Active);
        assert!(found.left_at.is_none());

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_not_found_wrong_user() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_uid_gid_nf_from").await;
        let (to_user, _) = create_test_user("find_by_uid_gid_nf_to").await;
        let (wrong_user, _) = create_test_user("find_by_uid_gid_nf_wrong").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_nf_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        // Try to find with wrong user
        let result = repository.find_by_user_id_and_group_id(wrong_user.id, group_chat.id, all_membership_statuses()).await;

        assert!(result.is_ok());
        let found_vec = result.unwrap();
        assert!(found_vec.is_empty(), "Expected no memberships for wrong user");

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(wrong_user.email.clone()).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_not_found_wrong_group() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_uid_gid_ng_from").await;
        let (to_user, _) = create_test_user("find_by_uid_gid_ng_to").await;
        let group_chat = create_test_group_chat("find_by_uid_gid_ng_group", from_user.id).await;
        let wrong_group = create_test_group_chat("find_by_uid_gid_ng_wrong_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        // Try to find with wrong group
        let result = repository.find_by_user_id_and_group_id(to_user.id, wrong_group.id, all_membership_statuses()).await;

        assert!(result.is_ok());
        let found_vec = result.unwrap();
        assert!(found_vec.is_empty(), "Expected no memberships for wrong group");

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(wrong_group.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_multiple_groups_different_users() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (creator, _) = create_test_user("find_by_uid_gid_multi_creator").await;
        let (user1, _) = create_test_user("find_by_uid_gid_multi_user1").await;
        let (user2, _) = create_test_user("find_by_uid_gid_multi_user2").await;

        let group1 = create_test_group_chat("find_by_uid_gid_multi_group1", creator.id).await;
        let group2 = create_test_group_chat("find_by_uid_gid_multi_group2", creator.id).await;

        let invitation1_u1 = create_test_invitation(creator.id, user1.id, group1.id).await;
        let invitation1_u2 = create_test_invitation(creator.id, user2.id, group1.id).await;
        let invitation2_u1 = create_test_invitation(creator.id, user1.id, group2.id).await;

        let membership1_u1 = repository.insert(
            GroupMembershipFactory::fake_new_group_membership_with_id(invitation1_u1.id)
        ).await.unwrap();
        let membership1_u2 = repository.insert(
            GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation1_u2.id)
        ).await.unwrap();
        let membership2_u1 = repository.insert(
            GroupMembershipFactory::fake_new_group_membership_with_id(invitation2_u1.id)
        ).await.unwrap();

        // Test user1 in group1
        let result1 = repository.find_by_user_id_and_group_id(user1.id, group1.id, all_membership_statuses()).await.unwrap();
        assert_eq!(result1.len(), 1);
        let found1 = &result1[0];
        assert_eq!(found1.user_id, user1.id);
        assert_eq!(found1.group_chat_id, group1.id);
        assert_eq!(found1.role, MemberRole::Member);

        // Test user2 in group1 (admin)
        let result2 = repository.find_by_user_id_and_group_id(user2.id, group1.id, all_membership_statuses()).await.unwrap();
        assert_eq!(result2.len(), 1);
        let found2 = &result2[0];
        assert_eq!(found2.user_id, user2.id);
        assert_eq!(found2.group_chat_id, group1.id);
        assert_eq!(found2.role, MemberRole::Admin);

        // Test user1 in group2
        let result3 = repository.find_by_user_id_and_group_id(user1.id, group2.id, all_membership_statuses()).await.unwrap();
        assert_eq!(result3.len(), 1);
        let found3 = &result3[0];
        assert_eq!(found3.user_id, user1.id);
        assert_eq!(found3.group_chat_id, group2.id);

        // Test user2 in group2 (should not exist)
        let result4 = repository.find_by_user_id_and_group_id(user2.id, group2.id, all_membership_statuses()).await.unwrap();
        assert!(result4.is_empty(), "Expected no membership for user2 in group2");

        // Cleanup
        cleanup_group_membership(membership2_u1).await;
        cleanup_group_membership(membership1_u2).await;
        cleanup_group_membership(membership1_u1).await;
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
        let repository = GroupMembershipRepository::new(&db);

        // Test with completely nonexistent IDs
        let result = repository.find_by_user_id_and_group_id(-999, -888, all_membership_statuses()).await.unwrap();
        assert!(result.is_empty(), "Expected no memberships for nonexistent IDs");
    }
}
