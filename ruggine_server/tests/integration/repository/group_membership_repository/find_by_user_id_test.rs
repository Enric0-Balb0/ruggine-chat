use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use ruggine_server::model::group_membership_model::GroupMembershipWithInvitationRow;

#[cfg(test)]
mod group_membership_repository_find_by_user_id_integration_tests {
    use super::*;
    use crate::{
        get_database, create_test_user, create_test_group_chat, cleanup_group_chat, 
        cleanup_user, cleanup_group_membership, create_test_invitation, cleanup_invitation
    };

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_success_single_membership() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_user_id_from_single").await;
        let (to_user, _) = create_test_user("find_by_user_id_to_single").await;
        let group_chat = create_test_group_chat("find_by_user_id_single", from_user.id).await;
        let invitation = create_test_invitation(from_user.id, to_user.id, group_chat.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation.id);
        let membership_id = repository.insert(new_membership.clone()).await.unwrap();

        let result = repository.find_by_user_id(to_user.id).await;

        assert!(result.is_ok(), "Failed to find memberships by user_id");
        let found: Vec<GroupMembershipWithInvitationRow> = result.unwrap();

        assert_eq!(found.len(), 1);
        let membership = &found[0];
        assert_eq!(membership.id, membership_id);
        assert_eq!(membership.user_id, to_user.id);
        assert_eq!(membership.group_chat_id, group_chat.id);
        assert_eq!(membership.role, new_membership.role);
        assert!(membership.joined_at.timestamp() > 0);
        assert!(membership.left_at.is_none());
        assert_eq!(membership.invitation_id, invitation.id);
        assert_eq!(membership.membership_status, MembershipStatus::Active);

        cleanup_group_membership(membership_id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(to_user.email.clone()).await;
        cleanup_user(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_multiple_memberships() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_user_id_from_multi").await;
        let (to_user, _) = create_test_user("find_by_user_id_to_multi").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_multi1", from_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_multi2", from_user.id).await;
        let group_chat3 = create_test_group_chat("find_by_user_id_multi3", from_user.id).await;
        
        let invitation1 = create_test_invitation(from_user.id, to_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user.id, group_chat2.id).await;
        let invitation3 = create_test_invitation(from_user.id, to_user.id, group_chat3.id).await;

        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let membership2 = GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation2.id);
        let membership3 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation3.id);

        let id1 = repository.insert(membership1.clone()).await.unwrap();
        let id2 = repository.insert(membership2.clone()).await.unwrap();
        let id3 = repository.insert(membership3.clone()).await.unwrap();

        let result = repository.find_by_user_id(to_user.id).await;

        assert!(result.is_ok(), "Failed to find multiple memberships");
        let found = result.unwrap();
        assert_eq!(found.len(), 3);

        // Verifico che tutti i membership appartengono al to_user
        for membership in &found {
            assert_eq!(membership.user_id, to_user.id);
        }

        // Verifico che abbiamo tutti i group_chat_id
        let group_chat_ids: std::collections::HashSet<_> = found.iter().map(|m| m.group_chat_id).collect();
        assert!(group_chat_ids.contains(&group_chat1.id));
        assert!(group_chat_ids.contains(&group_chat2.id));
        assert!(group_chat_ids.contains(&group_chat3.id));

        // Verifico che abbiamo il ruolo admin in uno dei membership
        let has_admin = found.iter().any(|m| m.role == MemberRole::Admin);
        assert!(has_admin, "Should have at least one Admin role");

        cleanup_group_membership(id1).await;
        cleanup_group_membership(id2).await;
        cleanup_group_membership(id3).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_invitation(invitation3.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_group_chat(group_chat3.id).await;
        cleanup_user(to_user.email.clone()).await;
        cleanup_user(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_empty_result() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);
        let (user, _) = create_test_user("find_by_user_id_empty").await;

        let result = repository.find_by_user_id(user.id).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_nonexistent_user() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let result = repository.find_by_user_id(-1).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_with_admin_and_member_roles() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_user_id_from_roles").await;
        let (to_user, _) = create_test_user("find_by_user_id_to_roles").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_roles1", from_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_roles2", from_user.id).await;
        
        let invitation1 = create_test_invitation(from_user.id, to_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user.id, group_chat2.id).await;

        let member_membership = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let admin_membership = GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation2.id);

        let member_id = repository.insert(member_membership).await.unwrap();
        let admin_id = repository.insert(admin_membership).await.unwrap();

        let result = repository.find_by_user_id(to_user.id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);

        let mut has_member = false;
        let mut has_admin = false;

        for membership in &found {
            assert_eq!(membership.user_id, to_user.id);
            assert_eq!(membership.membership_status, MembershipStatus::Active);
            
            match membership.role {
                MemberRole::Member => has_member = true,
                MemberRole::Admin => has_admin = true,
            }
        }

        assert!(has_member, "Should have Member role");
        assert!(has_admin, "Should have Admin role");

        cleanup_group_membership(member_id).await;
        cleanup_group_membership(admin_id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(to_user.email.clone()).await;
        cleanup_user(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_ordered_by_joined_at() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_user_id_from_ordered").await;
        let (to_user, _) = create_test_user("find_by_user_id_to_ordered").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_ordered1", from_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_ordered2", from_user.id).await;
        
        let invitation1 = create_test_invitation(from_user.id, to_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user.id, group_chat2.id).await;

        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let membership2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation2.id);

        // Inserisco con un piccolo delay per garantire ordine temporale diverso
        let id1 = repository.insert(membership1).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let id2 = repository.insert(membership2).await.unwrap();

        let result = repository.find_by_user_id(to_user.id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);

        // Verifico che sono ordinati per joined_at DESC (il più recente prima)
        assert!(found[0].joined_at >= found[1].joined_at);

        cleanup_group_membership(id1).await;
        cleanup_group_membership(id2).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(to_user.email.clone()).await;
        cleanup_user(from_user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_different_users_isolation() {
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        let (from_user, _) = create_test_user("find_by_user_id_from_isolation").await;
        let (to_user1, _) = create_test_user("find_by_user_id_to1_isolation").await;
        let (to_user2, _) = create_test_user("find_by_user_id_to2_isolation").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_isolation1", from_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_isolation2", from_user.id).await;
        
        let invitation1 = create_test_invitation(from_user.id, to_user1.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(from_user.id, to_user2.id, group_chat2.id).await;

        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation1.id);
        let membership2 = GroupMembershipFactory::fake_new_group_membership_with_id(invitation2.id);

        let id1 = repository.insert(membership1).await.unwrap();
        let id2 = repository.insert(membership2).await.unwrap();

        // Test if the user see only his memberships
        let result1 = repository.find_by_user_id(to_user1.id).await;
        let result2 = repository.find_by_user_id(to_user2.id).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let found1 = result1.unwrap();
        let found2 = result2.unwrap();

        assert_eq!(found1.len(), 1);
        assert_eq!(found2.len(), 1);

        assert_eq!(found1[0].user_id, to_user1.id);
        assert_eq!(found1[0].group_chat_id, group_chat1.id);

        assert_eq!(found2[0].user_id, to_user2.id);
        assert_eq!(found2[0].group_chat_id, group_chat2.id);

        cleanup_group_membership(id1).await;
        cleanup_group_membership(id2).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(to_user1.email.clone()).await;
        cleanup_user(to_user2.email.clone()).await;
        cleanup_user(from_user.email.clone()).await;
    }
}
