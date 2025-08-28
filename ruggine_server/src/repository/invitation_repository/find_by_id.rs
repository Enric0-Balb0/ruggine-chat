use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::invitation::Invitation;
use crate::repository::invitation_repository::InvitationRepository;

impl InvitationRepository {
    pub async fn find_by_id_inner(&self, id: i32) -> Result<Invitation, Error> {
        let query = sqlx::query_as::<_, Invitation>("SELECT * FROM \"invitation\" WHERE id = $1")
            .bind(id);

        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_one(&mut *tx_ref).await
        } else {
            query.fetch_one(self.db_conn.get_pool()).await
        }
    }
}

#[cfg(test)]
mod invitation_repository_find_by_id_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::repository::invitation_repository::InvitationRepositoryTrait;
    use crate::factory::invitation_factory::InvitationFactory;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let expected_invitation = InvitationFactory::fake_invitation();
        let expected_invitation_clone_1 = expected_invitation.clone();
        let expected_invitation_clone_2 = expected_invitation.clone(); // per secondo test

        let invitation_id = expected_invitation.id;

        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id))
            .times(1)
            .returning(move |_| {
                let invitation = expected_invitation_clone_1.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Act
        let mut result = mock_invitation_repo.find_by_id(invitation_id).await;

        // Assert
        assert!(result.is_ok());
        let found_invitation = result.unwrap();
        assert_eq!(found_invitation, expected_invitation);

        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id))
            .times(1)
            .returning(move |_| {
                let invitation = expected_invitation_clone_2.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Act
        result = mock_invitation_repo.find_by_id(invitation_id).await;

        // Assert
        assert!(result.is_ok());
        let found_invitation = result.unwrap();
        assert_eq!(found_invitation, expected_invitation);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let non_existent_id = -1;

        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(non_existent_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(Error::RowNotFound) })
            });

        // Act
        let result = mock_invitation_repo.find_by_id(non_existent_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::RowNotFound => {}, // Expected
            other => panic!("Expected RowNotFound error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_database_error() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let invitation_id = 1;

        mock_invitation_repo
            .expect_find_by_id()
            .with(eq(invitation_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::RowNotFound)
                })
            });

        // Act
        let result = mock_invitation_repo.find_by_id(invitation_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::RowNotFound => {}, // Expected
            other => panic!("Expected Database error, got: {:?}", other),
        }
    }
}