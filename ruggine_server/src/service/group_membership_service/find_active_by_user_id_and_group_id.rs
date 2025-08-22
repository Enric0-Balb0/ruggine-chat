use sqlx::Error;
use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};

impl GroupMembershipService {
    pub async fn find_active_by_user_id_and_group_id_internal(&self, user_id: i32, group_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        let group_membership = self.group_membership_repo.find_active_by_user_id_and_group_id(user_id, group_id).await.map_err(|e| {
            let db_error = match e {
                sqlx::Error::RowNotFound => ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound),
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            };
            db_error
        })?;

        Ok(GroupMembershipReadDto::from(group_membership))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use mockall::predicate::*;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;

    fn setup_service_with_mock_repo(mock_repo: MockGroupMembershipRepositoryTrait) -> GroupMembershipService {
        let mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        GroupMembershipService::with(
            Arc::new(mock_repo),
            Arc::new(mock_group_service),
            Arc::new(mock_user_service),
        )
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_internal_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let group_id = 10;
        let expected_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let expected_clone = expected_membership.clone();

        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let m = expected_clone.clone();
                Box::pin(async move { Ok(m) })
            });

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_active_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.id, expected_membership.id);
        assert_eq!(dto.user_id, expected_membership.user_id);
        assert_eq!(dto.group_chat_id, expected_membership.group_chat_id);
        assert_eq!(dto.role, expected_membership.role);
        assert_eq!(dto.membership_status, expected_membership.membership_status);
        assert_eq!(dto.current_action, expected_membership.current_action);
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_internal_admin_role() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 2;
        let group_id = 20;
        let expected_membership = GroupMembershipFactory::fake_admin_group_membership_with_invitation_row();
        let expected_clone = expected_membership.clone();

        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let m = expected_clone.clone();
                Box::pin(async move { Ok(m) })
            });

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_active_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.id, expected_membership.id);
        assert_eq!(dto.user_id, expected_membership.user_id);
        assert_eq!(dto.group_chat_id, expected_membership.group_chat_id);
        assert_eq!(dto.role, crate::entity::group_membership::MemberRole::Admin);
        assert_eq!(dto.membership_status, crate::entity::group_membership::MembershipStatus::Active);
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_internal_not_found() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = -1;
        let group_id = -1;

        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(|_, _| Box::pin(async { Err(sqlx::Error::RowNotFound) }));

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_active_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
        ));
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_internal_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let group_id = 1;

        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(|_, _| {
                let db_error = sqlx::Error::Database(Box::new(crate::utils::mock_database_error::MockDatabaseError::new("42000".to_string())));
                Box::pin(async move { Err(db_error) })
            });

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_active_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(matches!(result, Err(ApiError::DbError(DbError::SomethingWentWrong(_)))));
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_internal_wrong_user_id() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 999; // Non-existent user
        let group_id = 1;

        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(|_, _| Box::pin(async { Err(sqlx::Error::RowNotFound) }));

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_active_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
        ));
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_internal_wrong_group_id() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let group_id = 999; // Non-existent group

        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(|_, _| Box::pin(async { Err(sqlx::Error::RowNotFound) }));

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_active_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
        ));
    }

    #[tokio::test]
    async fn test_find_active_by_user_id_and_group_id_internal_zero_ids() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 0;
        let group_id = 0;

        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(|_, _| Box::pin(async { Err(sqlx::Error::RowNotFound) }));

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_active_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
        ));
    }
}