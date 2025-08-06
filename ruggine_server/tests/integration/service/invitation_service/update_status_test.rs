/*use ruggine_server::service::invitation_service::{InvitationService, InvitationServiceTrait};
use ruggine_server::factory::invitation_factory::InvitationFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::invitation_error::InvitationError;
use ruggine_server::entity::invitation::InvitationStatus;
use crate::common::{
    get_database, create_test_user, cleanup_user, cleanup_group_chat, 
    create_test_group_chat, cleanup_invitation, create_test_invitation
};

#[cfg(test)]
mod invitation_service_update_status_integration_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_success_accept() {
        // Arrange: Create real users and group in database
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create admin user and target user
        let (admin_user, _) = create_test_user("update_status_admin").await;
        let (target_user, _) = create_test_user("update_status_target").await;
        
        // Create group chat with admin as creator
        let group_chat = create_test_group_chat("update_status_group", admin_user.id).await;
        
        // Create a pending invitation
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;
        assert_eq!(invitation.status, InvitationStatus::Pending);

        // Create update status DTO
        let update_dto = InvitationFactory::fake_invitation_update_status_dto();
        let mut payload = update_dto.clone();
        payload.invitation_id = invitation.id;
        payload.status = InvitationStatus::Accepted;

        // Act: Update the invitation status
        let result = invitation_service.update_status(payload, target_user.id).await;

        // Assert: Verify invitation was updated successfully
        assert!(result.is_ok(), "Failed to update invitation status: {:?}", result);
        let response = result.unwrap();
        
        assert_eq!(response.id, invitation.id);
        assert_eq!(response.status, InvitationStatus::Accepted);
        assert!(response.responded_at <= chrono::Utc::now());

        // Verify that a group membership was created TODO
        let membership_result = sqlx::query!(
            "SELECT id, user_id, group_chat_id, role FROM group_membership WHERE user_id = $1 AND group_chat_id = $2",
            target_user.id,
            group_chat.id
        )
        .fetch_optional(db.get_pool())
        .await
        .expect("Failed to query group membership");

        assert!(membership_result.is_some(), "Group membership should have been created");
        let membership = membership_result.unwrap();
        assert_eq!(membership.user_id, target_user.id);
        assert_eq!(membership.group_chat_id, group_chat.id);

        // Cleanup: Delete the test data
        if let Some(membership_id) = membership.id {
            sqlx::query!("DELETE FROM group_membership WHERE id = $1", membership_id)
                .execute(db.get_pool())
                .await
                .expect("Failed to cleanup group membership");
        }
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_success_reject() {
        // Arrange: Create real users and group in database
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create admin user and target user
        let (admin_user, _) = create_test_user("update_status_reject_admin").await;
        let (target_user, _) = create_test_user("update_status_reject_target").await;
        
        // Create group chat with admin as creator
        let group_chat = create_test_group_chat("update_status_reject_group", admin_user.id).await;
        
        // Create a pending invitation
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;
        assert_eq!(invitation.status, InvitationStatus::Pending);

        // Create update status DTO
        let update_dto = InvitationFactory::fake_invitation_update_status_dto_rejected();
        let mut payload = update_dto.clone();
        payload.invitation_id = invitation.id;
        payload.status = InvitationStatus::Rejected;

        // Act: Update the invitation status
        let result = invitation_service.update_status(payload, target_user.id).await;

        // Assert: Verify invitation was updated successfully
        assert!(result.is_ok(), "Failed to update invitation status: {:?}", result);
        let response = result.unwrap();
        
        assert_eq!(response.id, invitation.id);
        assert_eq!(response.status, InvitationStatus::Rejected);
        assert!(response.responded_at <= chrono::Utc::now());

        // Verify that NO group membership was created
        let membership_result = sqlx::query!(
            "SELECT id FROM group_membership WHERE user_id = $1 AND group_chat_id = $2",
            target_user.id,
            group_chat.id
        )
        .fetch_optional(db.get_pool())
        .await
        .expect("Failed to query group membership");

        assert!(membership_result.is_none(), "Group membership should NOT have been created for rejected invitation");

        // Cleanup: Delete the test data
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_invitation_not_found() {
        // Arrange
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (target_user, _) = create_test_user("update_status_not_found_user").await;
        
        let non_existent_invitation_id = 99999;
        let payload = InvitationFactory::with_status_dto(
            InvitationFactory::fake_invitation_update_status_dto(),
            InvitationStatus::Accepted
        );
        let mut update_payload = payload.clone();
        update_payload.invitation_id = non_existent_invitation_id;

        // Act: Try to update non-existent invitation
        let result = invitation_service.update_status(update_payload, target_user.id).await;

        // Assert: Should return InvitationNotFound error
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationNotFound) => {
                // Success
            },
            e => panic!("Expected InvitationNotFound error, got: {:?}", e),
        }

        // Cleanup
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_user_not_authorized() {
        // Arrange: Create real users and group in database
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create admin user, target user, and unauthorized user
        let (admin_user, _) = create_test_user("update_status_auth_admin").await;
        let (target_user, _) = create_test_user("update_status_auth_target").await;
        let (unauthorized_user, _) = create_test_user("update_status_auth_unauthorized").await;
        
        // Create group chat with admin as creator
        let group_chat = create_test_group_chat("update_status_auth_group", admin_user.id).await;
        
        // Create a pending invitation for target_user
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        // Create update status DTO
        let payload = InvitationFactory::with_status_dto(
            InvitationFactory::fake_invitation_update_status_dto(),
            InvitationStatus::Accepted
        );
        let mut update_payload = payload.clone();
        update_payload.invitation_id = invitation.id;

        // Act: Try to update invitation with unauthorized user
        let result = invitation_service.update_status(update_payload, unauthorized_user.id).await;

        // Assert: Should return UserNotAuthorized error
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::UserNotAuthorized) => {
                // Success
            },
            e => panic!("Expected UserNotAuthorized error, got: {:?}", e),
        }

        // Cleanup: Delete the test data
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
        cleanup_user(unauthorized_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_invitation_already_responded() {
        // Arrange: Create real users and group in database
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create admin user and target user
        let (admin_user, _) = create_test_user("update_status_responded_admin").await;
        let (target_user, _) = create_test_user("update_status_responded_target").await;
        
        // Create group chat with admin as creator
        let group_chat = create_test_group_chat("update_status_responded_group", admin_user.id).await;
        
        // Create a pending invitation
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        // Manually update invitation status to accepted (simulate already responded)
        sqlx::query!(
            "UPDATE invitation SET status = 'accepted', responded_at = NOW() WHERE id = $1",
            invitation.id
        )
        .execute(db.get_pool())
        .await
        .expect("Failed to manually update invitation status");

        // Create update status DTO
        let payload = InvitationFactory::with_status_dto(
            InvitationFactory::fake_invitation_update_status_dto(),
            InvitationStatus::Rejected
        );
        let mut update_payload = payload.clone();
        update_payload.invitation_id = invitation.id;

        // Act: Try to update already responded invitation
        let result = invitation_service.update_status(update_payload, target_user.id).await;

        // Assert: Should return InvitationAlreadyResponded error
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvitationAlreadyResponded) => {
                // Success
            },
            e => panic!("Expected InvitationAlreadyResponded error, got: {:?}", e),
        }

        // Cleanup: Delete the test data
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_invalid_status_pending() {
        // Arrange
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        let (target_user, _) = create_test_user("update_status_invalid_user").await;
        
        // Create update status DTO with invalid status (Pending)
        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = 1; // Doesn't matter since validation happens first
        payload.status = InvitationStatus::Pending; // Invalid status for update

        // Act: Try to update with invalid status
        let result = invitation_service.update_status(payload, target_user.id).await;

        // Assert: Should return InvalidStatus error
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvitationError(InvitationError::InvalidStatus(_)) => {
                // Success
            },
            e => panic!("Expected InvalidStatus error, got: {:?}", e),
        }

        // Cleanup
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_with_correct_timestamp() {
        // Arrange: Create real users and group in database
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);
        
        // Create admin user and target user
        let (admin_user, _) = create_test_user("update_status_timestamp_admin").await;
        let (target_user, _) = create_test_user("update_status_timestamp_target").await;
        
        // Create group chat with admin as creator
        let group_chat = create_test_group_chat("update_status_timestamp_group", admin_user.id).await;
        
        // Create a pending invitation
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;
        let before_update = chrono::Utc::now();

        // Create update status DTO
        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = invitation.id;
        payload.status = InvitationStatus::Accepted;

        // Act: Update the invitation status
        let result = invitation_service.update_status(payload, target_user.id).await;
        let after_update = chrono::Utc::now();

        // Assert: Verify invitation was updated successfully with correct timestamp
        assert!(result.is_ok(), "Failed to update invitation status: {:?}", result);
        let response = result.unwrap();
        
        assert_eq!(response.id, invitation.id);
        assert_eq!(response.status, InvitationStatus::Accepted);
        assert!(response.responded_at >= before_update);
        assert!(response.responded_at <= after_update);

        // Cleanup: Delete the test data
        // First cleanup group membership if created
        sqlx::query!("DELETE FROM group_membership WHERE user_id = $1 AND group_chat_id = $2", target_user.id, group_chat.id)
            .execute(db.get_pool())
            .await
            .expect("Failed to cleanup group membership");
        
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }
}*/
