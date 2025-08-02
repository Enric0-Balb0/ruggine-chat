use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::group_membership::GroupMembership;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn find_by_id_inner(&self, id: i32) -> Result<GroupMembership, Error> {
        let membership = sqlx::query_as::<_, GroupMembership>(
            "SELECT * FROM \"group_membership\" WHERE id = $1"
        )
        .bind(id)
        .fetch_one(self.db_conn.get_pool())
        .await;
        membership
    }
}

#[cfg(test)]
mod group_membership_repository_find_tests {
    use mockall::predicate::*;
    use crate::entity::group_membership::{MembershipStatus, MemberRole};
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
    use crate::factory::group_membership_factory::GroupMembershipFactory;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let expected_membership = GroupMembershipFactory::fake_group_membership();
        let expected_membership_clone = expected_membership.clone();

        let membership_id = expected_membership.id;

        mock_group_membership_repo
            .expect_find_by_id()
            .with(eq(membership_id))
            .times(1)
            .returning(move |_| {
                let membership = expected_membership_clone.clone();
                Box::pin(async move { Ok(membership) })
            });

        // Act
        let result = mock_group_membership_repo.find_by_id(membership_id).await;

        // Assert
        assert!(result.is_ok());
        let found_membership = result.unwrap();
        assert_eq!(found_membership.id, expected_membership.id);
        assert_eq!(found_membership.user_id, expected_membership.user_id);
        assert_eq!(found_membership.group_chat_id, expected_membership.group_chat_id);
        assert_eq!(found_membership.role, expected_membership.role);
        assert_eq!(found_membership.joined_at, expected_membership.joined_at);
        assert_eq!(found_membership.left_at, expected_membership.left_at);
        assert_eq!(found_membership.membership_status, expected_membership.membership_status);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let membership_id = -1;

        mock_group_membership_repo
            .expect_find_by_id()
            .with(eq(membership_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { 
                    Err(sqlx::Error::RowNotFound) 
                })
            });

        // Act
        let result = mock_group_membership_repo.find_by_id(membership_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => (),
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_admin_membership() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let expected_membership = GroupMembershipFactory::fake_admin_group_membership();
        let expected_membership_clone = expected_membership.clone();

        let membership_id = expected_membership.id;

        mock_group_membership_repo
            .expect_find_by_id()
            .with(eq(membership_id))
            .times(1)
            .returning(move |_| {
                let membership = expected_membership_clone.clone();
                Box::pin(async move { Ok(membership) })
            });

        // Act
        let result = mock_group_membership_repo.find_by_id(membership_id).await;

        // Assert
        assert!(result.is_ok());
        let found_membership = result.unwrap();
        assert_eq!(found_membership.role, MemberRole::Admin);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_left_membership() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let expected_membership = GroupMembershipFactory::fake_left_group_membership();
        let expected_membership_clone = expected_membership.clone();

        let membership_id = expected_membership.id;

        mock_group_membership_repo
            .expect_find_by_id()
            .with(eq(membership_id))
            .times(1)
            .returning(move |_| {
                let membership = expected_membership_clone.clone();
                Box::pin(async move { Ok(membership) })
            });

        // Act
        let result = mock_group_membership_repo.find_by_id(membership_id).await;

        // Assert
        assert!(result.is_ok());
        let found_membership = result.unwrap();
        assert!(found_membership.left_at.is_some());
        assert_eq!(found_membership.membership_status, MembershipStatus::Left);
    }
}
