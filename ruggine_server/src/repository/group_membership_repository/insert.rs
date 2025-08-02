use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::group_membership::{MembershipStatus, NewGroupMembership};
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn insert_inner(&self, new_membership: NewGroupMembership) -> Result<i32, Error> {
        let now = chrono::Utc::now();

        let query = sqlx::query_scalar(
            r#"
        INSERT INTO "group_membership" (user_id, group_chat_id, role, joined_at, membership_status)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
        )
            .bind(new_membership.user_id)
            .bind(new_membership.group_chat_id)
            .bind(new_membership.role)
            .bind(now)
            .bind(MembershipStatus::Active);

        match query.fetch_one(self.db_conn.get_pool()).await {
            Ok(id) => Ok(id),
            Err(err) => {
                eprintln!("Failed to insert group membership: {:?}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod group_membership_repository_insert_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
    use crate::utils::mock_database_error::MockDatabaseError;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_success() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let new_membership = GroupMembershipFactory::fake_new_group_membership();
        let expected_id = 123;

        mock_group_membership_repo
            .expect_insert()
            .with(eq(new_membership.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(expected_id)
                })
            });

        // Act
        let result = mock_group_membership_repo.insert(new_membership).await;

        // Assert
        assert!(result.is_ok());
        let membership_id = result.unwrap();
        assert_eq!(membership_id, expected_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_admin_membership() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let new_membership = GroupMembershipFactory::fake_new_admin_group_membership();
        let expected_id = 456;

        mock_group_membership_repo
            .expect_insert()
            .with(eq(new_membership.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(expected_id)
                })
            });

        // Act
        let result = mock_group_membership_repo.insert(new_membership).await;

        // Assert
        assert!(result.is_ok());
        let membership_id = result.unwrap();
        assert_eq!(membership_id, expected_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_database_error() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let new_membership = GroupMembershipFactory::fake_new_group_membership();

        mock_group_membership_repo
            .expect_insert()
            .with(eq(new_membership.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::PoolClosed)
                })
            });

        // Act
        let result = mock_group_membership_repo.insert(new_membership).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), sqlx::Error::PoolClosed);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_foreign_key_violation() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let new_membership = GroupMembershipFactory::fake_new_group_membership();

        mock_group_membership_repo
            .expect_insert()
            .with(eq(new_membership.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(MockDatabaseError::foreign_key_violation())
                })
            });

        // Act
        let result = mock_group_membership_repo.insert(new_membership).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), Error::Database(_));
        
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_with_unique_memberships() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        
        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_ids(0, 0);
        let membership2 = GroupMembershipFactory::fake_new_group_membership_with_ids(0, 1);

        let expected_id1 = 100;
        let expected_id2 = 200;

        mock_group_membership_repo
            .expect_insert()
            .with(eq(membership1.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(expected_id1)
                })
            });

        mock_group_membership_repo
            .expect_insert()
            .with(eq(membership2.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(expected_id2)
                })
            });

        // Act
        let result1 = mock_group_membership_repo.insert(membership1).await;
        let result2 = mock_group_membership_repo.insert(membership2).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), expected_id1);
        assert_eq!(result2.unwrap(), expected_id2);
    }
}
