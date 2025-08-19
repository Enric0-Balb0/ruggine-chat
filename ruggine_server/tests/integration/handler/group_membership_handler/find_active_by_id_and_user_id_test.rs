use ruggine_server::handler::group_membership_handler::find_active_by_id_and_user_id::find_active_by_id_and_user_id;
use ruggine_server::state::group_membership_state::GroupMembershipState;
use ruggine_server::error::{api_error::ApiError, group_membership_error::GroupMembershipError};
use axum::{extract::{Path, State}, Extension};
use crate::common::{cleanup_user_by_email, cleanup_group_chat, cleanup_invitation, cleanup_group_membership,
                    create_test_user, create_test_group_chat, create_test_invitation, create_test_group_membership,
                    create_group_membership_state};
use crate::get_database;

#[cfg(test)]
mod find_active_by_id_and_user_id_handler_integration_tests {
    use crate::create_test_admin_group_membership;

    use super::*;


    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_finds_membership_successfully() {
        // Arrange: Create users, group chat, invitation, and membership
        let (admin_user, _password) = create_test_user("find_membership_admin").await;
        let (member_user, _password) = create_test_user("find_membership_member").await;
        let group_chat = create_test_group_chat("find_membership_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_id_and_user_id handler
        let result = find_active_by_id_and_user_id(
            Extension(member_user.clone()),
            State(group_membership_state),
            Path(membership.id),
        ).await;

        // Assert: Should succeed and return membership data
        assert!(result.is_ok(), "Find membership should succeed when user owns the membership");
        let membership_response = result.unwrap().0;
        
        assert_eq!(membership_response.data().id, membership.id);
        assert_eq!(membership_response.data().user_id, member_user.id);
        assert_eq!(membership_response.data().group_chat_id, group_chat.id);
        assert_eq!(membership_response.data().invitation_id, invitation.id);

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email).await;
        cleanup_user_by_email(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_fails_when_wrong_user() {
        // Arrange: Create users, group chat, invitation, and membership
        let (admin_user, _password) = create_test_user("find_wrong_admin").await;
        let (member_user, _password) = create_test_user("find_wrong_member").await;
        let (wrong_user, _password) = create_test_user("find_wrong_user").await;
        let group_chat = create_test_group_chat("find_wrong_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_id_and_user_id handler with wrong user
        let result = find_active_by_id_and_user_id(
            Extension(wrong_user.clone()),
            State(group_membership_state),
            Path(membership.id),
        ).await;

        // Assert: Should fail with GroupMembershipNotFound error
        assert!(result.is_err(), "Find membership should fail when wrong user tries to access");
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error"),
        }

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(wrong_user.email).await;
        cleanup_user_by_email(member_user.email).await;
        cleanup_user_by_email(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_fails_when_membership_not_found() {
        // Arrange: Create user with non-existent membership ID
        let (user, _password) = create_test_user("find_not_found_user").await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_id_and_user_id handler with non-existent ID
        let result = find_active_by_id_and_user_id(
            Extension(user.clone()),
            State(group_membership_state),
            Path(99999), // Non-existent ID
        ).await;

        // Assert: Should fail with GroupMembershipNotFound error
        assert!(result.is_err(), "Find membership should fail when membership doesn't exist");
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error"),
        }

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_finds_admin_membership() {
        // Arrange: Create users, group chat, invitation, and admin membership
        let (admin_user, _password) = create_test_user("find_admin_membership_admin").await;
        let (member_user, _password) = create_test_user("find_admin_membership_member").await;
        let group_chat = create_test_group_chat("find_admin_membership_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let admin_membership = create_test_admin_group_membership(invitation.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_id_and_user_id handler
        let result = find_active_by_id_and_user_id(
            Extension(member_user.clone()),
            State(group_membership_state),
            Path(admin_membership.id),
        ).await;

        // Assert: Should succeed and return admin membership data
        assert!(result.is_ok(), "Find admin membership should succeed when user owns the membership");
        let membership_response = result.unwrap().0;
        
        assert_eq!(membership_response.data().id, admin_membership.id);
        assert_eq!(membership_response.data().user_id, member_user.id);
        assert_eq!(membership_response.data().group_chat_id, group_chat.id);
        assert_eq!(membership_response.data().invitation_id, invitation.id);
        assert_eq!(membership_response.data().role, ruggine_server::entity::group_membership::MemberRole::Admin);

        // Cleanup
        cleanup_group_membership(admin_membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email).await;
        cleanup_user_by_email(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_returns_correct_membership_fields() {
        // Arrange: Create users, group chat, invitation, and membership
        let (admin_user, _password) = create_test_user("find_fields_admin").await;
        let (member_user, _password) = create_test_user("find_fields_member").await;
        let group_chat = create_test_group_chat("find_fields_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_id_and_user_id handler
        let result = find_active_by_id_and_user_id(
            Extension(member_user.clone()),
            State(group_membership_state),
            Path(membership.id),
        ).await;

        // Assert: Should succeed and return membership with all expected fields
        assert!(result.is_ok(), "Find membership should succeed");
        let membership_response = result.unwrap().0;
        let data = membership_response.data();
        
        assert_eq!(data.id, membership.id);
        assert_eq!(data.user_id, member_user.id);
        assert_eq!(data.group_chat_id, group_chat.id);
        assert_eq!(data.invitation_id, invitation.id);
        assert_eq!(data.role, ruggine_server::entity::group_membership::MemberRole::Member);
        assert_eq!(data.membership_status, ruggine_server::entity::group_membership::MembershipStatus::Active);
        assert!(data.joined_at <= chrono::Utc::now(), "Joined date should not be in future");
        assert!(data.left_at.is_none(), "Left date should be None for active membership");

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(member_user.email).await;
        cleanup_user_by_email(admin_user.email).await;
    }
}
