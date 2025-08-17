use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use ruggine_server::model::group_membership_model::GroupMembershipWithInvitationRow;
use sqlx::Error;

#[cfg(test)]
mod group_membership_repository_find_active_by_user_id_and_group_id_integration_tests {
    use super::*;
    use crate::{
        accept_test_invitation, cleanup_group_chat, cleanup_group_membership, cleanup_invitation, cleanup_user_by_email, create_test_group_chat, create_test_invitation, create_test_user, get_database
    };

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_success() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_active_uid_gid_from").await;
        let (to_user, _) = create_test_user("find_active_uid_gid_to").await;
        let group_chat = create_test_group_chat("find_active_uid_gid_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        let result = repository.find_active_by_user_id_and_group_id_inner(to_user.id, group_chat.id).await;

        assert!(result.is_ok(), "Failed to find active membership by user_id and group_id");
        let found: GroupMembershipWithInvitationRow = result.unwrap();

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
    async fn test_find_active_by_user_id_and_group_id_admin_role() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_active_admin_from").await;
        let (to_user, _) = create_test_user("find_active_admin_to").await;
        let group_chat = create_test_group_chat("find_active_admin_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        let result = repository.find_active_by_user_id_and_group_id_inner(to_user.id, group_chat.id).await;

        assert!(result.is_ok());
        let found = result.unwrap();

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
    async fn test_find_active_by_user_id_and_group_id_not_found_wrong_user() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_active_nf_from").await;
        let (to_user, _) = create_test_user("find_active_nf_to").await;
        let (wrong_user, _) = create_test_user("find_active_nf_wrong").await;
        let group_chat = create_test_group_chat("find_active_nf_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        // Try to find with wrong user
        let result = repository.find_active_by_user_id_and_group_id_inner(wrong_user.id, group_chat.id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(wrong_user.email.clone()).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_not_found_wrong_group() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_active_ng_from").await;
        let (to_user, _) = create_test_user("find_active_ng_to").await;
        let group_chat = create_test_group_chat("find_active_ng_group", from_user.id).await;
        let wrong_group = create_test_group_chat("find_active_ng_wrong_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        // Try to find with wrong group
        let result = repository.find_active_by_user_id_and_group_id_inner(to_user.id, wrong_group.id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(wrong_group.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_not_found_left_membership() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_active_left_from").await;
        let (to_user, _) = create_test_user("find_active_left_to").await;
        let group_chat = create_test_group_chat("find_active_left_group", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        // Create membership and then leave it
        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();
        
        // Update membership to left status
        let update_membership = ruggine_server::entity::group_membership::UpdateGroupMembership {
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(chrono::Utc::now()),
            id: membership_id,
            role: None
        };
        repository.update(update_membership).await.unwrap();

        // Try to find active membership for left user
        let result = repository.find_active_by_user_id_and_group_id_inner(to_user.id, group_chat.id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_nonexistent_ids() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Test with completely nonexistent IDs
        let result = repository.find_active_by_user_id_and_group_id_inner(-999, -888).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_multiple_same_group_only_one_active() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_active_multi_from").await;
        let (to_user, _) = create_test_user("find_active_multi_to").await;
        let group_chat = create_test_group_chat("find_active_multi_group", from_user.id).await;
        
        // Create first invitation and membership (will be left)
        let invitation1 = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        accept_test_invitation(invitation1.id).await;
        let new_membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let membership_id1 = repository.insert(new_membership1.clone()).await.unwrap();
        
        // Update first membership to left status
        let update_membership = ruggine_server::entity::group_membership::UpdateGroupMembership {
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(chrono::Utc::now()),
            id: membership_id1,
            role: None,
        };
        repository.update(update_membership).await.unwrap();

        // Create second invitation and membership (will be active)
        let invitation2 = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        accept_test_invitation(invitation2.id).await;
        let new_membership2 = GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation2.id);
        let membership_id2 = repository.insert(new_membership2.clone()).await.unwrap();

        // Find active membership - should only return the active one
        let result = repository.find_active_by_user_id_and_group_id_inner(to_user.id, group_chat.id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        
        assert_eq!(found.id, membership_id2);
        assert_eq!(found.user_id, to_user.id);
        assert_eq!(found.group_chat_id, group_chat.id);
        assert_eq!(found.role, MemberRole::Admin);
        assert_eq!(found.membership_status, MembershipStatus::Active);
        assert!(found.left_at.is_none());

        cleanup_group_membership(membership_id2).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_membership(membership_id1).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(to_user.email.clone()).await;
        cleanup_user_by_email(from_user.email.clone()).await;
    }
}