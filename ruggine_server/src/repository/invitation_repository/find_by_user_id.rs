use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::invitation::{Invitation, UserInvitationFilter};
use crate::repository::invitation_repository::InvitationRepository;

impl InvitationRepository {
    /// Find all invitations for a specific user based on the specified filter
    pub async fn find_by_user_id_inner(&self, user_id: i32, filter: UserInvitationFilter) -> Result<Vec<Invitation>, Error> {
        let (where_clause, bind_count) = match filter {
            UserInvitationFilter::AsSender => ("from_user_id = $1", 1),
            UserInvitationFilter::AsRecipient => ("to_user_id = $1", 1),
            UserInvitationFilter::Both => ("from_user_id = $1 OR to_user_id = $1", 1),
        };

        let query = format!(
            r#"
            SELECT * FROM "invitation" 
            WHERE {}
            ORDER BY sent_at DESC
            "#,
            where_clause
        );

        let invitations = sqlx::query_as::<_, Invitation>(&query)
            .bind(user_id)
            .fetch_all(self.db_conn.get_pool())
            .await;
        
        invitations
    }
}

#[cfg(test)]
mod invitation_repository_find_by_user_id_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::repository::invitation_repository::InvitationRepositoryTrait;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::entity::invitation::UserInvitationFilter;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_success_as_recipient() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 1;
        let filter = UserInvitationFilter::AsRecipient;
        
        let invitation1 = InvitationFactory::fake_invitation_from_ids(1, 2, user_id); // received by user
        let invitation2 = InvitationFactory::fake_invitation_from_ids(2, 3, user_id); // received by user
        let expected_invitations = vec![invitation1.clone(), invitation2.clone()];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(filter))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        // Act
        let result = mock_invitation_repo.find_by_user_id(user_id, UserInvitationFilter::AsRecipient).await;

        // Assert
        assert!(result.is_ok());
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2);
        assert_eq!(found_invitations, expected_invitations);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_success_as_sender() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 1;
        let filter = UserInvitationFilter::AsSender;
        
        let invitation1 = InvitationFactory::fake_invitation_from_ids(1, user_id, 2); // sent by user
        let invitation2 = InvitationFactory::fake_invitation_from_ids(2, user_id, 3); // sent by user
        let expected_invitations = vec![invitation1.clone(), invitation2.clone()];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(filter))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        // Act
        let result = mock_invitation_repo.find_by_user_id(user_id, UserInvitationFilter::AsSender).await;

        // Assert
        assert!(result.is_ok());
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2);
        assert_eq!(found_invitations, expected_invitations);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_success_both() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 1;
        let filter = UserInvitationFilter::Both;
        
        let invitation1 = InvitationFactory::fake_invitation_from_ids(1, user_id, 2); // sent by user
        let invitation2 = InvitationFactory::fake_invitation_from_ids(2, 3, user_id); // received by user
        let expected_invitations = vec![invitation1.clone(), invitation2.clone()];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(filter))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        // Act
        let result = mock_invitation_repo.find_by_user_id(user_id, UserInvitationFilter::Both).await;

        // Assert
        assert!(result.is_ok());
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2);
        assert_eq!(found_invitations, expected_invitations);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_empty_result() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 999; // Non-existent user
        let filter = UserInvitationFilter::AsRecipient;
        let expected_invitations: Vec<Invitation> = vec![];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(filter))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        // Act
        let result = mock_invitation_repo.find_by_user_id(user_id, UserInvitationFilter::AsRecipient).await;

        // Assert
        assert!(result.is_ok());
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 0);
        assert_eq!(found_invitations, expected_invitations);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_database_error() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 1;
        let filter = UserInvitationFilter::AsRecipient;

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(filter))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(Error::PoolClosed) })
            });

        // Act
        let result = mock_invitation_repo.find_by_user_id(user_id, UserInvitationFilter::AsRecipient).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::PoolClosed => {
                // Expected error
            }
            _ => panic!("Expected PoolClosed error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_only_sent_invitations() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 5;
        let filter = UserInvitationFilter::AsSender;
        
        let invitation1 = InvitationFactory::fake_invitation_from_ids(user_id, 10, 1); // sent by user
        let invitation2 = InvitationFactory::fake_invitation_from_ids(user_id, 11, 2); // sent by user
        let expected_invitations = vec![invitation1.clone(), invitation2.clone()];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(filter))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        // Act
        let result = mock_invitation_repo.find_by_user_id(user_id, UserInvitationFilter::AsSender).await;

        // Assert
        assert!(result.is_ok());
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2);
        assert!(found_invitations.iter().all(|inv| inv.from_user_id == user_id));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_only_received_invitations() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let user_id = 7;
        let filter = UserInvitationFilter::AsRecipient;
        
        let invitation1 = InvitationFactory::fake_invitation_from_ids(20, user_id, 3); // received by user
        let invitation2 = InvitationFactory::fake_invitation_from_ids(21, user_id, 4); // received by user
        let expected_invitations = vec![invitation1.clone(), invitation2.clone()];
        let expected_invitations_clone = expected_invitations.clone();

        mock_invitation_repo
            .expect_find_by_user_id()
            .with(eq(user_id), eq(filter))
            .times(1)
            .returning(move |_, _| {
                let invitations = expected_invitations_clone.clone();
                Box::pin(async move { Ok(invitations) })
            });

        // Act
        let result = mock_invitation_repo.find_by_user_id(user_id, UserInvitationFilter::AsRecipient).await;

        // Assert
        assert!(result.is_ok());
        let found_invitations = result.unwrap();
        assert_eq!(found_invitations.len(), 2);
        assert!(found_invitations.iter().all(|inv| inv.to_user_id == user_id));
    }
}
