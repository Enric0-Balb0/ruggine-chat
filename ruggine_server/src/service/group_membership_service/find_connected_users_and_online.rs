use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    /// Find all user_ids connected to an auth_user_id
    pub async fn find_connected_users_and_online_internal(&self, auth_user_id: i32) -> Result<Vec<i32>, ApiError> {
        match self.user_service.find_by_id(auth_user_id).await {
            Ok(user) => {
                if !user.is_online {
                    return Err(ApiError::GroupMembershipError(GroupMembershipError::CannotAccessIfUserIsNotOnline))
                }
            }
            Err(e) => return Err(e)
        }
        
        let connected_user_ids = self.group_membership_repo.find_connected_users_and_online(auth_user_id).await.map_err(|e| {
            let db_error = ApiError::DbError(DbError::SomethingWentWrong(e.to_string()));
            db_error
        })?;

        Ok(connected_user_ids)
    }
}

mod find_connected_users_and_online_service_tests {
    use crate::dto::user_dto::UserReadDto;
    use crate::entity::user::{User, UserStatus, UserType, Gender};
    use crate::error::api_error::ApiError;
    use crate::error::group_membership_error::GroupMembershipError;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::utils::mock_database_error::MockDatabaseError;
    use mockall::predicate::eq;
    use std::sync::Arc;
    use chrono::Utc;

    fn create_test_user(user_id: i32, is_online: bool) -> UserReadDto {
        UserReadDto::from(User {
            id: user_id,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username: format!("testuser{}", user_id),
            email: format!("test{}@example.com", user_id),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: UserStatus::Active,
            user_type: UserType::EndUser,
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            is_online,
            address: "Test Address".to_string(),
            gender: Gender::Other,
        })
    }

    #[tokio::test]
    async fn test_find_connected_users_and_online_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();

        let auth_user_id = 1;
        let connected_users = vec![2, 3, 4];

        // Mock user service to return online user
        mock_user_service
            .expect_find_by_id()
            .with(eq(auth_user_id))
            .times(1)
            .returning(move |id| {
                let user = create_test_user(id, true); // User is online
                Box::pin(async move { Ok(user) })
            });

        // Mock successful repository call
        mock_repo
            .expect_find_connected_users_and_online()
            .with(eq(auth_user_id))
            .times(1)
            .returning(move |_| {
                let users = connected_users.clone();
                Box::pin(async move { Ok(users) })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_repo),
            Arc::new(mock_group_service),
            Arc::new(mock_user_service),
        );

        let result = service.find_connected_users_and_online_internal(auth_user_id).await;

        assert!(result.is_ok());
        let user_ids = result.unwrap();
        assert_eq!(user_ids, vec![2, 3, 4]);
    }

    #[tokio::test]
    async fn test_find_connected_users_and_online_user_not_online() {
        let mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();

        let auth_user_id = 1;

        // Mock user service to return offline user
        mock_user_service
            .expect_find_by_id()
            .with(eq(auth_user_id))
            .times(1)
            .returning(move |id| {
                let user = create_test_user(id, false); // User is offline
                Box::pin(async move { Ok(user) })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_repo),
            Arc::new(mock_group_service),
            Arc::new(mock_user_service),
        );

        let result = service.find_connected_users_and_online_internal(auth_user_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::CannotAccessIfUserIsNotOnline) => {
                // Expected error
            }
            e => panic!("Expected CannotAccessIfUserIsNotOnline error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_find_connected_users_and_online_user_not_found() {
        let mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();

        let auth_user_id = 1;

        // Mock user service to return user not found error
        mock_user_service
            .expect_find_by_id()
            .with(eq(auth_user_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { 
                    Err(ApiError::UserError(crate::error::user_error::UserError::UserNotFound))
                })
            });

        let service = GroupMembershipService::with(
            Arc::new(mock_repo),
            Arc::new(mock_group_service),
            Arc::new(mock_user_service),
        );

        let result = service.find_connected_users_and_online_internal(auth_user_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::UserError(crate::error::user_error::UserError::UserNotFound) => {
                // Expected error
            }
            e => panic!("Expected UserNotFound error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_find_connected_users_and_online_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mock_group_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();

        let auth_user_id = 1;

        // Mock user service to return online user
        mock_user_service
            .expect_find_by_id()
            .with(eq(auth_user_id))
            .times(1)
            .returning(move |id| {
                let user = create_test_user(id, true); // User is online
                Box::pin(async move { Ok(user) })
            });

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

        let result = service.find_connected_users_and_online_internal(auth_user_id).await;

        assert!(result.is_err());
        if let Err(ApiError::DbError(_)) = result {
            // Expected database error
        } else {
            panic!("Expected DbError, got: {:?}", result);
        }
    }
}
