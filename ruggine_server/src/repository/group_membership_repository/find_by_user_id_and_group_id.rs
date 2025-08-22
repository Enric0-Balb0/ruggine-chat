use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::group_membership::MembershipStatus;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn find_by_user_id_and_group_id_inner(
        &self,
        user_id: i32,
        group_id: i32,
        membership_statuses: Vec<MembershipStatus>,
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
              AND i.group_chat_id = $2
              AND gm.membership_status = ANY($3)
            "#
        )
        .bind(user_id)
        .bind(group_id)
        .bind(&membership_statuses)
        .fetch_all(self.db_conn.get_pool())
        .await?;

        Ok(rows)
    }
}


#[cfg(test)]
mod group_membership_repository_find_by_user_id_and_group_id_tests {
    use mockall::predicate::*;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::entity::group_membership::{MembershipStatus, MemberRole, all_membership_statuses};
    use chrono::Utc;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let group_id = 10;

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id), eq(all_membership_statuses()))
            .times(1)
            .returning(move |_, _, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id;
                result.group_chat_id = group_id;
                result.membership_status = MembershipStatus::Active;
                Box::pin(async move { Ok(vec![result]) })
            });

        let rows = mock_repo.find_by_user_id_and_group_id(user_id, group_id, all_membership_statuses()).await.unwrap();
        assert_eq!(rows.len(), 1);
        let found = &rows[0];
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.group_chat_id, group_id);
        assert_eq!(found.membership_status, MembershipStatus::Active);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_not_found() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(999), eq(888), eq(all_membership_statuses()))
            .times(1)
            .returning(|_, _, _| {
                Box::pin(async move { Ok(vec![]) })  // niente record
            });

        let rows = mock_repo.find_by_user_id_and_group_id(999, 888, all_membership_statuses()).await.unwrap();
        assert!(rows.is_empty());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_admin_membership() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 5;
        let group_id = 15;

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id), eq(all_membership_statuses()))
            .times(1)
            .returning(move |_, _, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id;
                result.group_chat_id = group_id;
                result.role = MemberRole::Admin;
                result.membership_status = MembershipStatus::Active;
                result.joined_at = Utc::now();
                Box::pin(async move { Ok(vec![result]) })
            });

        let rows = mock_repo.find_by_user_id_and_group_id(user_id, group_id, all_membership_statuses()).await.unwrap();
        assert_eq!(rows.len(), 1);
        let found = &rows[0];
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
            .with(eq(1), eq(1), eq(all_membership_statuses()))
            .times(1)
            .returning(|_, _, _| {
                Box::pin(async move { Err(sqlx::Error::PoolClosed) })
            });

        let result = mock_repo.find_by_user_id_and_group_id(1, 1, all_membership_statuses()).await;
        assert!(result.is_err());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_different_users_same_group() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let group_id = 20;
        let user_id_1 = 10;
        let user_id_2 = 11;

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id_1), eq(group_id), eq(all_membership_statuses()))
            .times(1)
            .returning(move |_, _, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id_1;
                result.group_chat_id = group_id;
                result.role = MemberRole::Admin;
                Box::pin(async move { Ok(vec![result]) })
            });

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id_2), eq(group_id), eq(all_membership_statuses()))
            .times(1)
            .returning(move |_, _, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id_2;
                result.group_chat_id = group_id;
                result.role = MemberRole::Member;
                Box::pin(async move { Ok(vec![result]) })
            });

        let rows1 = mock_repo.find_by_user_id_and_group_id(user_id_1, group_id, all_membership_statuses()).await.unwrap();
        let rows2 = mock_repo.find_by_user_id_and_group_id(user_id_2, group_id, all_membership_statuses()).await.unwrap();

        assert_eq!(rows1[0].user_id, user_id_1);
        assert_eq!(rows1[0].role, MemberRole::Admin);
        assert_eq!(rows2[0].user_id, user_id_2);
        assert_eq!(rows2[0].role, MemberRole::Member);
        assert_eq!(rows1[0].group_chat_id, rows2[0].group_chat_id);
    }
    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_and_group_id_filtered_statuses() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 3;
        let group_id = 30;
        let requested_statuses = vec![MembershipStatus::Active, MembershipStatus::Banned];

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id), eq(requested_statuses.clone()))
            .times(1)
            .returning(move |_, _, _| {
                let mut active_member = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                active_member.user_id = user_id;
                active_member.group_chat_id = group_id;
                active_member.membership_status = MembershipStatus::Active;

                let mut banned_member = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                banned_member.user_id = user_id;
                banned_member.group_chat_id = group_id;
                banned_member.membership_status = MembershipStatus::Banned;

                let mut left_member = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                left_member.user_id = user_id;
                left_member.group_chat_id = group_id;
                left_member.membership_status = MembershipStatus::Left;

                // solo Active e Banned devono tornare
                Box::pin(async move { Ok(vec![active_member, banned_member]) })
            });

        let rows = mock_repo.find_by_user_id_and_group_id(user_id, group_id, requested_statuses.clone()).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| matches!(r.membership_status, MembershipStatus::Active | MembershipStatus::Banned)));
    }
}
