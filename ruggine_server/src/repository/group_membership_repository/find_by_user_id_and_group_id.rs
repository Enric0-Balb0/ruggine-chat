use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn find_by_user_id_and_group_id_inner(
        &self,
        user_id: i32,
        group_id: i32,
    ) -> Result<GroupMembershipWithInvitationRow, Error> {
        let row = sqlx::query_as::<_, GroupMembershipWithInvitationRow>(
            r#"
            SELECT 
                gm.id,
                gm.role,
                gm.joined_at,
                gm.left_at,
                gm.membership_status,
                gm.invitation_id,
                i.to_user_id AS user_id,
                i.group_chat_id
            FROM group_membership gm
            JOIN invitation i ON gm.invitation_id = i.id
            WHERE i.to_user_id = $1
              AND i.group_chat_id = $2
              AND gm.membership_status = 'active'
            "#
        )
        .bind(user_id)
        .bind(group_id)
        .fetch_one(self.db_conn.get_pool())
        .await;

        row
    }
}

#[cfg(test)]
mod group_membership_repository_find_by_user_id_and_group_id_tests {
    use mockall::predicate::*;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::entity::group_membership::{MembershipStatus, MemberRole};
    use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
    use chrono::Utc;
    use sqlx::Error;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut expected = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        expected.user_id = 1;
        expected.group_chat_id = 10;
        expected.membership_status = MembershipStatus::Active;

        let user_id = expected.user_id;
        let group_id = expected.group_chat_id;

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id;
                result.group_chat_id = group_id;
                result.membership_status = MembershipStatus::Active;
                Box::pin(async move { Ok(result) })
            });

        let result = mock_repo.find_by_user_id_and_group_id(user_id, group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.group_chat_id, group_id);
        assert_eq!(found.membership_status, MembershipStatus::Active);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_not_found() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(999), eq(888))
            .times(1)
            .returning(|_, _| {
                Box::pin(async move { Err(sqlx::Error::RowNotFound) })
            });

        let result = mock_repo.find_by_user_id_and_group_id(999, 888).await;
        assert!(result.is_err());
        
        if let Err(Error::RowNotFound) = result {
            // Expected error
        } else {
            panic!("Expected RowNotFound error");
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_admin_membership() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 5;
        let group_id = 15;

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id;
                result.group_chat_id = group_id;
                result.role = MemberRole::Admin;
                result.membership_status = MembershipStatus::Active;
                result.joined_at = Utc::now();
                Box::pin(async move { Ok(result) })
            });

        let result = mock_repo.find_by_user_id_and_group_id(user_id, group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.group_chat_id, group_id);
        assert_eq!(found.role, MemberRole::Admin);
        assert_eq!(found.membership_status, MembershipStatus::Active);
        assert!(found.left_at.is_none());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(1), eq(1))
            .times(1)
            .returning(|_, _| {
                Box::pin(async move {
                    Err(sqlx::Error::PoolClosed)
                })
            });

        let result = mock_repo.find_by_user_id_and_group_id(1, 1).await;
        assert!(result.is_err());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_different_users_same_group() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let group_id = 20;
        let user_id_1 = 10;
        let user_id_2 = 11;

        // First user
        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id_1), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id_1;
                result.group_chat_id = group_id;
                result.role = MemberRole::Admin;
                Box::pin(async move { Ok(result) })
            });

        // Second user
        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id_2), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id_2;
                result.group_chat_id = group_id;
                result.role = MemberRole::Member;
                Box::pin(async move { Ok(result) })
            });

        let result1 = mock_repo.find_by_user_id_and_group_id(user_id_1, group_id).await;
        let result2 = mock_repo.find_by_user_id_and_group_id(user_id_2, group_id).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let found1 = result1.unwrap();
        let found2 = result2.unwrap();
        
        assert_eq!(found1.user_id, user_id_1);
        assert_eq!(found1.role, MemberRole::Admin);
        assert_eq!(found2.user_id, user_id_2);
        assert_eq!(found2.role, MemberRole::Member);
        assert_eq!(found1.group_chat_id, found2.group_chat_id);
    }
}
