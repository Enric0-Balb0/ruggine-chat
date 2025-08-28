use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    /// Find all user_ids connected to an auth_user_id
    pub async fn find_connected_users_and_online_internal(&self, auth_user_id: i32) -> Result<Vec<i32>, ApiError> {
        let connected_user_ids = self.group_membership_repo.find_connected_users_and_online(auth_user_id).await.map_err(|e| {
            let db_error = ApiError::DbError(DbError::SomethingWentWrong(e.to_string()));
            db_error
        })?;

        Ok(connected_user_ids)
    }
}

mod find_connected_users_and_online_service_tests {
    use crate::error::api_error::ApiError;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::utils::mock_database_error::MockDatabaseError;
    use mockall::predicate::eq;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_connected_users_and_online_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let auth_user_id = 1;

        // Mock database error for group memberships
        mock_repo
            .expect_find_connected_users_and_online()
            .with(eq(auth_user_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(MockDatabaseError::constraint_violation()) })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_repo),
            Arc::new(mock_group_service),
            Arc::new(mock_user_service),
        );

        let result = service.find_connected_users_and_online(auth_user_id).await;

        assert!(result.is_err());
        if let Err(ApiError::DbError(_)) = result {
            // Expected database error
        } else {
            panic!("Expected DbError, got: {:?}", result);
        }
    }
}
