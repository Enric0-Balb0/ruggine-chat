use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::invitation::NewInvitation;
use crate::repository::invitation_repository::InvitationRepository;

impl InvitationRepository {
    pub async fn insert_inner(&self, new_invitation: NewInvitation) -> Result<i32, Error> {
        let now = chrono::Utc::now();
        let query = sqlx::query_scalar(
            r#"
            INSERT INTO "invitation" (from_user_id, to_user_id, group_chat_id, status, sent_at, role_at_join)
            VALUES ($1, $2, $3, 'pending', $4, $5)
            RETURNING id
            "#
        )
            .bind(new_invitation.from_user_id)
            .bind(new_invitation.to_user_id)
            .bind(new_invitation.group_chat_id)
            .bind(now)
            .bind(new_invitation.role_at_join);

        // se c'è una transazione, usala; altrimenti usa la pool
        let response;

        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            println!("Using transaction");
            response = query.fetch_one(&mut *tx_ref).await
        } else {
            println!("Using pool");
            response = if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
                query.fetch_one(&mut *tx_ref).await
            } else {
                query.fetch_one(self.db_conn.get_pool()).await
            }
        }

        match response {
            Ok(id) => Ok(id),
            Err(err) => {
                eprintln!("Failed to insert invitation repository: {:?}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod invitation_repository_insert_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::entity::group_membership::MemberRole;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::repository::invitation_repository::invitation_repository_trait::MockInvitationRepositoryTrait;
    use crate::repository::invitation_repository::InvitationRepositoryTrait;
    use crate::utils::mock_database_error::MockDatabaseError;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_success() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let new_invitation = InvitationFactory::fake_new_invitation();
        let expected_id = 123;

        mock_invitation_repo
            .expect_insert()
            .with(eq(new_invitation.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(expected_id)
                })
            });

        // Act
        let result = mock_invitation_repo.insert(new_invitation).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    async fn test_insert_admin_success() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let new_invitation = InvitationFactory::fake_new_admin_invitation();
        let expected_id = 123;

        mock_invitation_repo
            .expect_insert()
            .with(eq(new_invitation.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(expected_id)
                })
            });

        // Act
        let result = mock_invitation_repo.insert(new_invitation).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_database_error() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let new_invitation = InvitationFactory::fake_new_invitation();

        mock_invitation_repo
            .expect_insert()
            .with(eq(new_invitation.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::PoolClosed)
                })
            });

        // Act
        let result = mock_invitation_repo.insert(new_invitation).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::PoolClosed => {}, // Expected
            other => panic!("Expected PoolClosed error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_with_different_users() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        
        let invitation1 = NewInvitation { from_user_id: 1, to_user_id: 2, group_chat_id: 1, role_at_join: MemberRole::Member };
        let invitation2 =  NewInvitation { from_user_id: 2, to_user_id: 3, group_chat_id: 1, role_at_join: MemberRole::Admin };
        let invitation3 =  NewInvitation { from_user_id: 3, to_user_id: 4, group_chat_id: 1, role_at_join: MemberRole::Member };

        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation1.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(101)
                })
            });

        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation2.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(102)
                })
            });

        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation3.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(103)
                })
            });

        // Act
        let result1 = mock_invitation_repo.insert(invitation1).await;
        let result2 = mock_invitation_repo.insert(invitation2).await;
        let result3 = mock_invitation_repo.insert(invitation3).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        assert_eq!(result1.unwrap(), 101);
        assert_eq!(result2.unwrap(), 102);
        assert_eq!(result3.unwrap(), 103);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_with_factory_utilities() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        
        let base_invitation = InvitationFactory::fake_new_invitation();
        let custom_invitation = InvitationFactory::with_group_chat_id(
            InvitationFactory::with_to_user_id(
                InvitationFactory::with_from_user_id(base_invitation, 42),
                43
            ),
            44
        );

        mock_invitation_repo
            .expect_insert()
            .with(eq(custom_invitation.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(999)
                })
            });

        // Act
        let result = mock_invitation_repo.insert(custom_invitation).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 999);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_foreign_key_constraint_error() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        let mut invitation = InvitationFactory::fake_new_invitation();
        invitation.from_user_id = 99999; // Non-existent user ID

        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::Database(Box::new(MockDatabaseError::new("23503".to_string())))) // Simulate FK constraint error
                })
            });

        // Act
        let result = mock_invitation_repo.insert(invitation).await;

        // Assert
        assert!(result.is_err(), "Expected a foreign key constraint error");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_multiple_unique_invitations() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();
        
        let invitation1 = NewInvitation { from_user_id: 1, to_user_id: 2, group_chat_id: 1, role_at_join: MemberRole::Member };
        let invitation2 = NewInvitation { from_user_id: 1, to_user_id: 3, group_chat_id: 1, role_at_join: MemberRole::Member };
        let invitation3 = NewInvitation { from_user_id: 1, to_user_id: 4, group_chat_id: 1, role_at_join: MemberRole::Member };

        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation1.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(201)
                })
            });

        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation2.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(202)
                })
            });

        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation3.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(203)
                })
            });

        // Act
        let results = tokio::join!(
            mock_invitation_repo.insert(invitation1),
            mock_invitation_repo.insert(invitation2),
            mock_invitation_repo.insert(invitation3)
        );

        // Assert
        assert!(results.0.is_ok());
        assert!(results.1.is_ok());
        assert!(results.2.is_ok());
        assert_eq!(results.0.unwrap(), 201);
        assert_eq!(results.1.unwrap(), 202);
        assert_eq!(results.2.unwrap(), 203);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_duplicate_pending_invitation_should_fail() {
        // Arrange
        let mut mock_invitation_repo = MockInvitationRepositoryTrait::new();

        let invitation = NewInvitation {
            from_user_id: 1,
            to_user_id: 2,
            group_chat_id: 1,
            role_at_join: MemberRole::Member,
        };

        // Prima insert va a buon fine
        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(100)
                })
            });

        // Seconda insert con stessi valori simula duplicato (vincolo UNIQUE)
        mock_invitation_repo
            .expect_insert()
            .with(eq(invitation.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::Database(Box::new(MockDatabaseError::new("23505".to_string()))))
                })
            });

        // Act
        let first = mock_invitation_repo.insert(invitation.clone()).await;
        let second = mock_invitation_repo.insert(invitation.clone()).await;

        // Assert
        assert!(first.is_ok());
        assert_eq!(first.unwrap(), 100);

        assert!(second.is_err());
        match second.err() {
            Some(Error::Database(db_err)) => {
                assert_eq!(db_err.code().as_deref(), Some("23505"));
                // OK
            }
            _ => {
                panic!("Expected UniqueConstraintViolation (23505)");
            }
        }
    }
}