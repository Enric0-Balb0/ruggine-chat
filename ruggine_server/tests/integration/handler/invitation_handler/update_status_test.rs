use ruggine_server::handler::invitation_handler::update_status::update_status;
use ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto;
use ruggine_server::state::invitation_state::InvitationState;
use ruggine_server::error::{api_error::ApiError, request_error::ValidatedRequest, invitation_error::InvitationError};
use ruggine_server::entity::invitation::InvitationStatus;
use axum::{extract::State, Extension};
use crate::common::{cleanup_user, cleanup_group_chat, cleanup_invitation, cleanup_group_membership_by_invitation_id, 
                   create_test_user, create_test_group_chat, create_test_invitation};
use crate::get_database;

#[cfg(test)]
mod update_status_handler_integration_tests {
    use super::*;

    /// Helper function to create a real invitation state with database connections
    async fn create_invitation_state() -> InvitationState {
        let db = get_database().await;
        InvitationState::new(&db)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_handler_accepts_invitation_successfully() {
        // Arrange: Create users and invitation
        let (sender_user, _password) = create_test_user("update_status_sender").await;
        let (recipient_user, _password) = create_test_user("update_status_recipient").await;
        let group_chat = create_test_group_chat("update_status_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        let update_dto = InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };
        
        // Act: Call update_status handler
        let result = update_status(
            Extension(recipient_user.clone()),
            State(invitation_state),
            ValidatedRequest(update_dto),
        ).await;

        // Assert: Should succeed and return accepted status
        assert!(result.is_ok(), "Update status should succeed when recipient accepts invitation");
        let response = result.unwrap().0;
        
        assert_eq!(response.data().id, invitation.id);
        assert_eq!(response.data().status, InvitationStatus::Accepted);
        assert!(response.data().responded_at <= chrono::Utc::now(), "Responded date should not be in future");
        assert!(response.data().group_membership_id.is_some(), "Group membership ID should be set when accepted");

        // Cleanup
        cleanup_group_membership_by_invitation_id(invitation.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_handler_rejects_invitation_successfully() {
        // Arrange: Create users and invitation
        let (sender_user, _password) = create_test_user("update_reject_sender").await;
        let (recipient_user, _password) = create_test_user("update_reject_recipient").await;
        let group_chat = create_test_group_chat("update_reject_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        let update_dto = InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Rejected,
        };
        
        // Act: Call update_status handler
        let result = update_status(
            Extension(recipient_user.clone()),
            State(invitation_state),
            ValidatedRequest(update_dto),
        ).await;

        // Assert: Should succeed and return rejected status
        assert!(result.is_ok(), "Update status should succeed when recipient rejects invitation");
        let response = result.unwrap().0;
        
        assert_eq!(response.data().id, invitation.id);
        assert_eq!(response.data().status, InvitationStatus::Rejected);
        assert!(response.data().responded_at <= chrono::Utc::now(), "Responded date should not be in future");
        assert!(response.data().group_membership_id.is_none(), "Group membership ID should not be set when rejected");

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_handler_fails_when_user_not_recipient() {
        // Arrange: Create users and invitation
        let (sender_user, _password) = create_test_user("update_wrong_sender").await;
        let (recipient_user, _password) = create_test_user("update_wrong_recipient").await;
        let (wrong_user, _password) = create_test_user("update_wrong_user").await;
        let group_chat = create_test_group_chat("update_wrong_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        let update_dto = InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };
        
        // Act: Call update_status handler with wrong user
        let result = update_status(
            Extension(wrong_user.clone()),
            State(invitation_state),
            ValidatedRequest(update_dto),
        ).await;

        // Assert: Should fail with UserNotAuthorized error
        assert!(result.is_err(), "Update status should fail when user is not the recipient");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized) => {
                // Expected error
            }
            _ => panic!("Expected UserNotAuthorized error"),
        }

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(wrong_user.email).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_handler_fails_when_invitation_not_found() {
        // Arrange: Create user with non-existent invitation ID
        let (user, _password) = create_test_user("update_not_found_user").await;
        
        let invitation_state = create_invitation_state().await;
        let update_dto = InvitationUpdateStatusDto {
            invitation_id: 99999, // Non-existent ID
            status: InvitationStatus::Accepted,
        };
        
        // Act: Call update_status handler
        let result = update_status(
            Extension(user.clone()),
            State(invitation_state),
            ValidatedRequest(update_dto),
        ).await;

        // Assert: Should fail with InvitationNotFound error
        assert!(result.is_err(), "Update status should fail when invitation doesn't exist");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationNotFound) => {
                // Expected error
            }
            _ => panic!("Expected InvitationNotFound error"),
        }

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_handler_fails_when_invitation_already_responded() {
        // Arrange: Create users and invitation, then accept it first
        let (sender_user, _password) = create_test_user("update_already_sender").await;
        let (recipient_user, _password) = create_test_user("update_already_recipient").await;
        let group_chat = create_test_group_chat("update_already_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        let first_update_dto = InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };

        // First update (should succeed)
        let first_result = update_status(
            Extension(recipient_user.clone()),
            State(invitation_state.clone()),
            ValidatedRequest(first_update_dto),
        ).await;
        assert!(first_result.is_ok(), "First update should succeed");

        // Attempt second update
        let second_update_dto = InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Rejected,
        };
        
        // Act: Call update_status handler again
        let result = update_status(
            Extension(recipient_user.clone()),
            State(invitation_state),
            ValidatedRequest(second_update_dto),
        ).await;

        // Assert: Should fail with InvitationAlreadyResponded error
        assert!(result.is_err(), "Update status should fail when invitation already responded");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationAlreadyResponded) => {
                // Expected error
            }
            _ => panic!("Expected InvitationAlreadyResponded error"),
        }

        // Cleanup
        cleanup_group_membership_by_invitation_id(invitation.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_handler_fails_with_pending_status() {
        // Arrange: Create users and invitation
        let (sender_user, _password) = create_test_user("update_pending_sender").await;
        let (recipient_user, _password) = create_test_user("update_pending_recipient").await;
        let group_chat = create_test_group_chat("update_pending_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        let update_dto = InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Pending,
        };
        
        // Act: Call update_status handler with pending status
        let result = update_status(
            Extension(recipient_user.clone()),
            State(invitation_state),
            ValidatedRequest(update_dto),
        ).await;

        // Assert: Should fail with InvalidStatus error
        assert!(result.is_err(), "Update status should fail when trying to set status to pending");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvalidStatus(_)) => {
                // Expected error
            }
            _ => panic!("Expected InvalidStatus error"),
        }

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }
}
