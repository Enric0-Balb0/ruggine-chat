use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::invitation::{Invitation, InvitationStatus};
use crate::repository::invitation_repository::InvitationRepository;

impl InvitationRepository {
    pub async fn find_pending_invitations_for_user_inner(&self, user_id: i32) -> Result<Vec<Invitation>, Error> {
        let invitations = sqlx::query_as::<_, Invitation>(
            "SELECT * FROM \"invitation\" WHERE to_user_id = $1 AND status = $2 ORDER BY sent_at DESC"
        )
            .bind(user_id)
            .bind(InvitationStatus::Pending)
            .fetch_all(self.db_conn.get_pool())
            .await;
        invitations
    }
}

#[cfg(test)]
mod invitation_repository_find_pending_for_user_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::repository::invitation_repository::InvitationRepositoryTrait;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::entity::invitation::InvitationStatus;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitations_for_user_success() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 2;
        let expected_invitations = vec![
            InvitationFactory::fake_invitation_with_status(InvitationStatus::Pending),
            InvitationFactory::fake_invitation_with_status(InvitationStatus::Pending),
        ];

        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_pending_invitations_for_user()
            .with(eq(user_id))
            .times(1)
            .returning(move |_user_id| {
                let value = expected_invitations_clone.clone();
                Box::pin(async move {
                    Ok(value)
                })
            });

        // Act
        let result = mock_invitation_repo.find_pending_invitations_for_user(user_id).await;

        // Assert
        assert!(result.is_ok());
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2);
        assert_eq!(found_invitations, expected_invitations);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitations_for_user_empty_result() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = -1; // Non-existent user

        mock_invitation_repo
            .expect_find_pending_invitations_for_user()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(vec![]) })
            });

        // Act
        let result = mock_invitation_repo.find_pending_invitations_for_user(user_id).await;

        // Assert
        assert!(result.is_ok());
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_pending_invitations_for_user_database_error() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 1;

        mock_invitation_repo
            .expect_find_pending_invitations_for_user()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(Error::PoolClosed) })
            });

        // Act
        let result = mock_invitation_repo.find_pending_invitations_for_user(user_id).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::PoolClosed)));
    }
}
