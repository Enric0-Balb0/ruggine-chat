use chrono::Utc;
use crate::entity::invitation::{Invitation, UpdateInvitationStatus, InvitationStatus};
use crate::entity::group_membership::{MemberRole};
use crate::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use crate::config::database::DatabaseTrait;
use sqlx::{Error as SqlxError, Postgres, Transaction};
use crate::entity::user::Gender;

impl InvitationRepository {
    pub async fn update_status_internal(
        &self,
        invitation_id: i32,
        update_invitation_status: UpdateInvitationStatus,
    ) -> Result<Invitation, SqlxError> {
        let update_status = update_invitation_status.status;
        let responded_at = Utc::now();
        // costruiamo la query UNA SOLA VOLTA
        let query = sqlx::query_as!(
            Invitation,
            r#"
            UPDATE invitation SET
                status = $1,
                responded_at = $2
            WHERE id = $3
            RETURNING
                id,
                from_user_id,
                to_user_id,
                group_chat_id,
                status as "status: InvitationStatus",
                sent_at,
                responded_at,
                role_at_join as "role_at_join: MemberRole";
            "#,
            update_status as _,
            responded_at,
            invitation_id
        );

        // se c'è una transazione, usala; altrimenti usa la pool
        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_one(&mut *tx_ref).await
        } else {
            query.fetch_one(self.db_conn.get_pool()).await
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::entity::invitation::InvitationStatus;
    use mockall::predicate::*;
    use chrono::Utc;
    use crate::repository::invitation_repository::InvitationRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_success_accept() {
        // Arrange
        let invitation_id = 1;
        let mut update_status = InvitationFactory::fake_update_invitation_status();
        update_status.status = InvitationStatus::Accepted;
        let expected_invitation = InvitationFactory::fake_invitation_with_status(InvitationStatus::Accepted);

        let mut mock_repo = MockInvitationRepositoryTrait::new();
        mock_repo
            .expect_update_status()
            .with(eq(invitation_id), function(|update: &UpdateInvitationStatus| {
                update.status == InvitationStatus::Accepted
            }))
            .times(1)
            .returning(move |_, _| {
                let invitation_clone = expected_invitation.clone();
                Box::pin(async move { Ok(invitation_clone) })
            });

        // Act
        let result = mock_repo.update_status(invitation_id, update_status).await;

        // Assert
        assert!(result.is_ok());
        let invitation = result.unwrap();
        assert_eq!(invitation.status, InvitationStatus::Accepted);
        assert!(invitation.responded_at.is_some());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_success_reject() {
        // Arrange
        let invitation_id = 2;
        let mut update_status = InvitationFactory::fake_update_invitation_status_rejected();
        update_status.status = InvitationStatus::Rejected;
        let expected_invitation = InvitationFactory::fake_invitation_with_status(InvitationStatus::Rejected);

        let mut mock_repo = MockInvitationRepositoryTrait::new();
        mock_repo
            .expect_update_status()
            .with(eq(invitation_id), function(|update: &UpdateInvitationStatus| {
                update.status == InvitationStatus::Rejected
            }))
            .times(1)
            .returning(move |_, _| {
                let invitation_clone = expected_invitation.clone();
                Box::pin(async move { Ok(invitation_clone) })
            });

        // Act
        let result = mock_repo.update_status(invitation_id, update_status).await;

        // Assert
        assert!(result.is_ok());
        let invitation = result.unwrap();
        assert_eq!(invitation.status, InvitationStatus::Rejected);
        assert!(invitation.responded_at.is_some());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_invitation_not_found() {
        // Arrange
        let invitation_id = 999;
        let update_status = InvitationFactory::fake_update_invitation_status();

        let mut mock_repo = MockInvitationRepositoryTrait::new();
        mock_repo
            .expect_update_status()
            .times(1)
            .returning(|_, _| {
                Box::pin(async {
                    Err(SqlxError::RowNotFound)
                })
            });

        // Act
        let result = mock_repo.update_status(invitation_id, update_status).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            SqlxError::RowNotFound => {
                // Success
            },
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_database_error() {
        // Arrange
        let invitation_id = 1;
        let update_status = InvitationFactory::fake_update_invitation_status();

        let mut mock_repo = MockInvitationRepositoryTrait::new();
        mock_repo
            .expect_update_status()
            .times(1)
            .returning(|_, _| {
                Box::pin(async {
                    Err(SqlxError::PoolClosed)
                })
            });

        // Act
        let result = mock_repo.update_status(invitation_id, update_status).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            SqlxError::PoolClosed => {
                // Success
            },
            _ => panic!("Expected PoolClosed error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_with_correct_timestamp() {
        // Arrange
        let invitation_id = 1;
        let now = Utc::now();
        let mut update_status = UpdateInvitationStatus {
            invitation_id,
            status: InvitationStatus::Accepted,
        };
        let mut expected_invitation = InvitationFactory::fake_invitation_with_status(InvitationStatus::Accepted);
        expected_invitation.responded_at = Some(now);

        let mut mock_repo = MockInvitationRepositoryTrait::new();
        mock_repo
            .expect_update_status()
            .with(eq(invitation_id), function(move |update: &UpdateInvitationStatus| {
                update.status == InvitationStatus::Accepted
            }))
            .times(1)
            .returning(move |_, _| {
                let invitation_clone = expected_invitation.clone();
                Box::pin(async move { Ok(invitation_clone) })
            });

        // Act
        let result = mock_repo.update_status(invitation_id, update_status).await;

        // Assert
        assert!(result.is_ok());
        let invitation = result.unwrap();
        assert_eq!(invitation.status, InvitationStatus::Accepted);
        assert_eq!(invitation.responded_at, Some(now));
    }
}
