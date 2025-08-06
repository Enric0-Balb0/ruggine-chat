use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    pub async fn find_by_id_and_user_id_internal(&self, id: i32, user_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        // First find the membership by ID
        let group_membership = self.group_membership_repo.find_by_id_and_user_id(id, user_id).await.map_err(|e| {
            let db_error = match e {
                sqlx::Error::RowNotFound => ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound),

                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        if code == "23505" {
                            ApiError::DbError(DbError::UniqueConstraintViolation(db_err.to_string()))
                        } else {
                            ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                        }
                    } else {
                        ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                    }
                }

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
    use crate::service::invitation_service::invitation_service_trait::MockInvitationServiceTrait;

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
    async fn test_find_by_id_and_user_id_internal_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let membership_id = 1;
        let user_id = 1;
        let expected_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let expected_clone = expected_membership.clone();

        mock_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(user_id))
            .times(1)
            .returning(move |_, _| {
                let m = expected_clone.clone();
                Box::pin(async move { Ok(m) })
            });

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_by_id_and_user_id_internal(membership_id, user_id).await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.id, expected_membership.id);
        assert_eq!(dto.user_id, expected_membership.user_id);
        assert_eq!(dto.group_chat_id, expected_membership.group_chat_id);
        assert_eq!(dto.role, expected_membership.role);
    }

    #[tokio::test]
    async fn test_find_by_id_and_user_id_internal_not_found() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let membership_id = -1;
        let user_id = -1;

        mock_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(user_id))
            .times(1)
            .returning(|_, _| Box::pin(async { Err(sqlx::Error::RowNotFound) }));

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_by_id_and_user_id_internal(membership_id, user_id).await;

        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
        ));
    }

    #[tokio::test]
    async fn test_find_by_id_and_user_id_internal_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let membership_id = 1;
        let user_id = 1;

        mock_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(user_id))
            .times(1)
            .returning(|_, _| Box::pin(async { Err(sqlx::Error::PoolClosed) }));

        let service = setup_service_with_mock_repo(mock_repo);

        let result = service.find_by_id_and_user_id_internal(membership_id, user_id).await;

        assert!(matches!(
            result,
            Err(ApiError::DbError(DbError::SomethingWentWrong(_)))
        ));
    }
}