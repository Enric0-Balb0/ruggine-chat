use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
use crate::repository::group_membership_repository::GroupMembershipRepository;

impl GroupMembershipRepository {
    pub async fn find_by_id_and_user_id_inner(
        &self,
        id: i32,
        user_id: i32,
    ) -> Result<GroupMembershipWithInvitationRow, Error> {


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
            FROM group_membership gm
            JOIN invitation i ON gm.invitation_id = i.id
            WHERE gm.id = $1 AND i.to_user_id = $2
            "#
        )
        .bind(id)
        .bind(user_id);

        // se c'è una transazione, usala; altrimenti usa la pool
        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_one(&mut *tx_ref).await
        } else {
            query.fetch_one(self.db_conn.get_pool()).await
        }
    }
}

#[cfg(test)]
mod group_membership_repository_find_by_id_and_user_id_tests {
    use mockall::predicate::*;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::repository::group_membership_repository::GroupMembershipRepositoryTrait;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::entity::group_membership::{MembershipStatus, MemberRole};
    use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
    use chrono::Utc;
    use sqlx::Error;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let expected = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let id = expected.id;
        let user_id = expected.user_id;

        mock_repo
            .expect_find_by_id_and_user_id()
            .with(eq(id), eq(user_id))
            .times(1)
            .returning(move |_, _| {
                let result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                Box::pin(async move { Ok(result) })
            });

        let result = mock_repo.find_by_id_and_user_id(id, user_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, expected.id);
        assert_eq!(found.user_id, expected.user_id);
        assert_eq!(found.group_chat_id, expected.group_chat_id);
        assert_eq!(found.role, expected.role);
        assert_eq!(found.membership_status, expected.membership_status);
        assert_eq!(found.left_at, expected.left_at);
        assert_eq!(found.current_action, expected.current_action);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_not_found() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();

        mock_repo
            .expect_find_by_id_and_user_id()
            .with(eq(-1), eq(-1))
            .times(1)
            .returning(|_, _| {
                Box::pin(async move {
                    Err(Error::RowNotFound)
                })
            });

        let result = mock_repo.find_by_id_and_user_id(-1, -1).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_admin_role() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut expected = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        expected.role = MemberRole::Admin;

        let id = expected.id;
        let user_id = expected.user_id;

        mock_repo
            .expect_find_by_id_and_user_id()
            .with(eq(id), eq(user_id))
            .times(1)
            .returning(move |_, _| {
                let mut row = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                row.role = MemberRole::Admin;
                Box::pin(async move { Ok(row) })
            });

        let result = mock_repo.find_by_id_and_user_id(id, user_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().role, MemberRole::Admin);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_with_left_at() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut expected = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        expected.left_at = Some(Utc::now());
        expected.membership_status = MembershipStatus::Left;

        let id = expected.id;
        let user_id = expected.user_id;

        mock_repo
            .expect_find_by_id_and_user_id()
            .with(eq(id), eq(user_id))
            .times(1)
            .returning(move |_, _| {
                let mut row = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                row.left_at = expected.left_at;
                row.membership_status = MembershipStatus::Left;
                Box::pin(async move { Ok(row) })
            });

        let result = mock_repo.find_by_id_and_user_id(id, user_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.left_at.is_some());
        assert_eq!(found.membership_status, MembershipStatus::Left);
    }
}

