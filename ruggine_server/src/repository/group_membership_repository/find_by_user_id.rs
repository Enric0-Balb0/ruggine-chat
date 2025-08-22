use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn find_by_user_id_inner(
        &self,
        user_id: i32,
    ) -> Result<Vec<GroupMembershipWithInvitationRow>, Error> {
        let rows = sqlx::query_as::<_, GroupMembershipWithInvitationRow>(
            r#"
            SELECT 
                gm.id,
                gm.role,
                gm.joined_at,
                gm.left_at,
                gm.membership_status,
                gm.invitation_id,
                i.to_user_id AS user_id,
                i.group_chat_id,
                gm.current_action
            FROM group_membership gm
            JOIN invitation i ON gm.invitation_id = i.id
            WHERE i.to_user_id = $1
              AND gm.membership_status = 'active'
            ORDER BY gm.joined_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(self.db_conn.get_pool())
        .await;

        rows
    }
}

#[cfg(test)]
mod group_membership_repository_find_by_user_id_tests {
    use mockall::predicate::*;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::entity::group_membership::{MembershipStatus, MemberRole};
    use chrono::Utc;
    use sqlx::Error;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let expected = vec![
            GroupMembershipFactory::fake_group_membership_with_invitation_row(),
            GroupMembershipFactory::fake_group_membership_with_invitation_row(),
        ];
        let user_id = expected[0].user_id;

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                let result = vec![
                    GroupMembershipFactory::fake_group_membership_with_invitation_row(),
                    GroupMembershipFactory::fake_group_membership_with_invitation_row(),
                ];
                Box::pin(async move { Ok(result) })
            });

        let result = mock_repo.find_by_user_id(user_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].user_id, user_id);
        assert_eq!(found[1].user_id, user_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_empty_result() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();

        mock_repo
            .expect_find_by_user_id()
            .with(eq(-1))
            .times(1)
            .returning(|_| {
                Box::pin(async move { Ok(vec![]) })
            });

        let result = mock_repo.find_by_user_id(-1).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_single_membership() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let expected = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let user_id = expected.user_id;

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                let result = vec![GroupMembershipFactory::fake_group_membership_with_invitation_row()];
                Box::pin(async move { Ok(result) })
            });

        let result = mock_repo.find_by_user_id(user_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].user_id, user_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_with_admin_role() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                membership.role = MemberRole::Admin;
                membership.user_id = user_id;
                Box::pin(async move { Ok(vec![membership]) })
            });

        let result = mock_repo.find_by_user_id(user_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].role, MemberRole::Admin);
        assert_eq!(found[0].user_id, user_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_with_left_membership() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                membership.left_at = Some(Utc::now());
                membership.membership_status = MembershipStatus::Left;
                membership.user_id = user_id;
                Box::pin(async move { Ok(vec![membership]) })
            });

        let result = mock_repo.find_by_user_id(user_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].membership_status, MembershipStatus::Left);
        assert!(found[0].left_at.is_some());
        assert_eq!(found[0].user_id, user_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();

        mock_repo
            .expect_find_by_user_id()
            .with(eq(1))
            .times(1)
            .returning(|_| {
                Box::pin(async move {
                    Err(sqlx::Error::RowNotFound)
                })
            });

        let result = mock_repo.find_by_user_id(1).await;
        assert!(result.is_err());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_mixed_membership_statuses() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                let mut active_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                active_membership.user_id = user_id;
                active_membership.membership_status = MembershipStatus::Active;
                active_membership.role = MemberRole::Member;

                let mut left_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                left_membership.user_id = user_id;
                left_membership.membership_status = MembershipStatus::Left;
                left_membership.left_at = Some(Utc::now());
                left_membership.role = MemberRole::Admin;

                Box::pin(async move { Ok(vec![active_membership, left_membership]) })
            });

        let result = mock_repo.find_by_user_id(user_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);
        
        let active = &found[0];
        let left = &found[1];
        
        assert_eq!(active.membership_status, MembershipStatus::Active);
        assert_eq!(active.role, MemberRole::Member);
        assert!(active.left_at.is_none());
        
        assert_eq!(left.membership_status, MembershipStatus::Left);
        assert_eq!(left.role, MemberRole::Admin);
        assert!(left.left_at.is_some());
    }
}
