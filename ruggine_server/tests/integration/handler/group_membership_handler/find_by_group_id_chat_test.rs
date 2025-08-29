
use ruggine_server::handler::group_membership_handler::find_by_group_chat_id::find_by_group_chat_id;
use ruggine_server::state::group_membership_state::GroupMembershipState;
use ruggine_server::error::{api_error::ApiError, group_membership_error::GroupMembershipError};
use axum::{extract::{Path, State}, Extension};
use crate::common::{
    cleanup_user_by_email, cleanup_group_chat, cleanup_invitation, cleanup_group_membership,
    create_test_user, create_test_group_chat, create_test_invitation, create_test_group_membership,
    create_test_admin_group_membership, create_group_membership_state,
};
use crate::{add_test_user_to_a_group, cleanup_test_user_from_a_group_chat, cleanup_test_users, create_test_group_chat_with_invitation_and_membership, get_database, test_user_leave_from_a_group};

#[cfg(test)]
mod find_by_group_chat_id_handler_integration_tests {
    use crate::cleanup_test_users_from_a_group_chat;

    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_handler_finds_memberships_successfully() {
        // Arrange: Create users, group chat, invitations, and memberships
        let (admin_user, _) = create_test_user("find_by_group_chat_id_admin").await;
        let (member_user1, _) = create_test_user("find_by_group_chat_id_member1").await;
        let (member_user2, _) = create_test_user("find_by_group_chat_id_member2").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_by_group_chat_id_group", admin_user.id).await;

        // Create member memberships
        let invitation1 = create_test_invitation(admin_user.id, member_user1.id, group_chat.id).await;
        let invitation2 = create_test_invitation(admin_user.id, member_user2.id, group_chat.id).await;
        let membership1 = create_test_group_membership(invitation1.id, member_user1.id).await;
        let membership2 = create_test_group_membership(invitation2.id, member_user2.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_group_chat_id handler as admin user
        let result = find_by_group_chat_id(
            Extension(admin_user.clone()),
            State(group_membership_state),
            Path(group_chat.id),
        ).await;

        // Assert: Should succeed and return all memberships for the group
        assert!(result.is_ok(), "Find group memberships should succeed when user is a member");
        let memberships_response = result.unwrap().0;
        let memberships = memberships_response.data();
        
        assert_eq!(memberships.len(), 3);
        
        // Verify all memberships belong to the group
        let user_ids: Vec<i32> = memberships.iter().map(|m| m.user_id).collect();
        assert!(user_ids.contains(&admin_user.id));
        assert!(user_ids.contains(&member_user1.id));
        assert!(user_ids.contains(&member_user2.id));
        
        for membership_dto in memberships {
            assert_eq!(membership_dto.group_chat_id, group_chat.id);
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(user_ids, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(member_user1.email).await;
        cleanup_user_by_email(member_user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_handler_fails_when_user_not_member() {
        // Arrange: Create users and group, but don't add non_member to group
        let (admin_user, _) = create_test_user("find_by_group_chat_id_admin_fail").await;
        let (non_member_user, _) = create_test_user("find_by_group_chat_id_non_member").await;
        let group_chat = create_test_group_chat("find_by_group_chat_id_group_fail", admin_user.id).await;
        
        // Create admin membership only
        let admin_invitation = create_test_invitation(admin_user.id, admin_user.id, group_chat.id).await;
        let admin_membership = create_test_admin_group_membership(admin_invitation.id, admin_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_group_chat_id handler as non-member user
        let result = find_by_group_chat_id(
            Extension(non_member_user.clone()),
            State(group_membership_state),
            Path(group_chat.id),
        ).await;

        // Assert: Should fail with GroupMembershipNotFound error
        assert!(result.is_err(), "Find group memberships should fail when user is not a member");
        
        let error = result.unwrap_err();
        match error {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error, got: {:?}", error),
        }

        // Cleanup
        cleanup_group_membership(admin_membership.id).await;
        cleanup_invitation(admin_invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(non_member_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_handler_fails_when_group_not_found() {
        // Arrange: Create user
        let (user, _) = create_test_user("find_by_group_chat_id_no_group").await;
        let group_membership_state = create_group_membership_state().await;
        let non_existent_group_id = 999999;
        
        // Act: Call find_by_group_chat_id handler with non-existent group
        let result = find_by_group_chat_id(
            Extension(user.clone()),
            State(group_membership_state),
            Path(non_existent_group_id),
        ).await;

        // Assert: Should fail with GroupNotFound error
        assert!(result.is_err(), "Find group memberships should fail when group doesn't exist");
        
        let error = result.unwrap_err();
        match error {
            ApiError::GroupMembershipError(GroupMembershipError::GroupNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupNotFound error, got: {:?}", error),
        }

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    
    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_group_not_active_membership() {
        // Arrange
        let (admin_user, _) = create_test_user("find_by_group_chat_id_admin").await;
        let (member_user1, _) = create_test_user("find_by_group_chat_id_member1").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("find_by_group_chat_id_group", admin_user.id).await;
        add_test_user_to_a_group(member_user1.id, &group_chat).await;
        
        test_user_leave_from_a_group(member_user1.id, group_chat.id).await;

        // Act
        let group_membership_state = create_group_membership_state().await;
        let result = find_by_group_chat_id(
                Extension(member_user1.clone()),
                State(group_membership_state),
                Path(group_chat.id),
            ).await;

        // Assert: Should fail with GroupMembershipNotFound error
        assert!(result.is_err(), "Find group memberships should fail when group membership is not anymore active");

        let error = result.unwrap_err();
        match error {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error, got: {:?}", error),
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(member_user1.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_test_users(vec![admin_user.id, member_user1.id]).await
}

}