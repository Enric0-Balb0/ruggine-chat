use ruggine_server::service::invitation_service::{InvitationService};
use ruggine_server::factory::invitation_factory::InvitationFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::invitation_error::InvitationError;
use ruggine_server::entity::invitation::InvitationStatus;
use crate::common::{
    get_database, create_test_user, cleanup_user_by_email, cleanup_group_chat,
    create_test_group_chat, cleanup_invitation, create_test_invitation
};

#[cfg(test)]
mod invitation_service_update_status_integration_tests {
    use ruggine_server::config::database::DatabaseTrait;
    use ruggine_server::entity::invitation::Invitation;
    use ruggine_server::service::invitation_service::InvitationServiceTrait;
    use crate::cleanup_group_membership_by_invitation_id;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_accept_success() {
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);

        let (admin_user, _) = create_test_user("internal_accept_admin").await;
        let (target_user, _) = create_test_user("internal_accept_target").await;
        let group_chat = create_test_group_chat("internal_accept_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = invitation.id;
        payload.status = InvitationStatus::Accepted;

        let result = invitation_service.update_status(payload, target_user.id).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, InvitationStatus::Accepted);
        assert!(response.group_membership_id.is_some());

        cleanup_group_membership_by_invitation_id(invitation.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_reject_success() {
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);

        let (admin_user, _) = create_test_user("internal_reject_admin").await;
        let (target_user, _) = create_test_user("internal_reject_target").await;
        let group_chat = create_test_group_chat("internal_reject_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = invitation.id;
        payload.status = InvitationStatus::Rejected;

        let result = invitation_service.update_status(payload, target_user.id).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, InvitationStatus::Rejected);
        assert!(response.group_membership_id.is_none());

        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_invitation_not_found() {
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);

        let (user, _) = create_test_user("internal_not_found").await;
        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = 999999;
        payload.status = InvitationStatus::Accepted;

        let result = invitation_service.update_status(payload, user.id).await;

        assert!(matches!(
            result.unwrap_err(),
            ApiError::InvitationError(InvitationError::InvitationNotFound)
        ));

        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_user_not_authorized() {
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);

        let (admin_user, _) = create_test_user("internal_auth_admin").await;
        let (target_user, _) = create_test_user("internal_auth_target").await;
        let (unauth_user, _) = create_test_user("internal_auth_unauthorized").await;

        let group_chat = create_test_group_chat("internal_auth_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = invitation.id;
        payload.status = InvitationStatus::Accepted;

        let result = invitation_service.update_status(payload, unauth_user.id).await;

        assert!(matches!(
            result.unwrap_err(),
            ApiError::InvitationError(InvitationError::UserNotAuthorized(_))
        ));

        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
        cleanup_user_by_email(unauth_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_already_responded() {
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);

        let (admin_user, _) = create_test_user("internal_responded_admin").await;
        let (target_user, _) = create_test_user("internal_responded_target").await;
        let group_chat = create_test_group_chat("internal_responded_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        sqlx::query!(
            "UPDATE invitation SET status = 'accepted', responded_at = NOW() WHERE id = $1",
            invitation.id
        )
            .execute(db.get_pool())
            .await
            .unwrap();

        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = invitation.id;
        payload.status = InvitationStatus::Rejected;

        let result = invitation_service.update_status(payload, target_user.id).await;

        assert!(matches!(
            result.unwrap_err(),
            ApiError::InvitationError(InvitationError::InvitationAlreadyResponded)
        ));

        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_invalid_status_pending() {
        let db = get_database().await;
        let invitation_service = InvitationService::new(&db);

        let (user, _) = create_test_user("internal_invalid_status").await;

        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = 1;
        payload.status = InvitationStatus::Pending;

        let result = invitation_service.update_status(payload, user.id).await;

        assert!(matches!(
            result.unwrap_err(),
            ApiError::InvitationError(InvitationError::InvalidStatus(_))
        ));

        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_accept_fails_when_create_checked_fails() {
        // Questo test simula il fallimento di create_checked durante l'accettazione di un invito
        // per testare il rollback della transazione
        use ruggine_server::service::invitation_service::InvitationService;
        use ruggine_server::repository::invitation_repository::InvitationRepository;
        use ruggine_server::service::group_chat_service::GroupChatService;
        use ruggine_server::service::user_service::UserService;
        use ruggine_server::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
        use ruggine_server::error::group_membership_error::GroupMembershipError;
        use std::sync::Arc;
        use mockall::predicate::*;

        let db = get_database().await;
        
        let (admin_user, _) = create_test_user("internal_mock_admin").await;
        let (target_user, _) = create_test_user("internal_mock_target").await;
        let group_chat = create_test_group_chat("internal_mock_group", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        // Creiamo un mock del GroupMembershipService che fallisce
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        
        mock_group_membership_service
            .expect_create_checked()
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move {
                    Err(ApiError::DbError(ruggine_server::error::db_error::DbError::SomethingWentWrong(
                        "Simulated database error during create_checked".to_string()
                    )))
                })
            });

        // Creiamo un InvitationService con il mock
        let invitation_service = InvitationService::with(
            Arc::new(InvitationRepository::new(&db)),
            Arc::new(GroupChatService::new(&db)),
            Arc::new(UserService::new(&db)),
            Arc::new(mock_group_membership_service),
        );

        let mut payload = InvitationFactory::fake_invitation_update_status_dto();
        payload.invitation_id = invitation.id;
        payload.status = InvitationStatus::Accepted;

        let result = invitation_service.update_status(payload, target_user.id).await;

        // Il test dovrebbe fallire con l'errore simulato
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(ruggine_server::error::db_error::DbError::SomethingWentWrong(msg)) => {
                assert!(msg.contains("Simulated database error during create_checked"));
            }
            e => panic!("Expected DbError::SomethingWentWrong, got: {:?}", e),
        }

        // Verifichiamo che l'invitation non sia stata aggiornata (rollback della transazione)
        let invitation_after = invitation_service.find_by_id_and_user_id(invitation.id, target_user.id).await.unwrap();
        
        // L'invitation dovrebbe essere ancora pending, non accepted
        assert_eq!(invitation_after.status, InvitationStatus::Pending);

        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
    }
}
