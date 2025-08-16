use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use ruggine_server::model::group_membership_model::GroupMembershipWithInvitationRow;

#[cfg(test)]
mod group_membership_repository_find_by_group_id_integration_tests {
    use super::*;
    use crate::{
        get_database, create_test_user, create_test_group_chat, cleanup_group_chat,
        cleanup_user_by_email, cleanup_group_membership, create_test_invitation, cleanup_invitation
    };

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_success_single_membership() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_group_id_from_single").await;
        let (to_user, _) = create_test_user("find_by_group_id_to_single").await;
        let group_chat = create_test_group_chat("find_by_group_id_single", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        let result = repository.find_by_group_id(group_chat.id).await;

        assert!(result.is_ok(), "Failed to find memberships by group_id");
        let found: Vec<GroupMembershipWithInvitationRow> = result.unwrap();

        assert_eq!(found.len(), 1);
        let membership = &found[0];
        assert_eq!(membership.id, membership_id);
        assert_eq!(membership.user_id, to_user.id);
        assert_eq!(membership.group_chat_id, group_chat.id);
        assert_eq!(membership.role, new_membership.role);
        assert!(membership.joined_at.timestamp() > 0);
        assert_eq!(membership.invitation_id, invitation.id);
        assert_eq!(membership.membership_status, MembershipStatus::Active);

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_success_multiple_memberships() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_group_id_from_multi").await;
        let (to_user1, _) = create_test_user("find_by_group_id_to1_multi").await;
        let (to_user2, _) = create_test_user("find_by_group_id_to2_multi").await;
        let (to_user3, _) = create_test_user("find_by_group_id_to3_multi").await;
        let group_chat = create_test_group_chat("find_by_group_id_multi", from_user.id).await;

        // Create invitations and memberships for multiple users
        let invitation1 = create_test_invitation(from_user.id, to_user1.id, group_chat.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user2.id, group_chat.id).await;
        let invitation3 = create_test_invitation(from_user.id, to_user3.id, group_chat.id).await;

        let new_membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let new_membership2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation2.id);
        let new_membership3 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation3.id);

        let membership_id1 = repository.insert(new_membership1).await.unwrap();
        let membership_id2 = repository.insert(new_membership2).await.unwrap();
        let membership_id3 = repository.insert(new_membership3).await.unwrap();

        let result = repository.find_by_group_id(group_chat.id).await;

        assert!(result.is_ok(), "Failed to find memberships by group_id");
        let found: Vec<GroupMembershipWithInvitationRow> = result.unwrap();

        assert_eq!(found.len(), 3);
        
        // Verify all memberships belong to the same group
        for membership in &found {
            assert_eq!(membership.group_chat_id, group_chat.id);
            assert_eq!(membership.membership_status, MembershipStatus::Active);
            assert!(membership.joined_at.timestamp() > 0);
        }

        // Verify all user IDs are present
        let found_user_ids: std::collections::HashSet<i32> = found.iter().map(|m| m.user_id).collect();
        let expected_user_ids: std::collections::HashSet<i32> = [to_user1.id, to_user2.id, to_user3.id].iter().cloned().collect();
        assert_eq!(found_user_ids, expected_user_ids);

        // Verify memberships are ordered by joined_at ASC
        for i in 1..found.len() {
            assert!(
                found[i - 1].joined_at <= found[i].joined_at,
                "Memberships should be ordered by joined_at ASC"
            );
        }

        // Cleanup
        cleanup_group_membership(membership_id1).await;
        cleanup_group_membership(membership_id2).await;
        cleanup_group_membership(membership_id3).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_invitation(invitation3.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user1.email.clone()).await;
        cleanup_user_by_email(to_user2.email.clone()).await;
        cleanup_user_by_email(to_user3.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_empty_result() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let non_existing_group_id = 999999;
        let result = repository.find_by_group_id(non_existing_group_id).await;

        assert!(result.is_ok(), "Failed to query non-existing group");
        let found: Vec<GroupMembershipWithInvitationRow> = result.unwrap();
        assert_eq!(found.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_only_active_memberships() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_group_id_from_active").await;
        let (to_user1, _) = create_test_user("find_by_group_id_to1_active").await;
        let (to_user2, _) = create_test_user("find_by_group_id_to2_active").await;
        let group_chat = create_test_group_chat("find_by_group_id_active", from_user.id).await;

        // Create active membership
        let invitation1 = create_test_invitation(from_user.id, to_user1.id, group_chat.id).await;
        let new_membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let membership_id1 = repository.insert(new_membership1).await.unwrap();

        // Create membership that will be set to left status
        let invitation2 = create_test_invitation(from_user.id, to_user2.id, group_chat.id).await;
        let new_membership2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation2.id);
        let membership_id2 = repository.insert(new_membership2).await.unwrap();

        // Update second membership to Left status
        let update_membership = ruggine_server::entity::group_membership::UpdateGroupMembership {
            id: membership_id2,
            role: None,
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(chrono::Utc::now()),
        };
        repository.update(update_membership).await.unwrap();

        let result = repository.find_by_group_id(group_chat.id).await;

        assert!(result.is_ok(), "Failed to find memberships by group_id");
        let found: Vec<GroupMembershipWithInvitationRow> = result.unwrap();

        // Should find only active memberships (user who left should not be included)
        assert_eq!(found.len(), 1);
        
        // Verify the found membership is active and belongs to the correct user
        let membership = &found[0];
        assert_eq!(membership.group_chat_id, group_chat.id);
        assert_eq!(membership.membership_status, MembershipStatus::Active);
        assert_eq!(membership.user_id, to_user1.id);

        // Cleanup
        cleanup_group_membership(membership_id1).await;
        cleanup_group_membership(membership_id2).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user1.email.clone()).await;
        cleanup_user_by_email(to_user2.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_id_with_different_roles() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_group_id_from_roles").await;
        let (admin_user, _) = create_test_user("find_by_group_id_admin_roles").await;
        let (member_user, _) = create_test_user("find_by_group_id_member_roles").await;
        let group_chat = create_test_group_chat("find_by_group_id_roles", from_user.id).await;

        // Create admin invitation and membership
        let admin_invitation = create_test_invitation(from_user.id, admin_user.id, group_chat.id).await;
        let mut admin_membership = GroupMembershipFactory::fake_new_group_membership_with_id(admin_invitation.id);
        admin_membership.role = MemberRole::Admin;
        let admin_membership_id = repository.insert(admin_membership).await.unwrap();

        // Create member invitation and membership
        let member_invitation = create_test_invitation(from_user.id, member_user.id, group_chat.id).await;
        let mut member_membership = GroupMembershipFactory::fake_new_group_membership_with_id(member_invitation.id);
        member_membership.role = MemberRole::Member;
        let member_membership_id = repository.insert(member_membership).await.unwrap();

        let result = repository.find_by_group_id(group_chat.id).await;

        assert!(result.is_ok(), "Failed to find memberships by group_id");
        let found: Vec<GroupMembershipWithInvitationRow> = result.unwrap();

        assert_eq!(found.len(), 2);
        
        // Find admin and member roles
        let admin_memberships: Vec<&GroupMembershipWithInvitationRow> = found.iter()
            .filter(|m| m.role == MemberRole::Admin)
            .collect();
        let member_memberships: Vec<&GroupMembershipWithInvitationRow> = found.iter()
            .filter(|m| m.role == MemberRole::Member)
            .collect();

        assert_eq!(admin_memberships.len(), 1);
        assert_eq!(member_memberships.len(), 1);
        assert_eq!(admin_memberships[0].user_id, admin_user.id);
        assert_eq!(member_memberships[0].user_id, member_user.id);

        // Cleanup
        cleanup_group_membership(admin_membership_id).await;
        cleanup_group_membership(member_membership_id).await;
        cleanup_invitation(admin_invitation.id).await;
        cleanup_invitation(member_invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }
}
