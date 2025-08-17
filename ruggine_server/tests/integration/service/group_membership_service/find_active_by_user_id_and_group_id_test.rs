use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;

#[cfg(test)]
mod group_membership_service_find_active_by_user_id_and_group_id_integration_tests {
    use super::*;
    use crate::{
        add_test_user_to_a_group, cleanup_group_chat, cleanup_group_membership, cleanup_invitation, cleanup_test_user_from_a_group_chat, cleanup_user_by_email, create_test_group_chat_with_invitation_and_membership, create_test_user, get_database
    };
    use ruggine_server::utils::service_initializer::ServiceInitializer;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_success() {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let service = service_init.group_membership_service();

        let (creator, _) = create_test_user("find_active_svc_creator").await;
        let (member_user, _) = create_test_user("find_active_svc_member").await;
        
        // Create group chat with creator membership
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_active_svc_group", creator.id).await;
        
        // Add member to group
        let membership = add_test_user_to_a_group(member_user.id, &group_chat).await;

        let result = service.find_active_by_user_id_and_group_id(member_user.id, group_chat.id).await;

        assert!(result.is_ok(), "Failed to find active membership by user_id and group_id");
        let found_dto = result.unwrap();

        assert_eq!(found_dto.user_id, member_user.id);
        assert_eq!(found_dto.group_chat_id, group_chat.id);
        assert_eq!(found_dto.role, MemberRole::Member);
        assert_eq!(found_dto.membership_status, MembershipStatus::Active);
        assert!(found_dto.left_at.is_none());

        // Cleanup
        cleanup_test_user_from_a_group_chat(creator.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(creator.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_admin_role() {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let service = service_init.group_membership_service();

        let (creator, _) = create_test_user("find_active_svc_admin_creator").await;
        let (admin_user, _) = create_test_user("find_active_svc_admin_member").await;
        
        // Create group chat with creator membership
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_active_svc_admin_group", creator.id).await;
        
        // Create admin invitation and membership manually
        let invitation = crate::create_test_admin_invitation(creator.id, admin_user.id, group_chat.id).await;
        let membership = crate::create_test_admin_group_membership(invitation.id, admin_user.id).await;

        let result = service.find_active_by_user_id_and_group_id(admin_user.id, group_chat.id).await;

        assert!(result.is_ok());
        let found_dto = result.unwrap();

        assert_eq!(found_dto.user_id, admin_user.id);
        assert_eq!(found_dto.group_chat_id, group_chat.id);
        assert_eq!(found_dto.role, MemberRole::Admin);
        assert_eq!(found_dto.membership_status, MembershipStatus::Active);
        assert!(found_dto.left_at.is_none());

        // Cleanup
        cleanup_test_user_from_a_group_chat(creator.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email.clone()).await;
        cleanup_user_by_email(creator.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_not_found_wrong_user() {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let service = service_init.group_membership_service();

        let (creator, _) = create_test_user("find_active_svc_nf_creator").await;
        let (member_user, _) = create_test_user("find_active_svc_nf_member").await;
        let (wrong_user, _) = create_test_user("find_active_svc_nf_wrong").await;
        
        // Create group chat with creator membership
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_active_svc_nf_group", creator.id).await;
        
        // Add member to group
        let membership = add_test_user_to_a_group(member_user.id, &group_chat).await;

        // Try to find with wrong user
        let result = service.find_active_by_user_id_and_group_id(wrong_user.id, group_chat.id).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)
        ));

        // Cleanup
        cleanup_test_user_from_a_group_chat(creator.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(wrong_user.email.clone()).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(creator.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_not_found_wrong_group() {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let service = service_init.group_membership_service();

        let (creator, _) = create_test_user("find_active_svc_ng_creator").await;
        let (member_user, _) = create_test_user("find_active_svc_ng_member").await;
        
        // Create two group chats
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_active_svc_ng_group", creator.id).await;
        let wrong_group = create_test_group_chat_with_invitation_and_membership("find_active_svc_ng_wrong_group", creator.id).await;
        
        // Add member to first group only
        let membership = add_test_user_to_a_group(member_user.id, &group_chat).await;

        // Try to find with wrong group
        let result = service.find_active_by_user_id_and_group_id(member_user.id, wrong_group.id).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)
        ));

        // Cleanup
        cleanup_test_user_from_a_group_chat(creator.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(creator.id, wrong_group.id).await;
        cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
        cleanup_group_chat(wrong_group.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(creator.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_not_found_left_membership() {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let service = service_init.group_membership_service();

        let (creator, _) = create_test_user("find_active_svc_left_creator").await;
        let (member_user, _) = create_test_user("find_active_svc_left_member").await;
        
        // Create group chat with creator membership
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_active_svc_left_group", creator.id).await;
        
        // Add member to group
        let membership = add_test_user_to_a_group(member_user.id, &group_chat).await;

        // Leave the group
        let leave_dto = ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto {
            id: membership.id,
        };
        service.leave_group(leave_dto, member_user.id).await.unwrap();

        // Try to find active membership for left user
        let result = service.find_active_by_user_id_and_group_id(member_user.id, group_chat.id).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)
        ));

        // Cleanup
        cleanup_test_user_from_a_group_chat(creator.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(member_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(creator.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_nonexistent_ids() {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let service = service_init.group_membership_service();

        // Test with completely nonexistent IDs
        let result = service.find_active_by_user_id_and_group_id(-999, -888).await;
        
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)
        ));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_active_by_user_id_and_group_id_invitation_not_accepted() {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let service = service_init.group_membership_service();

        let (creator, _) = create_test_user("find_active_svc_not_accepted_creator").await;
        let (member_user, _) = create_test_user("find_active_svc_not_accepted_member").await;
        
        // Create group chat with creator membership
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_active_svc_not_accepted_group", creator.id).await;
        
        // Create invitation but do NOT create membership
        let invitation = crate::create_test_invitation(creator.id, member_user.id, group_chat.id).await;

        // Try to find membership - should not exist
        let result = service.find_active_by_user_id_and_group_id(member_user.id, group_chat.id).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)
        ));

        // Cleanup
        cleanup_test_user_from_a_group_chat(creator.id, group_chat.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email.clone()).await;
        cleanup_user_by_email(creator.email.clone()).await;
    }
}
