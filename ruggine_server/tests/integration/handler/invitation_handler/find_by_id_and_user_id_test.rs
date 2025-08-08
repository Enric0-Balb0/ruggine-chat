use ruggine_server::handler::invitation_handler::find_by_id_and_user_id::find_by_id_and_user_id;
use ruggine_server::state::invitation_state::InvitationState;
use ruggine_server::error::{api_error::ApiError, invitation_error::InvitationError};
use axum::{extract::{Path, State}, Extension};
use crate::common::{cleanup_user, cleanup_group_chat, cleanup_invitation, create_test_user, create_test_group_chat, create_test_invitation};
use crate::get_database;

#[cfg(test)]
mod find_by_id_handler_integration_tests {
    use ruggine_server::dto::invitation_dto::InvitationReadDto;
    use crate::create_invitation_state;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_returns_invitation_successfully() {
        // Arrange: Create users, group chat and invitation
        let (admin_user, _password) = create_test_user("find_by_id_and_user_id_success_admin").await;
        let group_chat = create_test_group_chat("find_by_id_and_user_id_success_group", admin_user.id).await;
        let (target_user, _password) = create_test_user("find_by_id_and_user_id_success_target").await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        let current_user = admin_user.clone();
        
        // Act: Call find_by_id handler
        let result = find_by_id_and_user_id(
            Extension(current_user),
            State(invitation_state),
            Path(invitation.id),
        ).await;

        // Assert: Should succeed and return invitation data
        assert!(result.is_ok(), "Find by ID should succeed with valid invitation ID");
        let invitation_response = result.unwrap().0;
        let data = invitation_response.data();

        assert_eq!(*data, InvitationReadDto::from(invitation.clone()));

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_fails_when_invitation_not_found() {
        // Arrange: Create user and invitation state
        let (current_user, _password) = create_test_user("find_by_id_and_user_id_not_found_user").await;
        let invitation_state = create_invitation_state().await;
        let non_existent_id = -1;
        
        // Act: Call find_by_id handler with non-existent ID
        let result = find_by_id_and_user_id(
            Extension(current_user.clone()),
            State(invitation_state),
            Path(non_existent_id),
        ).await;

        // Assert: Should fail with InvitationNotFound error
        assert!(result.is_err(), "Find by ID should fail when invitation not found");
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationNotFound) => {
                // Expected error
            }
            other => panic!("Expected InvitationError::InvitationNotFound, got {:?}", other),
        }

        // Cleanup
        cleanup_user(current_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_returns_different_invitations() {
        // Arrange: Create users, group chats and invitations
        let (admin_user1, _password) = create_test_user("find_by_id_and_user_id_different_admin1").await;
        let group_chat1 = create_test_group_chat("find_by_id_and_user_id_different_group1", admin_user1.id).await;
        let (target_user1, _password) = create_test_user("find_by_id_and_user_id_different_target1").await;
        let invitation1 = create_test_invitation(admin_user1.id, target_user1.id, group_chat1.id).await;
        
        let (admin_user2, _password) = create_test_user("find_by_id_and_user_id_different_admin2").await;
        let group_chat2 = create_test_group_chat("find_by_id_and_user_id_different_group2", admin_user2.id).await;
        let (target_user2, _password) = create_test_user("find_by_id_and_user_id_different_target2").await;
        let invitation2 = create_test_invitation(admin_user2.id, target_user2.id, group_chat2.id).await;
        
        let invitation_state = create_invitation_state().await;
        let current_user1 = admin_user1.clone();
        let current_user2 = admin_user2.clone();
        
        // Act: Find first invitation
        let result1 = find_by_id_and_user_id(
            Extension(current_user1.clone()),
            State(invitation_state.clone()),
            Path(invitation1.id),
        ).await;
        
        // Act: Find second invitation
        let result2 = find_by_id_and_user_id(
            Extension(current_user2),
            State(invitation_state),
            Path(invitation2.id),
        ).await;

        // Assert: Both should succeed with correct data
        assert!(result1.is_ok(), "First find should succeed");
        assert!(result2.is_ok(), "Second find should succeed");
        
        let invitation1_response = result1.unwrap().0;
        let invitation2_response = result2.unwrap().0;

        let data1 = invitation1_response.data();
        assert_eq!(*data1, InvitationReadDto::from(invitation1.clone()));

        let data2 = invitation2_response.data();
        assert_eq!(*data2, InvitationReadDto::from(invitation2.clone()));
        
        assert_ne!(invitation1_response.data().id, invitation2_response.data().id);

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_user(admin_user1.email).await;
        cleanup_user(target_user1.email).await;
        cleanup_user(admin_user2.email).await;
        cleanup_user(target_user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_dont_works_with_any_authenticated_user() {
        // Arrange: Create users, group chat and invitation
        let (admin_user, _password) = create_test_user("find_by_id_and_user_id_any_user_admin").await;
        let group_chat = create_test_group_chat("find_by_id_and_user_id_any_user_group", admin_user.id).await;
        let (target_user, _password) = create_test_user("find_by_id_and_user_id_any_user_target").await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;
        
        let (random_user, _password) = create_test_user("find_by_id_and_user_id_any_user_random").await;
        
        let invitation_state = create_invitation_state().await;
        
        // Act: Call find_by_id handler with random authenticated user
        let result = find_by_id_and_user_id(
            Extension(random_user.clone()),
            State(invitation_state),
            Path(invitation.id),
        ).await;

        // Assert: Should succeed (no authorization check on find_by_id)
        assert!(result.is_err(), "Find by ID should not succeed for any authenticated user");
        assert!(
            matches!(result, Err(ApiError::InvitationError(InvitationError::InvitationNotFound))),
            "Expected InvitationError::NotFound"
        );

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
        cleanup_user(random_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_handler_returns_invitation_with_specific_data() {
        // Arrange: Create users, group chat and invitation with specific data
        let (admin_user, _password) = create_test_user("find_by_id_and_user_id_specific_admin").await;
        let group_chat = create_test_group_chat("find_by_id_and_user_id_specific_group", admin_user.id).await;
        let (target_user, _password) = create_test_user("find_by_id_and_user_id_specific_target").await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;
        
        let invitation_state = create_invitation_state().await;
        let current_user = target_user.clone(); // Use target user as current user
        
        // Act: Call find_by_id handler
        let result = find_by_id_and_user_id(
            Extension(current_user),
            State(invitation_state),
            Path(invitation.id),
        ).await;

        // Assert: Should succeed and return correct specific data
        assert!(result.is_ok(), "Find by ID should succeed");
        let invitation_response = result.unwrap().0;
        
        // Verify all fields match exactly
        let data = invitation_response.data();
        assert_eq!(*data, InvitationReadDto::from(invitation.clone()));
        
        // Verify the invitation is for the correct target user
        assert_eq!(invitation_response.data().to_user_id, target_user.id);
        assert_ne!(invitation_response.data().from_user_id, target_user.id);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }
}
