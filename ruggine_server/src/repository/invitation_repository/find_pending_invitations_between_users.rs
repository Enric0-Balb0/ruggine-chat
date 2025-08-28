use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::invitation::{Invitation, InvitationStatus};
use crate::repository::invitation_repository::InvitationRepository;

impl InvitationRepository {
    pub async fn find_pending_invitation_between_users_inner(
        &self,
        from_user_id: i32,
        to_user_id: i32,
        group_chat_id: i32,
    ) -> Result<Option<Invitation>, Error> {
        let query = sqlx::query_as::<_, Invitation>(
            r#"
        SELECT * FROM "invitation"
        WHERE from_user_id = $1 AND to_user_id = $2 AND group_chat_id = $3 AND status = $4
        "#,
        )
            .bind(from_user_id)
            .bind(to_user_id)
            .bind(group_chat_id)
            .bind(InvitationStatus::Pending);

        // se c'è una transazione, usala; altrimenti usa la pool
        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_optional(&mut *tx_ref).await
        } else {
            query.fetch_optional(self.db_conn.get_pool()).await
        }
    }
}

#[cfg(test)]
mod invitation_repository_find_pending_between_users_tests {
    use mockall::predicate::*;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::repository::invitation_repository::InvitationRepositoryTrait;
    use crate::factory::invitation_factory::InvitationFactory;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_found() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let from_user_id = 1;
        let to_user_id = 2;
        let expected_invitation = InvitationFactory::fake_invitation_from_ids(from_user_id, to_user_id, 1);
        let expected_invitation_clone = expected_invitation.clone();

        mock_invitation_repo
            .expect_find_pending_invitation_between_users()
            .with(eq(from_user_id), eq(to_user_id), eq(1))
            .times(1)
            .returning(move |_from_id, _to_id, _group_chat_id| {
                let value = expected_invitation_clone.clone();
                Box::pin(async move {
                    Ok(Some(value))
                })
            });

        // Act
        let result = mock_invitation_repo.find_pending_invitation_between_users(from_user_id, to_user_id, 1).await;

        // Assert
        assert!(result.is_ok());
        let found_invitation = result.unwrap();
        assert!(found_invitation.is_some());
        
        let invitation = found_invitation.unwrap();
        assert_eq!(expected_invitation, invitation)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_not_found() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let from_user_id = 999;
        let to_user_id = 998;
        let group_chat_id = 1;

        mock_invitation_repo
            .expect_find_pending_invitation_between_users()
            .with(eq(from_user_id), eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _, _| {
                Box::pin(async move { Ok(None) })
            });

        // Act
        let result = mock_invitation_repo.find_pending_invitation_between_users(from_user_id, to_user_id, group_chat_id).await;

        // Assert
        assert!(result.is_ok());
        let found_invitation = result.unwrap();
        assert!(found_invitation.is_none());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_database_error() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let from_user_id = 1;
        let to_user_id = 2;
        let group_chat_id = 1;

        mock_invitation_repo
            .expect_find_pending_invitation_between_users()
            .with(eq(from_user_id), eq(to_user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _,_| {
                Box::pin(async move { Err(sqlx::Error::PoolClosed) })
            });

        // Act
        let result = mock_invitation_repo.find_pending_invitation_between_users(from_user_id, to_user_id, group_chat_id).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), sqlx::Error::PoolClosed));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitation_between_users_different_users() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let from_user_id = 5;
        let to_user_id = 10;
        let expected_invitation = InvitationFactory::fake_invitation_from_ids(from_user_id, to_user_id, 3);
        let expected_invitation_clone = expected_invitation.clone();

        mock_invitation_repo
            .expect_find_pending_invitation_between_users()
            .with(eq(from_user_id), eq(to_user_id), eq(3))
            .times(1)
            .returning(move |_from_id, _to_id, _group_chat_id| {
                let value = expected_invitation_clone.clone();
                Box::pin(async move {
                    Ok(Some(value))
                })
            });

        // Act
        let result = mock_invitation_repo.find_pending_invitation_between_users(from_user_id, to_user_id, 3).await;

        // Assert
        assert!(result.is_ok());
        let found_invitation = result.unwrap();
        assert!(found_invitation.is_some());
        
        let invitation = found_invitation.unwrap();
        assert_eq!(expected_invitation, invitation);
    }
}
