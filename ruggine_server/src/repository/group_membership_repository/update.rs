use sqlx::Error;
use tracing::info;
use crate::config::database::DatabaseTrait;
use crate::entity::group_membership::UpdateGroupMembership;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn update_inner(&self, update_membership: UpdateGroupMembership) -> Result<(), Error> {
        // Validate the update before executing
        if !update_membership.has_updates() {
            return Ok(()); // No changes to make
        }

        if !update_membership.is_valid() {
            return Err(Error::RowNotFound); // Invalid update combination
        }

        // Build dynamic query based on what needs to be updated
        let mut set_clauses = Vec::new();
        let mut bind_index = 2; // Start from $2 since $1 is the id

        if update_membership.role.is_some() {
            set_clauses.push(format!("role = ${}", bind_index));
            bind_index += 1;
        }

        if update_membership.membership_status.is_some() {
            set_clauses.push(format!("membership_status = ${}", bind_index));
            bind_index += 1;
        }

        if update_membership.left_at.is_some() {
            set_clauses.push(format!("left_at = ${}", bind_index));
            bind_index += 1;
        }

        if update_membership.current_action.is_some() {
            set_clauses.push(format!("current_action = ${}", bind_index));
            bind_index += 1;
        }

        if set_clauses.is_empty() {
            return Ok(()); // Nothing to update
        }

        let sql = format!(
            "UPDATE group_membership SET {} WHERE id = $1",
            set_clauses.join(", ")
        );

        // Build query with dynamic binding
        let mut query = sqlx::query(&sql).bind(update_membership.id);

        if let Some(role) = update_membership.role {
            query = query.bind(role);
        }

        if let Some(membership_status) = update_membership.membership_status {
            query = query.bind(membership_status);
        }

        if let Some(left_at) = update_membership.left_at {
            query = query.bind(left_at);
        }

        if let Some(current_cation) = update_membership.current_action {
            query = query.bind(current_cation)
        }

        /* match query.execute(self.db_conn.get_pool()).await {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(Error::RowNotFound)
                } else {
                    Ok(())
                }
            }
            Err(err) => {
                eprintln!("Failed to update group membership: {:?}", err);
                Err(err)
            }
        } */

        let result = if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            info!("Using transaction");
            query.execute(&mut *tx_ref).await
        } else {
            info!("Using pool");
            query.execute(self.db_conn.get_pool()).await
        };

        match result {
            Ok(done) => {
                if done.rows_affected() == 0 {
                    Err(Error::RowNotFound)
                } else {
                    Ok(())
                }
            }
            Err(err) => {
                eprintln!("Failed to update group membership: {:?}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod group_membership_repository_update_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
    use crate::entity::group_membership::{CurrentAction, MemberRole, MembershipStatus};
    use chrono::Utc;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_success() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let update_membership = GroupMembershipFactory::fake_leave_group_membership();

        mock_group_membership_repo
            .expect_update()
            .with(eq(update_membership.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        // Act
        let result = mock_group_membership_repo.update(update_membership).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_membership_not_found() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let update_membership = GroupMembershipFactory::fake_leave_group_membership();

        mock_group_membership_repo
            .expect_update()
            .with(eq(update_membership.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Err(Error::RowNotFound) }));

        // Act
        let result = mock_group_membership_repo.update(update_membership).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::RowNotFound => {
                // Expected error
            }
            e => panic!("Expected RowNotFound error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_role_change() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let update_membership = UpdateGroupMembership {
            id: 1,
            role: Some(MemberRole::Admin),
            membership_status: None,
            left_at: None,
            current_action: None
        };

        mock_group_membership_repo
            .expect_update()
            .with(eq(update_membership.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        // Act
        let result = mock_group_membership_repo.update(update_membership).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_current_action_change() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let update_membership = UpdateGroupMembership {
            id: 1,
            role: None,
            membership_status: None,
            left_at: None,
            current_action: Some(CurrentAction::Writing),
        };

        mock_group_membership_repo
            .expect_update()
            .with(eq(update_membership.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        // Act
        let result = mock_group_membership_repo.update(update_membership).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_leave_group() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let update_membership = UpdateGroupMembership {
            id: 1,
            role: None,
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(Utc::now()),
            current_action: None,
        };

        mock_group_membership_repo
            .expect_update()
            .with(eq(update_membership.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        // Act
        let result = mock_group_membership_repo.update(update_membership).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_database_error() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let update_membership = GroupMembershipFactory::fake_leave_group_membership();

        mock_group_membership_repo
            .expect_update()
            .with(eq(update_membership.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { 
                Err(Error::Database(Box::new(crate::utils::mock_database_error::MockDatabaseError::new("23000".to_string()))))
            }));

        // Act
        let result = mock_group_membership_repo.update(update_membership).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Database(_) => {
                // Expected error
            }
            e => panic!("Expected Database error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_no_changes() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let update_membership = UpdateGroupMembership {
            id: 1,
            role: None,
            membership_status: None,
            left_at: None,
            current_action: None,
        };

        mock_group_membership_repo
            .expect_update()
            .with(eq(update_membership.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        // Act
        let result = mock_group_membership_repo.update(update_membership).await;

        // Assert
        assert!(result.is_ok());
    }
}
