use ruggine_server::handler::invitation_handler::send::send;
use ruggine_server::dto::invitation_dto::InvitationCreateDto;
use ruggine_server::state::invitation_state::InvitationState;
use ruggine_server::error::{api_error::ApiError, request_error::ValidatedRequest, invitation_error::InvitationError, group_chat_error::GroupChatError, user_error::UserError};
use axum::{extract::State, Extension};
use crate::common::{cleanup_user, cleanup_group_chat, cleanup_invitation, create_test_user, create_test_group_chat};
use crate::get_database;

#[cfg(test)]
mod send_handler_integration_tests {
    use ruggine_server::entity::group_membership::MemberRole;
    use crate::{clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id, create_test_group_chat_with_invitation_and_membership};
    use super::*;

    /// Helper function to create a real invitation state with database connections
    async fn create_invitation_state() -> InvitationState {
        let db = get_database().await;
        InvitationState::new(&db)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_handler_creates_invitation_successfully_as_admin() {
        // Arrange: Create admin user and group chat
        let (admin_user, _password) = create_test_user("send_success_admin").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("send_success_group", admin_user.id).await;
        let (target_user, _password) = create_test_user("send_success_target").await;
        
        let invitation_state = create_invitation_state().await;
        let invitation_create_dto = InvitationCreateDto {
            to_user_id: target_user.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Admin,
        };
        
        // Act: Call send handler
        let result = send(
            Extension(admin_user.clone()),
            State(invitation_state),
            ValidatedRequest(invitation_create_dto.clone()),
        ).await;

        // Assert: Should succeed and return invitation data
        assert!(result.is_ok(), "Send invitation should succeed when user is admin");
        let invitation_response = result.unwrap().0;
        
        assert_eq!(invitation_response.data().from_user_id, admin_user.id);
        assert_eq!(invitation_response.data().to_user_id, target_user.id);
        assert_eq!(invitation_response.data().group_chat_id, group_chat.id);
        assert_eq!(invitation_response.data().status, ruggine_server::entity::invitation::InvitationStatus::Pending);
        assert!(invitation_response.data().sent_at <= chrono::Utc::now(), "Sent date should not be in future");

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_invitation(invitation_response.data().id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_handler_fails_when_user_not_admin() {
        // Arrange: Create admin user, group chat, and non-admin user
        let (admin_user, _password) = create_test_user("send_not_admin_admin").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("send_not_admin_group", admin_user.id).await;
        let (non_admin_user, _password) = create_test_user("send_not_admin_user").await;
        let (target_user, _password) = create_test_user("send_not_admin_target").await;
        
        let invitation_state = create_invitation_state().await;
        let invitation_create_dto = InvitationCreateDto {
            to_user_id: target_user.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Admin
        };
        
        // Act: Call send handler with non-admin user
        let result = send(
            Extension(non_admin_user.clone()),
            State(invitation_state),
            ValidatedRequest(invitation_create_dto),
        ).await;

        // Assert: Should fail with UserNotAuthorized error
        assert!(result.is_err(), "Send invitation should fail when user is not admin");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized(_)) => {
                // Expected error
            }
            other => panic!("Expected GroupChatError::UserNotAuthorized, got {:?}", other),
        }

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(non_admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_handler_fails_when_invitation_already_exists() {
        // Arrange: Create admin user, group chat, and target user
        let (admin_user, _password) = create_test_user("send_duplicate_admin").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("send_duplicate_group", admin_user.id).await;
        let (target_user, _password) = create_test_user("send_duplicate_target").await;
        
        let invitation_state = create_invitation_state().await;
        let invitation_create_dto = InvitationCreateDto {
            to_user_id: target_user.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Admin
        };
        
        // Send invitation first time
        let first_result = send(
            Extension(admin_user.clone()),
            State(invitation_state.clone()),
            ValidatedRequest(invitation_create_dto.clone()),
        ).await;
        assert!(first_result.is_ok(), "First send should succeed");

        // Act: Try to send the same invitation again
        let result = send(
            Extension(admin_user.clone()),
            State(invitation_state),
            ValidatedRequest(invitation_create_dto),
        ).await;

        // Assert: Should fail with InvitationAlreadyExists error
        assert!(result.is_err(), "Second send should fail");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::AlreadyInvitationPending) => {
                // Expected error
            }
            other => panic!("Expected InvitationError::InvitationAlreadyExists, got {:?}", other),
        }

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_invitation(first_result.unwrap().0.data().id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_handler_fails_when_target_user_not_found() {
        // Arrange: Create admin user and group chat
        let (admin_user, _password) = create_test_user("send_user_not_found_admin").await;
        let group_chat = create_test_group_chat("send_user_not_found_group", admin_user.id).await;
        
        let invitation_state = create_invitation_state().await;
        let invitation_create_dto = InvitationCreateDto {
            to_user_id: -1, // Non-existent user ID
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Admin
        };
        
        // Act: Call send handler with non-existent target user
        let result = send(
            Extension(admin_user.clone()),
            State(invitation_state),
            ValidatedRequest(invitation_create_dto),
        ).await;

        // Assert: Should fail with UserNotFound error
        assert!(result.is_err(), "Send invitation should fail when target user not found");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitedUserNotFound) => {
                // Expected error
            }
            other => panic!("Expected InvitedUserNotFound, got {:?}", other),
        }

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_handler_fails_when_group_chat_not_found() {
        // Arrange: Create admin user and target user
        let (admin_user, _password) = create_test_user("send_group_not_found_admin").await;
        let (target_user, _password) = create_test_user("send_group_not_found_target").await;
        
        let invitation_state = create_invitation_state().await;
        let invitation_create_dto = InvitationCreateDto {
            to_user_id: target_user.id,
            group_chat_id: -1, // Non-existent group chat ID
            role_at_join: MemberRole::Admin,
        };
        
        // Act: Call send handler with non-existent group chat
        let result = send(
            Extension(admin_user.clone()),
            State(invitation_state),
            ValidatedRequest(invitation_create_dto),
        ).await;

        // Assert: Should fail with GroupChatNotFound error
        assert!(result.is_err(), "Send invitation should fail when group chat not found");
        match result.unwrap_err() {
            ApiError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                // Expected error
            }
            other => panic!("Expected GroupChatError::GroupChatNotFound, got {:?}", other),
        }

        // Cleanup
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_handler_creates_invitation_with_different_users() {
        // Arrange: Create admin user, group chat, and different target users
        let (admin_user, _password) = create_test_user("send_different_admin").await;
        let group_chat = create_test_group_chat_with_invitation_and_membership("send_different_group", admin_user.id).await;
        let (target_user1, _password) = create_test_user("send_different_target1").await;
        let (target_user2, _password) = create_test_user("send_different_target2").await;
        
        let invitation_state = create_invitation_state().await;
        
        // Send first invitation
        let invitation_create_dto1 = InvitationCreateDto {
            to_user_id: target_user1.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Member
        };
        
        let result1 = send(
            Extension(admin_user.clone()),
            State(invitation_state.clone()),
            ValidatedRequest(invitation_create_dto1),
        ).await;
        
        // Send second invitation to different user
        let invitation_create_dto2 = InvitationCreateDto {
            to_user_id: target_user2.id,
            group_chat_id: group_chat.id,
            role_at_join: MemberRole::Member,
        };
        
        let result2 = send(
            Extension(admin_user.clone()),
            State(invitation_state),
            ValidatedRequest(invitation_create_dto2),
        ).await;

        // Assert: Both should succeed
        assert!(result1.is_ok(), "First invitation should succeed");
        assert!(result2.is_ok(), "Second invitation should succeed");
        
        let invitation1_response = result1.unwrap().0;
        let invitation2_response = result2.unwrap().0;

        assert_eq!(invitation1_response.data().to_user_id, target_user1.id);
        assert_eq!(invitation2_response.data().to_user_id, target_user2.id);
        assert_eq!(invitation1_response.data().from_user_id, admin_user.id);
        assert_eq!(invitation2_response.data().from_user_id, admin_user.id);

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_user.id, group_chat.id).await;
        cleanup_invitation(invitation1_response.data().id).await;
        cleanup_invitation(invitation2_response.data().id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user1.email).await;
        cleanup_user(target_user2.email).await;
    }
}
