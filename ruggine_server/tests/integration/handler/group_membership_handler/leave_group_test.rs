use ruggine_server::handler::group_membership_handler::leave_group::leave_group;
use ruggine_server::state::group_membership_state::GroupMembershipState;
use ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto;
use ruggine_server::error::{api_error::ApiError, group_membership_error::GroupMembershipError};
use ruggine_server::entity::group_membership::MembershipStatus;
use axum::{extract::State, Extension};
use crate::common::{cleanup_user_by_id, cleanup_group_chat, cleanup_invitation, cleanup_group_membership,
                   create_test_user, create_test_group_chat, create_test_invitation, create_test_group_membership};
use crate::get_database;

#[cfg(test)]
mod leave_group_handler_integration_tests {
    use ruggine_server::error::request_error::ValidatedRequest;
    use super::*;

    /// Helper function to create a real group membership state with database connections
    async fn create_group_membership_state() -> GroupMembershipState {
        let db = get_database().await;
        GroupMembershipState::new(&db)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_handler_successfully_leaves_group() {
        // Arrange: Create users, group chat, invitation, and membership
        let (admin_user, _password) = create_test_user("leave_group_admin").await;
        let (member_user, _password) = create_test_user("leave_group_member").await;
        let group_chat = create_test_group_chat("leave_group_test", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        let leave_dto = LeaveGroupMembershipDto { id: membership.id };
        
        // Act: Call leave_group handler
        let result = leave_group(
            Extension(member_user.clone()),
            State(group_membership_state),
            ValidatedRequest(leave_dto),
        ).await;

        // Assert: Should succeed and return updated membership with left status
        assert!(result.is_ok(), "Leave group should succeed when user owns the membership");
        let membership_response = result.unwrap().0;
        
        assert_eq!(membership_response.data().id, membership.id);
        assert_eq!(membership_response.data().user_id, member_user.id);
        assert_eq!(membership_response.data().group_chat_id, group_chat.id);
        assert_eq!(membership_response.data().membership_status, MembershipStatus::Left);
        assert!(membership_response.data().left_at.is_some());

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_id(admin_user.id).await;
        cleanup_user_by_id(member_user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_handler_fails_when_membership_not_found() {
        // Arrange: Create a user but no membership
        let (member_user, _password) = create_test_user("leave_group_no_membership").await;
        let group_membership_state = create_group_membership_state().await;
        let leave_dto = LeaveGroupMembershipDto { id: 999999 }; // Non-existent ID
        
        // Act: Call leave_group handler
        let result = leave_group(
            Extension(member_user.clone()),
            State(group_membership_state),
            ValidatedRequest(leave_dto),
        ).await;

        // Assert: Should fail with GroupMembershipNotFound error
        assert!(result.is_err(), "Leave group should fail when membership doesn't exist");
        
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            },
            other => panic!("Expected GroupMembershipNotFound error, got: {:?}", other),
        }

        // Cleanup
        cleanup_user_by_id(member_user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_handler_fails_when_user_not_authorized() {
        // Arrange: Create users, group chat, invitation, and membership for one user
        let (admin_user, _password) = create_test_user("leave_group_admin_unauth").await;
        let (member_user, _password) = create_test_user("leave_group_member_unauth").await;
        let (unauthorized_user, _password) = create_test_user("leave_group_unauthorized").await;
        
        let group_chat = create_test_group_chat("leave_group_unauth", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        let leave_dto = LeaveGroupMembershipDto { id: membership.id };
        
        // Act: Call leave_group handler with unauthorized user
        let result = leave_group(
            Extension(unauthorized_user.clone()),
            State(group_membership_state),
            ValidatedRequest(leave_dto),
        ).await;

        // Assert: Should fail with GroupMembershipNotFound error (security)
        assert!(result.is_err(), "Leave group should fail when user is not authorized");
        
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error (for security, we return not found instead of unauthorized)
            },
            other => panic!("Expected GroupMembershipNotFound error, got: {:?}", other),
        }

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_id(admin_user.id).await;
        cleanup_user_by_id(member_user.id).await;
        cleanup_user_by_id(unauthorized_user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_handler_fails_when_user_already_left() {
        // Arrange: Create users, group chat, invitation, and membership
        let (admin_user, _password) = create_test_user("leave_group_admin_already").await;
        let (member_user, _password) = create_test_user("leave_group_member_already").await;
        let group_chat = create_test_group_chat("leave_group_already", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        let leave_dto = LeaveGroupMembershipDto { id: membership.id };
        
        // Act: Leave group first time (should succeed)
        let first_result = leave_group(
            Extension(member_user.clone()),
            State(group_membership_state.clone()),
            ValidatedRequest(leave_dto.clone()),
        ).await;
        
        assert!(first_result.is_ok(), "First leave should succeed");
        
        // Act: Try to leave group second time (should fail)
        let second_result = leave_group(
            Extension(member_user.clone()),
            State(group_membership_state),
            ValidatedRequest(leave_dto),
        ).await;

        // Assert: Should fail with UserAlreadyLeftGroup error
        assert!(second_result.is_err(), "Second leave should fail when user already left");
        
        match second_result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyLeftGroup) => {
                // Expected error
            },
            other => panic!("Expected UserAlreadyLeftGroup error, got: {:?}", other),
        }

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_id(admin_user.id).await;
        cleanup_user_by_id(member_user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_handler_fails_with_invalid_membership_id() {
        // Arrange: Create a user
        let (member_user, _password) = create_test_user("leave_group_invalid_id").await;
        let group_membership_state = create_group_membership_state().await;
        let leave_dto = LeaveGroupMembershipDto { id: -1 }; // Invalid ID (should fail validation)
        
        // Act: Call leave_group handler
        let result = leave_group(
            Extension(member_user.clone()),
            State(group_membership_state),
            ValidatedRequest(leave_dto),
        ).await;

        // Assert: Should fail with validation error
        assert!(result.is_err(), "Leave group should fail with invalid membership ID");

        // Cleanup
        cleanup_user_by_id(member_user.id).await;
    }
}
