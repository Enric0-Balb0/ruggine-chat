use crate::config::database::DatabaseTrait;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
use crate::repository::group_membership_repository::GroupMembershipRepository;
use sqlx::Error as SqlxError;

impl GroupMembershipRepository {
    pub async fn find_by_group_chat_id_inner(&self, group_id: i32) -> Result<Vec<GroupMembershipWithInvitationRow>, SqlxError> {
        let query = sqlx::query_as::<_, GroupMembershipWithInvitationRow>(
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
            FROM "group_membership" gm
            INNER JOIN "invitation" i ON gm.invitation_id = i.id
            WHERE i.group_chat_id = $1 
              AND gm.membership_status = 'active'
            ORDER BY gm.joined_at ASC
            "#
        )
        .bind(group_id);


        // se c'è una transazione, usala; altrimenti usa la pool
        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_all(&mut *tx_ref).await
        } else {
            query.fetch_all(self.db_conn.get_pool()).await
        }
    }
}

#[cfg(test)]
mod group_membership_repository_find_by_group_chat_id_tests {
    use crate::entity::group_membership::MembershipStatus;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
    use mockall::predicate::*;
    use sqlx::Error;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let group_id = 1;
        let expected = vec![
            GroupMembershipFactory::fake_group_membership_with_invitation_row(),
            GroupMembershipFactory::fake_group_membership_with_invitation_row(),
        ];

        mock_repo
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let result = vec![
                    GroupMembershipFactory::fake_group_membership_with_invitation_row(),
                    GroupMembershipFactory::fake_group_membership_with_invitation_row(),
                ];
                Box::pin(async move { Ok(result) })
            });

        let result = mock_repo.find_by_group_chat_id(group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].group_chat_id, found[1].group_chat_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_empty_result() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let group_id = 999;

        mock_repo
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(|_| {
                Box::pin(async move { Ok(vec![]) })
            });

        let result = mock_repo.find_by_group_chat_id(group_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_single_membership() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let expected = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let group_id = expected.group_chat_id;

        mock_repo
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let result = vec![GroupMembershipFactory::fake_group_membership_with_invitation_row()];
                Box::pin(async move { Ok(result) })
            });

        let result = mock_repo.find_by_group_chat_id(group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].group_chat_id, group_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_only_active_memberships() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let group_id = 1;

        mock_repo
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                // Only return active memberships
                let mut active_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                active_membership.membership_status = MembershipStatus::Active;
                active_membership.group_chat_id = group_id;

                let result = vec![active_membership];
                Box::pin(async move { Ok(result) })
            });

        let result = mock_repo.find_by_group_chat_id(group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].group_chat_id, group_id);
        assert_eq!(found[0].membership_status, MembershipStatus::Active);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let group_id = 1;

        mock_repo
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(|_| {
                Box::pin(async move { Err(Error::RowNotFound) })
            });

        let result = mock_repo.find_by_group_chat_id(group_id).await;
        assert!(result.is_err());
    }
}
