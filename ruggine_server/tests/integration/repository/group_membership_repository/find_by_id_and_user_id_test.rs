use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use ruggine_server::model::group_membership_model::GroupMembershipWithInvitationRow;

#[cfg(test)]
mod group_membership_repository_integration_tests {
    use ruggine_server::model::group_membership_model::GroupMembershipWithInvitationRow;
    use super::*;
    use crate::{get_database, create_test_user, create_test_group_chat, cleanup_group_chat, cleanup_user, cleanup_group_membership, create_test_invitation, cleanup_invitation};

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_success() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_id_and_user_id_from_user").await;
        let (to_user, _) = create_test_user("find_by_id_and_user_id_from_user").await;
        let group_chat = create_test_group_chat("find_by_id_and_user_id", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        let result = repository.find_by_id_and_user_id(membership_id, to_user.id).await;

        assert!(result.is_ok(), "Failed to find membership by id");
        let found: GroupMembershipWithInvitationRow = result.unwrap();

        assert_eq!(found.id, membership_id);
        assert_eq!(found.user_id, to_user.id);
        assert_eq!(found.group_chat_id, group_chat.id);
        assert_eq!(found.role, new_membership.role);
        assert!(found.joined_at.timestamp() > 0);
        assert!(found.left_at.is_none());
        assert_eq!(found.invitation_id, invitation.id);
        assert_eq!(found.membership_status, MembershipStatus::Active);
        assert_eq!(found.invitation_id, new_membership.invitation_id);

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(to_user.email.clone()).await;
        cleanup_user(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_wrong_user_id() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("wrong_user_id_from_user").await;
        let (to_user, _) = create_test_user("wrong_user_id_to_user_id").await;
        let group_chat = create_test_group_chat("wrong_user_id", from_user.id).await;

        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership).await.unwrap();

        let result = repository.find_by_id_and_user_id(membership_id, -1).await;

        assert!(matches!(result, Err(sqlx::Error::RowNotFound)));

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(to_user.email.clone()).await;
        cleanup_user(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_admin_membership() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("admin_user_id_from_user").await;
        let (to_user, _) = create_test_user("admin_user_id_to_user").await;
        let group_chat = create_test_group_chat("admin_user_id", from_user.id).await;

        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;
        let new_membership = GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        let result = repository.find_by_id_and_user_id(membership_id, to_user.id).await;

        assert!(result.is_ok(), "Failed to find admin membership");
        let found = result.unwrap();

        assert_eq!(found.id, membership_id);
        assert_eq!(found.user_id, to_user.id);
        assert_eq!(found.group_chat_id, group_chat.id);
        assert_eq!(found.role, MemberRole::Admin);
        assert!(found.left_at.is_none());
        assert_eq!(found.membership_status, MembershipStatus::Active);
        assert_eq!(found.invitation_id, new_membership.invitation_id);

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(to_user.email.clone()).await;
        cleanup_user(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_not_found() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        let (user, _) = create_test_user("not_found_test").await;

        let result = repository.find_by_id_and_user_id(-1, user.id).await;
        assert!(matches!(result, Err(sqlx::Error::RowNotFound)));

        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_multiple_memberships() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (user1, _) = create_test_user("multi1").await;
        let (user2, _) = create_test_user("multi2").await;
        let group1 = create_test_group_chat("multi_gc1", user1.id).await;
        let group2 = create_test_group_chat("multi_gc2", user1.id).await;

        let invitation1 = create_test_invitation(user1.id, user1.id, group1.id).await;
        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let invitation2 = create_test_invitation(user1.id, user2.id, group2.id).await;
        let membership2 = GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation2.id);

        let id1 = repository.insert(membership1.clone()).await.unwrap();
        let id2 = repository.insert(membership2.clone()).await.unwrap();

        let found1 = repository.find_by_id_and_user_id(id1, user1.id).await.unwrap();
        let found2 = repository.find_by_id_and_user_id(id2, user2.id).await.unwrap();

        assert_eq!(found1.user_id, user1.id);
        assert_eq!(found1.group_chat_id, group1.id);
        assert_eq!(found1.role, MemberRole::Member);

        assert_eq!(found2.user_id, user2.id);
        assert_eq!(found2.group_chat_id, group2.id);
        assert_eq!(found2.role, MemberRole::Admin);

        cleanup_group_membership(id1).await;
        cleanup_group_membership(id2).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group1.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_user(user1.email.clone()).await;
        cleanup_user(user2.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_with_negative_id() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let result = repository.find_by_id_and_user_id(-1, -1).await;

        assert!(matches!(result, Err(sqlx::Error::RowNotFound)));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_with_zero_id() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let result = repository.find_by_id_and_user_id(0, 0).await;

        assert!(matches!(result, Err(sqlx::Error::RowNotFound)));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_unauthorized() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        let (from_user, _) = create_test_user("unauthorized1").await;
        let (to_user1, _) = create_test_user("unauthorized2").await;
        let (to_user2, _) = create_test_user("unauthorized3").await;
        let (to_user3, _) = create_test_user("unauthorized4").await;

        let group1 = create_test_group_chat("multi_gc1", from_user.id).await;

        let invitation1 = create_test_invitation(from_user.id, to_user1.id, group1.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user2.id, group1.id).await;
        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let membership2 = GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation2.id);

        let id1 = repository.insert(membership1.clone()).await.unwrap();
        let id2 = repository.insert(membership2.clone()).await.unwrap();

        // Act
        let result1 = repository.find_by_id_and_user_id(id1, to_user2.id).await;
        let result2 = repository.find_by_id_and_user_id(id2, to_user1.id).await;
        let result3 = repository.find_by_id_and_user_id(id1, to_user3.id).await;
        let result4 = repository.find_by_id_and_user_id(id2, to_user3.id).await;

        // Assert
        assert!(matches!(result1, Err(sqlx::Error::RowNotFound)));
        assert!(matches!(result2, Err(sqlx::Error::RowNotFound)));
        assert!(matches!(result3, Err(sqlx::Error::RowNotFound)));
        assert!(matches!(result4, Err(sqlx::Error::RowNotFound)));

        // Clean up
        cleanup_group_membership(id1).await;
        cleanup_group_membership(id2).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group1.id).await;
        cleanup_user(from_user.email.clone()).await;
        cleanup_user(to_user1.email.clone()).await;
        cleanup_user(to_user2.email.clone()).await;
        cleanup_user(to_user3.email.clone()).await;

    }
}
